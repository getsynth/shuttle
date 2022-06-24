use super::{BuildLogReceiver, BuildLogSender};

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use tokio::sync::{broadcast, Mutex, RwLock};

const BUFFER_SIZE: usize = 10;

#[derive(Clone)]
pub struct BuildLogsManager {
    deployments: Arc<Mutex<HashMap<String, Deployment>>>,
}

impl BuildLogsManager {
    pub fn new() -> Self {
        BuildLogsManager {
            deployments: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn for_deployment(&self, name: String) -> BuildLogWriter {
        let (sender, mut receiver) = broadcast::channel(BUFFER_SIZE);

        let logs_so_far = Arc::new(RwLock::new(Vec::new()));

        let logs_so_far_clone = logs_so_far.clone();
        // This Tokio task is responsible for receiving build log lines and
        // storing them in the `logs_so_far` vector.
        let log_store_task_handle = tokio::spawn(async move {
            while let Ok(line) = receiver.recv().await {
                logs_so_far_clone.write().await.push(line);
            }
        });

        self.deployments.lock().await.insert(
            name,
            Deployment {
                sender: sender.clone(),
                logs_so_far: logs_so_far.clone(),
                log_store_task_handle,
            },
        );

        BuildLogWriter {
            sender,
            buffer: String::new(),
        }
    }

    pub async fn delete_deployment(&self, name: &str) {
        self.deployments.lock().await.remove(name);
    }

    pub async fn take_receiver(&self, name: &str) -> Option<BuildLogReceiver> {
        self.deployments
            .lock()
            .await
            .get(name)
            .map(|d| d.sender.subscribe())
    }

    pub async fn get_logs_so_far(&self, name: &str) -> Option<Vec<String>> {
        if let Some(deployment) = self.deployments.lock().await.get(name) {
            Some(deployment.logs_so_far.read().await.clone())
        } else {
            None
        }
    }
}

struct Deployment {
    sender: BuildLogSender,
    logs_so_far: Arc<RwLock<Vec<String>>>,
    log_store_task_handle: tokio::task::JoinHandle<()>,
}

impl Drop for Deployment {
    fn drop(&mut self) {
        self.log_store_task_handle.abort();
    }
}

pub struct BuildLogWriter {
    sender: BuildLogSender,
    buffer: String,
}

impl io::Write for BuildLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for c in buf {
            let c = *c as char;

            if c == '\n' {
                self.flush()?;
            } else {
                self.buffer.push(c);
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let sender = self.sender.clone();
        let msg = self.buffer.clone();

        self.buffer.clear();

        std::thread::spawn(move || {
            let _ = sender.send(msg);
        })
        .join()
        .unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn abc() {}
}
