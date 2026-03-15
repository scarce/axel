//! JSONL file logger for event persistence.

use std::path::PathBuf;

use anyhow::Result;
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::mpsc};

use super::events::TimestampedEvent;

/// Async event logger that writes to a JSONL file
pub struct EventLogger {
    tx: mpsc::Sender<TimestampedEvent>,
}

impl EventLogger {
    /// Create a new event logger that writes to the specified path
    pub async fn new(path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let (tx, rx) = mpsc::channel::<TimestampedEvent>(1000);

        // Spawn the writer task
        tokio::spawn(writer_task(path, rx));

        Ok(Self { tx })
    }

    /// Log an event (non-blocking)
    pub fn log(&self, event: TimestampedEvent) {
        // Use try_send to avoid blocking; events are dropped if buffer is full
        let _ = self.tx.try_send(event);
    }

    /// Get a clone of the sender for use in handlers
    pub fn sender(&self) -> mpsc::Sender<TimestampedEvent> {
        self.tx.clone()
    }
}

/// Background task that writes events to the JSONL file
async fn writer_task(path: PathBuf, mut rx: mpsc::Receiver<TimestampedEvent>) {
    let file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file {:?}: {}", path, e);
            return;
        }
    };

    let mut writer = tokio::io::BufWriter::new(file);

    while let Some(event) = rx.recv().await {
        match serde_json::to_string(&event) {
            Ok(json) => {
                if let Err(e) = writer.write_all(json.as_bytes()).await {
                    eprintln!("Failed to write event: {}", e);
                    continue;
                }
                if let Err(e) = writer.write_all(b"\n").await {
                    eprintln!("Failed to write newline: {}", e);
                    continue;
                }
                // Flush periodically to ensure events are written
                if let Err(e) = writer.flush().await {
                    eprintln!("Failed to flush log file: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize event: {}", e);
            }
        }
    }
}
