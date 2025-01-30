use serde::{Deserialize, Serialize};

/// Status of a telemetry export configuration for an external sink
#[derive(Eq, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct TelemetrySinkStatus {
    /// Indicates that the associated project is configured to export telemetry data to this sink
    enabled: bool,
}

/// A safe-for-display representation of the current telemetry export configuration for a given project
#[derive(Eq, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct ProjectTelemetryConfigResponse {
    betterstack: Option<TelemetrySinkStatus>,
    datadog: Option<TelemetrySinkStatus>,
    grafana_cloud: Option<TelemetrySinkStatus>,
}

impl From<Vec<ProjectTelemetrySinkConfig>> for ProjectTelemetryConfigResponse {
    fn from(value: Vec<ProjectTelemetrySinkConfig>) -> Self {
        let mut instance = Self::default();

        for sink in value {
            match sink {
                ProjectTelemetrySinkConfig::Betterstack(_) => {
                    instance.betterstack = Some(TelemetrySinkStatus { enabled: true })
                }
                ProjectTelemetrySinkConfig::Datadog(_) => {
                    instance.datadog = Some(TelemetrySinkStatus { enabled: true })
                }
                ProjectTelemetrySinkConfig::GrafanaCloud(_) => {
                    instance.grafana_cloud = Some(TelemetrySinkStatus { enabled: true })
                }
            }
        }

        instance
    }
}

/// The user-supplied config required to export telemetry to a given external sink
#[derive(Eq, Clone, PartialEq, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[typeshare::typeshare]
pub enum ProjectTelemetrySinkConfig {
    /// [Betterstack](https://betterstack.com/docs/logs/open-telemetry/)
    Betterstack(BetterstackConfig),
    /// [Datadog](https://docs.datadoghq.com/opentelemetry/collector_exporter/otel_collector_datadog_exporter)
    Datadog(DatadogConfig),
    /// [Grafana Cloud](https://grafana.com/docs/grafana-cloud/send-data/otlp/)
    GrafanaCloud(GrafanaCloudConfig),
}

impl ProjectTelemetrySinkConfig {
    pub fn as_db_type(&self) -> String {
        format!("project::telemetry::{}::config", self.as_ref())
    }
}

#[derive(Eq, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct BetterstackConfig {
    pub source_token: String,
}
#[derive(Eq, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct DatadogConfig {
    pub api_key: String,
}
#[derive(Eq, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare::typeshare]
pub struct GrafanaCloudConfig {
    pub token: String,
    pub endpoint: String,
    pub instance_id: String,
}
