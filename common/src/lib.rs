#[cfg(feature = "service")]
pub mod certificate;
#[cfg(feature = "claims")]
pub mod claims;
pub mod constants;
pub mod database;
#[cfg(feature = "service")]
pub mod deployment;
#[cfg(feature = "service")]
pub type DeploymentId = uuid::Uuid;
#[cfg(feature = "extract_propagation")]
pub mod extract_propagation;
#[cfg(feature = "claims")]
pub mod limits;
#[cfg(feature = "service")]
pub mod log;
#[cfg(feature = "service")]
pub use log::LogItem;
#[cfg(feature = "service")]
pub use log::LogItemBeta;
#[cfg(feature = "models")]
pub mod models;
pub mod resource;
pub mod secrets;
pub use secrets::{Secret, SecretStore};
pub mod templates;
#[cfg(feature = "tracing")]
pub mod tracing;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

////// Resource Input/Output types

/// The input given to Shuttle DB resources
#[derive(Clone, Deserialize, Serialize, Default)]
pub struct DbInput {
    pub local_uri: Option<String>,
    /// Override the default db name. Only applies to RDS.
    pub db_name: Option<String>,
}

/// The output produced by Shuttle DB resources
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum DatabaseResource {
    ConnectionString(String),
    Info(DatabaseInfo),
}

/// Holds the data for building a database connection string.
///
/// Use [`Self::connection_string_shuttle`] when running on Shuttle,
/// otherwise [`Self::connection_string_public`] for the public URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    engine: String,
    role_name: String,
    role_password: Secret<String>,
    database_name: String,
    port: String,
    // aliases to parse older versions of this struct
    #[serde(alias = "address_private")]
    hostname_shuttle: String,
    #[serde(alias = "address_public")]
    hostname_public: String,
}

impl DatabaseInfo {
    pub fn new(
        engine: String,
        role_name: String,
        role_password: String,
        database_name: String,
        port: String,
        hostname_shuttle: String,
        hostname_public: String,
    ) -> Self {
        Self {
            engine,
            role_name,
            role_password: Secret::new(role_password),
            database_name,
            port,
            hostname_shuttle,
            hostname_public,
        }
    }
    /// For connecting to the db from inside the Shuttle network
    pub fn connection_string_shuttle(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.engine,
            self.role_name,
            self.role_password.expose(),
            self.hostname_shuttle,
            self.port,
            self.database_name,
        )
    }
    /// For connecting to the db from the Internet
    pub fn connection_string_public(&self, show_password: bool) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.engine,
            self.role_name,
            if show_password {
                self.role_password.expose()
            } else {
                self.role_password.redacted()
            },
            self.hostname_public,
            self.port,
            self.database_name,
        )
    }

    pub fn role_name(&self) -> String {
        self.role_name.to_string()
    }

    pub fn database_name(&self) -> String {
        self.database_name.to_string()
    }
}

/// Holds the data for building a database connection string on the Beta platform.
#[derive(Clone, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct DatabaseInfoBeta {
    engine: String,
    role_name: String,
    role_password: String,
    database_name: String,
    port: String,
    #[serde(alias = "hostname_shuttle")] // compatibility to parse from a `DatabaseInfo`
    hostname: String,
    /// The RDS instance name, which is required for deleting provisioned RDS instances, it's
    /// optional because it isn't needed for shared PG deletion.
    instance_name: Option<String>,
}

impl DatabaseInfoBeta {
    pub fn new(
        engine: String,
        role_name: String,
        role_password: String,
        database_name: String,
        port: String,
        hostname: String,
        instance_name: Option<String>,
    ) -> Self {
        Self {
            engine,
            role_name,
            role_password,
            database_name,
            port,
            hostname,
            instance_name,
        }
    }

    /// For connecting to the database.
    pub fn connection_string(&self, show_password: bool) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.engine,
            self.role_name,
            if show_password {
                &self.role_password
            } else {
                "********"
            },
            self.hostname,
            self.port,
            self.database_name,
        )
    }

    pub fn role_name(&self) -> String {
        self.role_name.to_string()
    }

    pub fn database_name(&self) -> String {
        self.database_name.to_string()
    }

    pub fn instance_name(&self) -> Option<String> {
        self.instance_name.clone()
    }
}

// Don't leak password in Debug
impl std::fmt::Debug for DatabaseInfoBeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseInfo {{ {:?} }}", self.connection_string(false))
    }
}

/// Used to request a container from the local run provisioner
#[derive(Serialize, Deserialize)]
pub struct ContainerRequest {
    pub project_name: String,
    /// Type of container, used in the container name. ex "qdrant"
    pub container_name: String,
    /// ex. "qdrant/qdrant:latest"
    pub image: String,
    /// The internal port that the container should expose. ex. "6334/tcp"
    pub port: String,
    /// list of "KEY=value" strings
    pub env: Vec<String>,
}

/// Response from requesting a container from the local run provisioner
#[derive(Serialize, Deserialize)]
pub struct ContainerResponse {
    /// The port that the container exposes to the host.
    /// Is a string for parity with the Docker respose.
    pub host_port: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version of gateway
    pub gateway: semver::Version,
    /// Latest version of cargo-shuttle compatible with this gateway.
    pub cargo_shuttle: semver::Version,
    /// Latest version of shuttle-deployer compatible with this gateway.
    pub deployer: semver::Version,
    /// Latest version of shuttle-runtime compatible with the above deployer.
    pub runtime: semver::Version,
}

/// Check if two versions are compatible based on the rule used by cargo:
/// "Versions `a` and `b` are compatible if their left-most nonzero digit is the same."
pub fn semvers_are_compatible(a: &semver::Version, b: &semver::Version) -> bool {
    if a.major != 0 || b.major != 0 {
        a.major == b.major
    } else if a.minor != 0 || b.minor != 0 {
        a.minor == b.minor
    } else {
        a.patch == b.patch
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    #[test]
    fn semver_compatibility_check_works() {
        let semver_tests = &[
            ("1.0.0", "1.0.0", true),
            ("1.8.0", "1.0.0", true),
            ("0.1.0", "0.2.1", false),
            ("0.9.0", "0.2.0", false),
        ];
        for (version_a, version_b, are_compatible) in semver_tests {
            let version_a = semver::Version::from_str(version_a).unwrap();
            let version_b = semver::Version::from_str(version_b).unwrap();
            assert_eq!(
                super::semvers_are_compatible(&version_a, &version_b),
                *are_compatible
            );
        }
    }
}
