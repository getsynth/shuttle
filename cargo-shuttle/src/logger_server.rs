use std::net::SocketAddr;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use opentelemetry_proto::tonic::{
    collector::logs::v1::{
        logs_service_server::{LogsService, LogsServiceServer},
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    },
    logs::v1::LogRecord,
};
use shuttle_common::{
    backends::tracing::{from_any_value_kv_to_serde_json_map, from_any_value_to_serde_json_value},
    tracing::{FILEPATH_KEY, LINENO_KEY, MESSAGE_KEY, TARGET_KEY},
    LogItem,
};
use tokio::task::JoinHandle;
use tonic::{
    transport::{self, Server},
    Request, Response, Status,
};

pub struct LocalLogger;

impl LocalLogger {
    pub fn new() -> Self {
        Self
    }

    pub fn start(self, address: SocketAddr) -> JoinHandle<Result<(), transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(LogsServiceServer::new(self))
                .serve(address)
                .await
        })
    }
}

#[async_trait]
impl LogsService for LocalLogger {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let request = request.into_inner();

        let logs = request.resource_logs.into_iter().flat_map(|logs| {
            logs.scope_logs
                .into_iter()
                .flat_map(|scope| scope.log_records.into_iter().flat_map(try_from_log_record))
        });

        for log in logs {
            println!("{log}");
        }

        Ok(Response::new(Default::default()))
    }
}

/// Try to get a [LogItem] from an OTLP [LogRecord]
fn try_from_log_record(log_record: LogRecord) -> Option<LogItem> {
    let level = log_record.severity_number().into();
    let naive = NaiveDateTime::from_timestamp_opt(
        (log_record.time_unix_nano / 1_000_000_000)
            .try_into()
            .unwrap_or_default(),
        (log_record.time_unix_nano % 1_000_000_000) as u32,
    )
    .unwrap_or_default();
    let mut fields = from_any_value_kv_to_serde_json_map(log_record.attributes);
    let message = from_any_value_to_serde_json_value(log_record.body?);

    fields.insert(MESSAGE_KEY.to_string(), message);

    Some(LogItem {
        id: Default::default(),
        timestamp: DateTime::from_utc(naive, Utc),
        state: shuttle_common::deployment::State::Running,
        level,
        file: fields
            .remove(FILEPATH_KEY)
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        line: fields
            .remove(LINENO_KEY)
            .and_then(|v| v.as_u64())
            .map(|u| u as u32),
        target: fields
            .remove(TARGET_KEY)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default(),
        fields: serde_json::to_vec(&serde_json::Value::Object(fields)).unwrap_or_default(),
    })
}
