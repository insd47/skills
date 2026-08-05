//! endpoint 검증과 length-framed Unix socket의 단일 receive pump를 소유한다.

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

const MAX_FRAME_BYTES: usize = 256 * 1024 * 1024;

pub struct Transport {
    stream: UnixStream,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "type")]
    pub kind: String,
    pub request_id: Option<String>,
    pub source_client_id: Option<String>,
    pub target_client_ids: Option<Vec<String>>,
    pub version: Option<u64>,
    pub method: Option<String>,
    pub params: Option<Value>,
    pub result: Option<Value>,
    pub result_type: Option<String>,
    pub error: Option<String>,
}

impl Transport {
    pub async fn connect() -> Result<Self> {
        let path = Self::endpoint();
        Self::validate(&path)?;
        let stream = UnixStream::connect(&path).await.with_context(|| {
            format!(
                "Failed to connect to Codex Desktop IPC at {}.",
                path.display()
            )
        })?;
        Ok(Self { stream })
    }

    pub async fn send(&mut self, message: &Value) -> Result<()> {
        let payload = serde_json::to_vec(message)?;
        if payload.len() > MAX_FRAME_BYTES {
            bail!("Codex Desktop IPC payload is too large.");
        }
        let length =
            u32::try_from(payload.len()).context("Desktop IPC payload length exceeds u32.")?;
        self.stream.write_u32_le(length).await?;
        self.stream.write_all(&payload).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Message> {
        loop {
            let message = self.frame().await?;
            if message.kind == "client-discovery-request" {
                self.send(&json!({
                    "type":"client-discovery-response", "requestId":message.request_id,
                    "response":{"canHandle":false}
                }))
                .await?;
                continue;
            }
            if message.kind == "request" {
                self.send(&json!({
                    "type":"response", "requestId":message.request_id,
                    "resultType":"error", "error":"no-handler-for-request"
                }))
                .await?;
                continue;
            }
            return Ok(message);
        }
    }

    async fn frame(&mut self) -> Result<Message> {
        let length = self
            .stream
            .read_u32_le()
            .await
            .context("Failed to read a Codex Desktop IPC frame header.")?;
        let length =
            usize::try_from(length).context("Desktop IPC frame length does not fit usize.")?;
        if length > MAX_FRAME_BYTES {
            bail!("Codex Desktop IPC frame is too large.");
        }
        let mut payload = vec![0; length];
        self.stream
            .read_exact(&mut payload)
            .await
            .context("Failed to read a Codex Desktop IPC frame.")?;
        serde_json::from_slice(&payload).context("Invalid Codex Desktop IPC JSON.")
    }

    fn endpoint() -> PathBuf {
        if let Some(path) = env::var_os("CODEX_APP_IPC_PATH") {
            return PathBuf::from(path);
        }
        let home = env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".codex")
            });
        home.join("ipc/ipc.sock")
    }

    fn validate(path: &Path) -> Result<()> {
        if env::var_os("CODEX_APP_IPC_PATH").is_some() {
            return Ok(());
        }
        let directory = path
            .parent()
            .ok_or_else(|| anyhow!("Codex Desktop IPC path has no parent directory."))?;
        let unavailable = || {
            anyhow!(
                "Codex Desktop IPC is unavailable at {}. Open Codex Desktop and keep the target task visible.",
                path.display()
            )
        };
        let directory = std::fs::symlink_metadata(directory).map_err(|_| unavailable())?;
        let socket = std::fs::symlink_metadata(path).map_err(|_| unavailable())?;
        let uid = rustix::process::getuid().as_raw();
        if !directory.is_dir()
            || directory.uid() != uid
            || directory.mode() & 0o022 != 0
            || !socket.file_type().is_socket()
            || socket.uid() != uid
        {
            bail!(
                "Refusing insecure Codex Desktop IPC endpoint {}.",
                path.display()
            );
        }
        Ok(())
    }
}
