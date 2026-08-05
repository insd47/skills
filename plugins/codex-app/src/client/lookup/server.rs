//! codex app-server 자식 프로세스와의 JSON-lines RPC 상관관계를 소유한다.
//! 이 wire는 "jsonrpc" 필드가 없는 의도적 비표준이라 표준 JSON-RPC crate로 대체할 수 없다.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::env;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

const CLOSE_TIMEOUT: Duration = Duration::from_millis(1_500);

pub struct AppServer {
    child: Child,
    input: ChildStdin,
    output: Lines<BufReader<ChildStdout>>,
    next_id: u64,
}

impl AppServer {
    pub async fn connect(cwd: &Path) -> Result<Self> {
        let binary = env::var_os("CODEX_APP_BIN")
            .or_else(|| env::var_os("CODEX_CLI_PATH"))
            .unwrap_or_else(|| "codex".into());

        let mut child = Command::new(binary)
            .arg("app-server")
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start codex app-server.")?;

        let input = child
            .stdin
            .take()
            .context("codex app-server did not expose stdin.")?;

        let stdout = child
            .stdout
            .take()
            .context("codex app-server did not expose stdout.")?;

        let mut server = Self {
            child,
            input,
            output: BufReader::new(stdout).lines(),
            next_id: 1,
        };

        let _: Value = server
            .request(
                "initialize",
                json!({
                    "clientInfo": {
                        "name":"claude_code_codex_app", "title":"Claude Code Codex App Plugin",
                        "version":env!("CARGO_PKG_VERSION")
                    },
                    "capabilities":{"experimentalApi":false,"requestAttestation":false}
                }),
            )
            .await?;

        server.notify("initialized", json!({})).await?;
        Ok(server)
    }

    pub async fn close(mut self) {
        let _ = self.input.shutdown().await;

        if tokio::time::timeout(CLOSE_TIMEOUT, self.child.wait())
            .await
            .is_err()
        {
            let _ = self.child.kill().await;
        }
    }

    pub async fn request<T: DeserializeOwned>(&mut self, method: &str, params: Value) -> Result<T> {
        let id = self.next_id;

        self.next_id += 1;
        self.send(&json!({"id":id,"method":method,"params":params}))
            .await?;

        loop {
            let envelope = self.read().await?;

            if let Some(incoming_method) = envelope.method {
                if let Some(incoming_id) = envelope.id {
                    self.method_not_found(incoming_id, &incoming_method).await?;
                }

                continue;
            }

            if envelope.id != Some(id) {
                continue;
            }

            if let Some(error) = envelope.error {
                bail!("{}", error.message);
            }

            return serde_json::from_value(envelope.result.unwrap_or_else(|| json!({})))
                .with_context(|| format!("Invalid {method} response."));
        }
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.send(&json!({"method":method,"params":params})).await
    }

    async fn method_not_found(&mut self, id: u64, method: &str) -> Result<()> {
        self.send(&json!({"id":id,"error":{"code":-32601,
            "message":format!("codex-app title lookup cannot answer {method}.")}}))
            .await
    }

    async fn send(&mut self, value: &Value) -> Result<()> {
        let mut bytes = serde_json::to_vec(value)?;
        bytes.push(b'\n');

        self.input
            .write_all(&bytes)
            .await
            .context("Failed to write an app-server request.")?;

        self.input
            .flush()
            .await
            .context("Failed to flush an app-server request.")
    }

    async fn read(&mut self) -> Result<RpcEnvelope> {
        let line = self
            .output
            .next_line()
            .await
            .context("Failed to read app-server stdout.")?
            .context("codex app-server exited before responding.")?;

        serde_json::from_str(&line).context("Invalid app-server JSON.")
    }
}

#[derive(Debug, Deserialize)]
struct RpcEnvelope {
    id: Option<u64>,
    method: Option<String>,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}
