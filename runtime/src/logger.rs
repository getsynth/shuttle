use opentelemetry_proto::tonic::{
    collector::logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest},
    common::v1::InstrumentationScope,
    logs::v1::{ResourceLogs, ScopeLogs},
};
use shuttle_common::{
    backends::tracing::{into_log_record, serde_json_map_to_key_value_list},
    tracing::JsonVisitor,
};
use tokio::sync::mpsc;
use tracing::{
    error,
    span::{Attributes, Id},
    Level, Metadata, Subscriber,
};
use tracing_subscriber::Layer;

/// Record a single log
pub trait LogRecorder: Send + Sync {
    fn record_log(&self, visitor: JsonVisitor, metadata: &Metadata);
}

/// Recorder to send logs over OTLP
pub struct OtlpRecorder {
    tx: mpsc::UnboundedSender<ScopeLogs>,
    deployment_id: String,
}

impl OtlpRecorder {
    /// Send deployment logs to `destination`. Also mark all logs as belonging to this `deployment_id`
    pub fn new(deployment_id: &str, destination: &str) -> Self {
        let destination = destination.to_string();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let resource_attributes = vec![("deployment_id".into(), deployment_id.into())];
        let resource_attributes =
            serde_json_map_to_key_value_list(serde_json::Map::from_iter(resource_attributes));

        let resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
            attributes: resource_attributes,
            ..Default::default()
        });

        tokio::spawn(async move {
            match LogsServiceClient::connect(destination).await {
                Ok(mut otlp_client) => {
                    while let Some(scope_logs) = rx.recv().await {
                        let resource_log = ResourceLogs {
                            scope_logs: vec![scope_logs],
                            resource: resource.clone(),
                            ..Default::default()
                        };
                        let request = tonic::Request::new(ExportLogsServiceRequest {
                            resource_logs: vec![resource_log],
                        });

                        if let Err(error) = otlp_client.export(request).await {
                            error!(
                        error = &error as &dyn std::error::Error,
                        "Otlp deployment log recorder encountered error while exporting the logs"
                    );
                        };
                    }
                }
                Err(error) => {
                    error!(
                        error = &error as &dyn std::error::Error,
                        "Could not connect to OTLP collector for logs. No logs will be send"
                    );

                    // Consume the logs so that the channel does not overflow
                    while let Some(_scope_logs) = rx.recv().await {}
                }
            };
        });
        Self {
            tx,
            deployment_id: deployment_id.to_string(),
        }
    }
}

impl LogRecorder for OtlpRecorder {
    fn record_log(&self, visitor: JsonVisitor, metadata: &Metadata) {
        let log_record = into_log_record(visitor, metadata);

        let scope_attributes = vec![("deployment_id".into(), self.deployment_id.clone().into())];
        let scope_attributes =
            serde_json_map_to_key_value_list(serde_json::Map::from_iter(scope_attributes));

        let scope_logs = ScopeLogs {
            scope: Some(InstrumentationScope {
                attributes: scope_attributes,
                ..Default::default()
            }),
            log_records: vec![log_record],
            ..Default::default()
        };

        if let Err(error) = self.tx.send(scope_logs) {
            error!(
                error = &error as &dyn std::error::Error,
                "Failed to send deployment log in recorder"
            );
        }
    }
}

pub struct Logger<R> {
    recorder: R,
}

impl<R> Logger<R> {
    pub fn new(recorder: R) -> Self {
        Self { recorder }
    }
}

impl<S, R> Layer<S> for Logger<R>
where
    S: Subscriber,
    R: LogRecorder + Send + Sync + 'static,
{
    fn on_new_span(
        &self,
        attrs: &Attributes,
        _id: &Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = attrs.metadata();
        let level = metadata.level();

        // Ignore span logs from the default level for #[instrument] (INFO) and below (greater than).
        // TODO: make this configurable
        if level >= &Level::INFO {
            return;
        }

        let mut visitor = JsonVisitor::default();
        attrs.record(&mut visitor);

        // Make the span name the log message
        visitor.fields.insert(
            "message".to_string(),
            format!("[span] {}", metadata.name()).into(),
        );

        self.recorder.record_log(visitor, metadata);
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = JsonVisitor::default();

        event.record(&mut visitor);
        let metadata = event.metadata();

        self.recorder.record_log(visitor, metadata);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::*;

    use tracing_subscriber::prelude::*;

    #[derive(Default, Clone)]
    struct DummyRecorder {
        lines: Arc<Mutex<VecDeque<(Level, String)>>>,
    }

    impl LogRecorder for DummyRecorder {
        fn record_log(&self, visitor: JsonVisitor, metadata: &Metadata) {
            self.lines.lock().unwrap().push_back((
                *metadata.level(),
                visitor
                    .fields
                    .get("message")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
    }

    #[test]
    fn logging() {
        let recorder = DummyRecorder::default();
        let logger = Logger::new(recorder.clone());

        let _guard = tracing_subscriber::registry().with(logger).set_default();

        let span = tracing::info_span!("this is an info span");
        span.in_scope(|| {
            tracing::debug!("this is");
            tracing::info!("hi");
        });
        let span = tracing::warn_span!("this is a warn span");
        span.in_scope(|| {
            tracing::warn!("from");
            tracing::error!("logger");
        });

        assert_eq!(
            recorder.lines.lock().unwrap().pop_front(),
            Some((Level::DEBUG, "this is".to_string()))
        );
        assert_eq!(
            recorder.lines.lock().unwrap().pop_front(),
            Some((Level::INFO, "hi".to_string()))
        );
        assert_eq!(
            recorder.lines.lock().unwrap().pop_front(),
            Some((Level::WARN, "[span] this is a warn span".to_string()))
        );
        assert_eq!(
            recorder.lines.lock().unwrap().pop_front(),
            Some((Level::WARN, "from".to_string()))
        );
        assert_eq!(
            recorder.lines.lock().unwrap().pop_front(),
            Some((Level::ERROR, "logger".to_string()))
        );
    }
}
