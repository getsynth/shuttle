use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use fqdn::FQDN;
use http::StatusCode;
use instant_acme::{
    Account, AccountCredentials, Authorization, AuthorizationStatus, Challenge, ChallengeType,
    Identifier, KeyAuthorization, LetsEncrypt, NewAccount, NewOrder, Order, OrderStatus,
};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use shuttle_backends::project_name::ProjectName;
use shuttle_common::models::error::ApiError;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, trace, warn};

const MAX_RETRIES: usize = 15;
const MAX_RETRIES_CERTIFICATE_FETCHING: usize = 5;

#[derive(Debug, Eq, PartialEq)]
pub struct CustomDomain {
    pub fqdn: FQDN,
    pub project_name: ProjectName,
    pub certificate: String,
    pub private_key: String,
}

/// An ACME client implementation that completes Http01 challenges
/// It is safe to clone this type as it functions as a singleton
#[derive(Clone, Default)]
pub struct AcmeClient(Arc<Mutex<HashMap<String, KeyAuthorization>>>);

impl AcmeClient {
    pub fn new() -> Self {
        Self::default()
    }

    async fn add_http01_challenge_authorization(&self, token: String, key: KeyAuthorization) {
        trace!(token, "saving acme http01 challenge");
        self.0.lock().await.insert(token, key);
    }

    pub async fn get_http01_challenge_authorization(&self, token: &str) -> Option<String> {
        self.0
            .lock()
            .await
            .get(token)
            .map(|key| key.as_str().to_owned())
    }

    async fn remove_http01_challenge_authorization(&self, token: &str) {
        trace!(token, "removing acme http01 challenge");
        self.0.lock().await.remove(token);
    }

    /// Create a new ACME account that can be restored by using the deserialization
    /// of the returned JSON into a [instant_acme::AccountCredentials]
    pub async fn create_account(
        &self,
        email: &str,
        acme_server: Option<String>,
    ) -> Result<serde_json::Value, AcmeClientError> {
        let acme_server = acme_server.unwrap_or_else(|| LetsEncrypt::Production.url().to_string());

        trace!(email, acme_server, "creating acme account");

        let account: NewAccount = NewAccount {
            contact: &[&format!("mailto:{email}")],
            terms_of_service_agreed: true,
            only_return_existing: false,
        };

        let account = Account::create(&account, &acme_server)
            .await
            .map_err(|error| {
                error!(
                    error = &error as &dyn std::error::Error,
                    "got error while creating acme account"
                );
                AcmeClientError::AccountCreation
            })?;

        let credentials = serde_json::to_value(account.credentials()).map_err(|error| {
            error!(
                error = &error as &dyn std::error::Error,
                "got error while extracting credentials from acme account"
            );
            AcmeClientError::Serializing
        })?;

        Ok(credentials)
    }

    /// Create an ACME-signed certificate and return it and its
    /// associated PEM-encoded private key
    pub async fn create_certificate(
        &self,
        identifier: &str,
        challenge_type: ChallengeType,
        credentials: AccountCredentials<'_>,
    ) -> Result<(String, String), AcmeClientError> {
        trace!(identifier, "requesting acme certificate");

        let mut order = AccountWrapper::from(credentials)
            .0
            .new_order(&NewOrder {
                identifiers: &[Identifier::Dns(identifier.to_string())],
            })
            .await
            .map_err(|error| {
                error!(
                    error = &error as &dyn std::error::Error,
                    "failed to order certificate"
                );
                AcmeClientError::OrderCreation
            })?;

        let authorizations = order.authorizations().await.map_err(|error| {
            error!(
                error = &error as &dyn std::error::Error,
                "failed to get authorizations information"
            );
            AcmeClientError::AuthorizationCreation
        })?;

        // There should only ever be 1 authorization as we only provide 1 domain at a time
        debug_assert!(authorizations.len() == 1);
        let authorization = &authorizations[0];

        trace!(?authorization, "got authorization");

        self.complete_challenge(challenge_type, authorization, &mut order)
            .await?;

        let certificate = {
            let mut params = CertificateParams::new(vec![identifier.to_owned()]);
            params.distinguished_name = DistinguishedName::new();
            Certificate::from_params(params).map_err(|error| {
                error!(
                    error = &error as &dyn std::error::Error,
                    "failed to create certificate"
                );
                AcmeClientError::CertificateCreation
            })?
        };
        let signing_request = certificate.serialize_request_der().map_err(|error| {
            error!(
                error = &error as &dyn std::error::Error,
                "failed to create certificate signing request"
            );
            AcmeClientError::CertificateSigning
        })?;

        order.finalize(&signing_request).await.map_err(|error| {
            error!(
                error = &error as &dyn std::error::Error,
                "failed to finalize certificate request"
            );
            AcmeClientError::OrderFinalizing
        })?;

        // Poll for certificate, do this for few rounds.
        let mut res: Option<String> = None;
        let mut retries = MAX_RETRIES_CERTIFICATE_FETCHING;
        while res.is_none() && retries > 0 {
            res = order.certificate().await.map_err(|error| {
                error!(
                    error = &error as &dyn std::error::Error,
                    "failed to fetch the certificate chain"
                );
                AcmeClientError::CertificateCreation
            })?;
            retries -= 1;
            sleep(Duration::from_secs(1)).await;
        }

        Ok((
            res.expect("panicked when returning the certificate chain"),
            certificate.serialize_private_key_pem(),
        ))
    }

