#[cfg(config)]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(config)]
use std::collections::HashMap;
use std::{collections::BTreeMap, fmt::Debug};
use zeroize::Zeroize;

/// Wrapper type for secret values such as passwords or authentication keys.
///
/// Once wrapped, the inner value cannot leak accidentally, as both the [`std::fmt::Display`] and [`Debug`]
/// implementations cover up the actual value and only show the type.
///
/// If you need access to the inner value, there is an [expose](`Secret::expose`) method.
///
/// To make sure nothing leaks after the [`Secret`] has been dropped, a custom [`Drop`]
/// implementation will zero-out the underlying memory.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED {:?}]", std::any::type_name::<T>())
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl<T: Zeroize> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Zeroize> Secret<T> {
    pub fn new(secret: T) -> Self {
        Self(secret)
    }

    /// Expose the underlying value of the secret
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Display a placeholder for the secret
    pub fn redacted(&self) -> &str {
        "********"
    }
}

/// Store that holds all the secrets available to a deployment
#[derive(Deserialize, Serialize, Clone)]
#[serde(transparent)]
pub struct SecretStore {
    pub(crate) secrets: BTreeMap<String, Secret<String>>,
}

impl SecretStore {
    pub fn new(secrets: BTreeMap<String, Secret<String>>) -> Self {
        Self { secrets }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.secrets.get(key).map(|s| s.expose().to_owned())
    }
}

impl IntoIterator for SecretStore {
    type Item = (String, String);
    type IntoIter = <BTreeMap<String, String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.secrets
            .into_iter()
            .map(|(k, s)| (k, s.expose().to_owned()))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
    }
}

#[cfg(config)]
impl SecretStore {
    pub fn deserialize(self) -> T
    where
        T: DeserializeOwned,
    {
        let secrets = self.into_iter().collect::<HashMap<_, _>>();

        config::Config::builder()
            .add_source(
                config::Environment::default()
                    .source(Some(secrets))
                    .try_parsing(true)
                    .separator("__"),
            )
            .build()
            .expect("Failed to load app configuration")
            .try_deserialize()
            .expect("Cannot deserialize configuration")
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod secrets_tests {
    use super::*;

    #[test]
    fn redacted() {
        let password_string = String::from("VERYSECRET");
        let secret = Secret::new(password_string);
        assert_eq!(secret.redacted(), "********");
    }

    #[test]
    fn debug() {
        let password_string = String::from("VERYSECRET");
        let secret = Secret::new(password_string);
        let printed = format!("{:?}", secret);
        assert_eq!(printed, "[REDACTED \"alloc::string::String\"]");
    }

    #[test]
    fn expose() {
        let password_string = String::from("VERYSECRET");
        let secret = Secret::new(password_string);
        let printed = secret.expose();
        assert_eq!(printed, "VERYSECRET");
    }

    #[test]
    fn secret_struct() {
        #[derive(Debug)]
        struct Wrapper {
            password: Secret<String>,
        }

        let password_string = String::from("VERYSECRET");
        let secret = Secret::new(password_string);
        let wrapper = Wrapper { password: secret };
        let printed = format!("{:?}", wrapper);
        assert_eq!(
            printed,
            "Wrapper { password: [REDACTED \"alloc::string::String\"] }"
        );
    }

    #[test]
    fn secretstore_intoiter() {
        let bt = BTreeMap::from([
            ("1".to_owned(), "2".to_owned().into()),
            ("3".to_owned(), "4".to_owned().into()),
        ]);
        let ss = SecretStore::new(bt);

        let mut iter = ss.into_iter();
        assert_eq!(iter.next(), Some(("1".to_owned(), "2".to_owned())));
        assert_eq!(iter.next(), Some(("3".to_owned(), "4".to_owned())));
        assert_eq!(iter.next(), None);
    }
}
