use anyhow::bail;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(PartialEq, Eq, Hash, sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct ApiKey(String);

impl Zeroize for ApiKey {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl ApiKey {
    pub fn parse(key: &str) -> anyhow::Result<Self> {
        let key = key.trim();

        let mut errors = vec![];
        if !key.chars().all(char::is_alphanumeric) {
            errors.push("The API key should consist of only alphanumeric characters.");
        }

        if key.len() != 16 {
            errors.push("The API key should be exactly 16 characters in length.");
        }

        if !errors.is_empty() {
            let message = errors.join("\n");
            bail!("Invalid API key:\n{message}")
        }

        Ok(Self(key.to_string()))
    }

    pub fn generate() -> Self {
        Self(Alphanumeric.sample_string(&mut rand::thread_rng(), 16))
    }
}

impl AsRef<str> for ApiKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        // The API key should be a 16 character alphanumeric string.
        fn parses_valid_api_keys(s in "[a-zA-Z0-9]{16}") {
            ApiKey::parse(&s).unwrap();
        }
    }

    #[test]
    fn generated_api_key_is_valid() {
        let key = ApiKey::generate();

        assert!(ApiKey::parse(key.as_ref()).is_ok());
    }

    #[test]
    #[should_panic(expected = "The API key should be exactly 16 characters in length.")]
    fn invalid_api_key_length() {
        ApiKey::parse("tooshort").unwrap();
    }

    #[test]
    #[should_panic(expected = "The API key should consist of only alphanumeric characters.")]
    fn non_alphanumeric_api_key() {
        ApiKey::parse("dh9z58jttoes3qv@").unwrap();
    }
}