    fn find_challenge(
        ty: ChallengeType,
        authorization: &Authorization,
    ) -> Result<&Challenge, AcmeClientError> {
        authorization
            .challenges
            .iter()
            .find(|c| c.r#type == ty)
            .ok_or_else(|| {
                let error = AcmeClientError::MissingChallenge;
                error!(
                    error = &error as &dyn std::error::Error,
                    "http-01 challenge not found"
                );
                error
            })
    }

    async fn wait_for_termination(&self, order: &mut Order) -> Result<(), AcmeClientError> {
        // Exponential backoff until order changes status
        let mut tries = 1;
        let mut delay = Duration::from_millis(250);
        let state = loop {
            sleep(delay).await;
            let state = order.refresh().await.map_err(|error| {
                error!(
                    error = &error as &dyn std::error::Error,
                    "got error while fetching state"
                );
                AcmeClientError::FetchingState
            })?;

            trace!(?state, "order state refreshed");
            match state.status {
                OrderStatus::Ready => break state,
                OrderStatus::Invalid => {
                    return Err(AcmeClientError::ChallengeInvalid);
                }
                OrderStatus::Pending => {
                    delay *= 2;
                    tries += 1;
                    if tries < MAX_RETRIES {
                        trace!(?state, tries, attempt_in=?delay, "order not yet ready");
                    } else {
                        let error = AcmeClientError::ChallengeTimeout;
                        error!(
                            error = &error as &dyn std::error::Error,
                            ?state,
                            tries,
                            "order not ready in {MAX_RETRIES} tries"
                        );
                        return Err(error);
                    }
                }
                _ => unreachable!(),
            }
        };

        trace!(?state, "challenge completed");

        Ok(())
    }

    async fn complete_challenge(
        &self,
        ty: ChallengeType,
        authorization: &Authorization,
        order: &mut Order,
    ) -> Result<(), AcmeClientError> {
        // Don't complete challenge for orders that are already valid
        if let AuthorizationStatus::Valid = authorization.status {
            return Ok(());
        }
        let challenge = Self::find_challenge(ty, authorization)?;
        match ty {
            ChallengeType::Http01 => self.complete_http01_challenge(challenge, order).await,
            ChallengeType::Dns01 => {
                self.complete_dns01_challenge(&authorization.identifier, challenge, order)
                    .await
            }
            _ => Err(AcmeClientError::ChallengeNotSupported),
        }
    }

    async fn complete_dns01_challenge(
        &self,
        identifier: &Identifier,
        challenge: &Challenge,
        order: &mut Order,
    ) -> Result<(), AcmeClientError> {
        let Identifier::Dns(domain) = identifier;

        let digest = order.key_authorization(challenge).dns_value();
        warn!("dns-01 challenge: _acme-challenge.{domain} 300 IN TXT \"{digest}\"");

        // Wait 60 secs to insert the record manually and for it to
        // propagate before moving on
        sleep(Duration::from_secs(60)).await;

        order
            .set_challenge_ready(&challenge.url)
            .await
            .map_err(|error| {
                error!(
                    error = &error as &dyn std::error::Error,
                    "failed to mark challenge as ready"
                );
                AcmeClientError::SetReadyFailed
            })?;

        self.wait_for_termination(order).await
    }

    async fn complete_http01_challenge(
        &self,
        challenge: &Challenge,
        order: &mut Order,
    ) -> Result<(), AcmeClientError> {
        trace!(?challenge, "will complete challenge");

        self.add_http01_challenge_authorization(
            challenge.token.clone(),
            order.key_authorization(challenge),
        )
        .await;

        order
            .set_challenge_ready(&challenge.url)
            .await
            .map_err(|error| {
                error!(
                    error = &error as &dyn std::error::Error,
                    "failed to mark challenge as ready"
                );
                AcmeClientError::SetReadyFailed
            })?;

        let res = self.wait_for_termination(order).await;

        self.remove_http01_challenge_authorization(&challenge.token)
            .await;

        res
    }
}

#[derive(Clone)]
pub struct AccountWrapper(pub Account);

impl<'a> From<AccountCredentials<'a>> for AccountWrapper {
    fn from(value: AccountCredentials<'a>) -> Self {
        AccountWrapper(
            Account::from_credentials(value)
                .map_err(|error| {
                    error!(
                        error = &error as &dyn std::error::Error,
                        "failed to convert acme credentials into account"
                    );
                })
                .expect("Malformed account credentials."),
        )
    }
}

#[derive(Debug, strum::Display)]
pub enum AcmeClientError {
    AccountCreation,
    AuthorizationCreation,
    CertificateCreation,
    CertificateSigning,
    ChallengeInvalid,
    ChallengeTimeout,
    FetchingState,
    OrderCreation,
    OrderFinalizing,
    MissingChallenge,
    ChallengeNotSupported,
    Serializing,
    SetReadyFailed,
}

impl std::error::Error for AcmeClientError {}

impl From<AcmeClientError> for ApiError {
    fn from(value: AcmeClientError) -> Self {
        Self {
            message: value.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        }
    }
}
