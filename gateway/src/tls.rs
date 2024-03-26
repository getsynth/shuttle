use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::sync::Arc;

use axum_server::accept::DefaultAcceptor;
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use futures::executor::block_on;
use pem::Pem;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::{self, CertifiedKey};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::Item;
use shuttle_common::models::error::ErrorKind;
use tokio::runtime::Handle;
use tokio::sync::RwLock;

use crate::Error;

/// LetsEncrypt recommends to renew a certificate when its close to 30 days validity window.
pub const RENEWAL_VALIDITY_THRESHOLD_IN_DAYS: i64 = 30;

#[derive(Clone)]
pub struct ChainAndPrivateKey {
    chain: Vec<Certificate>,
    private_key: PrivateKey,
}

impl ChainAndPrivateKey {
    pub fn parse_pem<R: Read>(rd: R) -> Result<Self, Error> {
        let mut private_key = None;
        let mut chain = Vec::new();

        for item in rustls_pemfile::read_all(&mut BufReader::new(rd))
            .map_err(|_| Error::from_kind(ErrorKind::Internal))?
        {
            match item {
                Item::X509Certificate(cert) => chain.push(Certificate(cert)),
                Item::ECKey(key) | Item::PKCS8Key(key) | Item::RSAKey(key) => {
                    private_key = Some(PrivateKey(key))
                }
                _ => return Err(Error::from_kind(ErrorKind::Internal)),
            }
        }

        Ok(Self {
            chain,
            private_key: private_key.unwrap(),
        })
    }

    pub fn load_pem<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let rd = File::open(path)?;
        Self::parse_pem(rd)
    }

    pub fn into_pem(self) -> Result<String, Error> {
        let mut pems = Vec::new();
        for cert in self.chain {
            pems.push(Pem {
                tag: "CERTIFICATE".to_string(),
                contents: cert.0,
            });
        }

        pems.push(Pem {
            tag: "PRIVATE KEY".to_string(),
            contents: self.private_key.0,
        });

        Ok(pem::encode_many(&pems))
    }

    pub fn into_certified_key(self) -> Result<CertifiedKey, Error> {
        let signing_key = sign::any_supported_type(&self.private_key)
            .map_err(|_| Error::from_kind(ErrorKind::Internal))?;
        Ok(CertifiedKey::new(self.chain, signing_key))
    }

    pub fn save_pem<P: AsRef<Path>>(self, path: P) -> Result<(), Error> {
        let as_pem = self.into_pem()?;
        let mut f = File::create(path)?;
        f.write_all(as_pem.as_bytes())?;
        Ok(())
    }
}

#[derive(Default)]
pub struct GatewayCertResolver {
    keys: RwLock<HashMap<String, Arc<CertifiedKey>>>,
    default: RwLock<Option<Arc<CertifiedKey>>>,
}

impl GatewayCertResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the loaded [CertifiedKey] associated with the given
    /// domain.
    pub async fn get(&self, sni: &str) -> Option<Arc<CertifiedKey>> {
        self.keys.read().await.get(sni).map(Arc::clone)
    }

    pub async fn serve_default_der(&self, certs: ChainAndPrivateKey) -> Result<(), Error> {
        *self.default.write().await = Some(Arc::new(certs.into_certified_key()?));
        Ok(())
    }

    pub async fn serve_default_pem<R: Read>(&self, rd: R) -> Result<(), Error> {
        let certs = ChainAndPrivateKey::parse_pem(rd)?;
        self.serve_default_der(certs).await
    }

    /// Load a new certificate chain and private key to serve when
    /// receiving incoming TLS connections for the given domain.
    pub async fn serve_der(&self, sni: &str, certs: ChainAndPrivateKey) -> Result<(), Error> {
        let certified_key = certs.into_certified_key()?;
        self.keys
            .write()
            .await
            .insert(sni.to_string(), Arc::new(certified_key));
        Ok(())
    }

    pub async fn serve_pem<R: Read>(&self, sni: &str, rd: R) -> Result<(), Error> {
        let certs = ChainAndPrivateKey::parse_pem(rd)?;
        self.serve_der(sni, certs).await
    }
}

impl ResolvesServerCert for GatewayCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        let sni = client_hello.server_name()?;
        let handle = Handle::current();
        let _ = handle.enter();
        block_on(async move {
            if let Some(cert) = self.get(sni).await {
                Some(cert)
            } else {
                self.default.read().await.clone()
            }
        })
    }
}

pub fn make_tls_acceptor() -> (Arc<GatewayCertResolver>, RustlsAcceptor<DefaultAcceptor>) {
    let resolver = Arc::new(GatewayCertResolver::new());

    let mut server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_cert_resolver(Arc::clone(&resolver) as Arc<dyn ResolvesServerCert>);
    server_config.alpn_protocols = vec![b"http/1.1".to_vec()];

    let rustls_config = RustlsConfig::from_config(Arc::new(server_config));

    (resolver, RustlsAcceptor::new(rustls_config))
}
