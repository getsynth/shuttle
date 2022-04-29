use chrono::{DateTime, Utc};
use log::{Level, Metadata, Record};
use shuttle_common::{DeploymentId, LogItem};
use std::sync::mpsc::SyncSender;

#[derive(Debug)]
pub struct Log {
    pub deployment_id: DeploymentId,
    pub datetime: DateTime<Utc>,
    pub item: LogItem,
}

pub struct Logger {
    deployment_id: DeploymentId,
    tx: SyncSender<Log>,
}

impl Logger {
    pub fn new(tx: SyncSender<Log>, deployment_id: DeploymentId) -> Self {
        Self { tx, deployment_id }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let item = LogItem {
                body: format!("{}", record.args()),
                level: record.level(),
                target: record.target().to_string(),
            };

            self.tx
                .send(Log {
                    item,
                    datetime: Utc::now(),
                    deployment_id: self.deployment_id.clone(),
                })
                .expect("sending log should succeed");
        }
    }

    fn flush(&self) {}
}
