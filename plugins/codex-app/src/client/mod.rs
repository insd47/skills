//! Desktop IPC의 handshake와 request correlation을 소유하며 framing은 transport에 맡긴다.

use anyhow::{Context, Result, bail};
pub use follower::Follower;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use transport::{Message, Transport};
use uuid::Uuid;

mod follower;
pub mod lookup;
mod transport;

const INITIAL_CLIENT_ID: &str = "initializing-client";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const START_VERSION: u8 = 2;
const STEER_VERSION: u8 = 1;
const INTERRUPT_VERSION: u8 = 4;

/// 초기화가 끝난 Codex Desktop IPC client다.
pub struct Client {
    transport: Transport,
    client_id: String,
    broadcasts: VecDeque<Message>,
}

impl Client {
    pub async fn connect() -> Result<Self> {
        let mut client = Self {
            transport: Transport::connect().await?,
            client_id: INITIAL_CLIENT_ID.into(),
            broadcasts: VecDeque::new(),
        };

        let response = client
            .request(
                "initialize",
                0,
                json!({"clientType": "claude-code-codex-app-plugin"}),
            )
            .await?;

        let initialized: Initialize = serde_json::from_value(response)?;
        client.client_id = initialized.client_id;

        Ok(client)
    }

    pub async fn start(&mut self, thread_id: &str, prompt: &str, cwd: &Path) -> Result<String> {
        let response = self
            .request(
                "thread-follower-start-turn",
                START_VERSION,
                json!({
                    "conversationId":thread_id,
                    "turnStart":{
                        "request":{"threadId":thread_id,"input":Self::input(prompt),"cwd":cwd},
                        "context":{}
                    }
                }),
            )
            .await?;

        let started: TurnStarted = serde_json::from_value(response)?;
        started.turn_id().context("Codex did not return a turn id.")
    }

    pub async fn steer(
        &mut self,
        thread_id: &str,
        prompt: &str,
        cwd: &Path,
    ) -> Result<Option<String>> {
        let restore = json!({
            "id":Uuid::new_v4().to_string(), "text":prompt,
            "context":{"prompt":prompt,"addedFiles":[],"fileAttachments":[],"ideContext":null,
                "imageAttachments":[],"commentAttachments":[],"workspaceRoots":[cwd]},
            "cwd":cwd,
            "createdAt":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
        });

        let response = self
            .request(
                "thread-follower-steer-turn",
                STEER_VERSION,
                json!({
                    "conversationId":thread_id, "clientUserMessageId":restore["id"],
                    "input":Self::input(prompt), "serviceTier":null, "attachments":[],
                    "additionalContext":null, "restoreMessage":restore
                }),
            )
            .await?;

        let started: TurnStarted = serde_json::from_value(response)?;
        Ok(started.turn_id())
    }

    pub async fn interrupt(&mut self, thread_id: &str, turn_id: &str) -> Result<()> {
        self.request(
            "thread-follower-interrupt-turn",
            INTERRUPT_VERSION,
            json!({"conversationId":thread_id,"mode":"user","expectedTurnId":turn_id}),
        )
        .await?;

        Ok(())
    }

    async fn request(&mut self, method: &str, version: u8, params: Value) -> Result<Value> {
        if self.client_id == INITIAL_CLIENT_ID && method != "initialize" {
            bail!("Codex Desktop IPC is not initialized.");
        }

        let request_id = Uuid::new_v4().to_string();
        self.transport
            .send(&json!({
                "type":"request", "requestId":request_id, "sourceClientId":self.client_id,
                "version":version, "method":method, "params":params,
                "timeoutMs":REQUEST_TIMEOUT.as_millis()
            }))
            .await?;

        tokio::time::timeout(REQUEST_TIMEOUT, async {
            loop {
                let message = self.transport.receive().await?;

                if message.kind == "broadcast" {
                    self.broadcasts.push_back(message);
                    continue;
                }

                if message.kind != "response" || message.request_id.as_deref() != Some(&request_id)
                {
                    continue;
                }

                if message.result_type.as_deref() == Some("error") {
                    bail!(
                        "Codex Desktop IPC {method} failed: {}",
                        message.error.as_deref().unwrap_or("unknown error")
                    );
                }

                return Ok(message.result.unwrap_or(Value::Object(Default::default())));
            }
        })
        .await
        .with_context(|| format!("Codex Desktop IPC {method} timed out."))?
    }

    async fn broadcast(&mut self, method: &str, params: Value) -> Result<()> {
        self.transport
            .send(&json!({
                "type":"broadcast", "method":method, "sourceClientId":self.client_id,
                "version":1, "params":params
            }))
            .await
    }

    async fn next_broadcast(&mut self) -> Result<Message> {
        while let Some(message) = self.broadcasts.pop_front() {
            if self.is_target(&message) {
                return Ok(message);
            }
        }

        loop {
            let message = self.transport.receive().await?;
            if message.kind == "broadcast" && self.is_target(&message) {
                return Ok(message);
            }
        }
    }

    fn is_target(&self, message: &Message) -> bool {
        message
            .target_client_ids
            .as_ref()
            .is_none_or(|targets| targets.contains(&self.client_id))
    }

    fn input(prompt: &str) -> Value {
        json!([{"type":"text","text":prompt,"text_elements":[]}])
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Initialize {
    client_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TurnStarted {
    result: Option<Box<TurnStarted>>,
    turn_id: Option<String>,
    turn: Option<TurnId>,
}

impl TurnStarted {
    /// Desktop follower 응답은 method에 따라 turn을 한 겹 더 감싼 result 안에 싣기도 한다.
    fn turn_id(self) -> Option<String> {
        self.turn_id
            .or_else(|| self.turn.map(|turn| turn.id))
            .or_else(|| self.result.and_then(|inner| inner.turn_id()))
    }
}

#[derive(Debug, Deserialize)]
struct TurnId {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn text_input_uses_the_codex_shape() {
        assert_eq!(
            Client::input("hello"),
            json!([{"type":"text","text":"hello","text_elements":[]}])
        );
    }

    #[test]
    fn turn_id_reads_only_the_known_response_fields() {
        let direct: TurnStarted = serde_json::from_value(json!({"turnId":"direct"})).unwrap();
        assert_eq!(direct.turn_id().as_deref(), Some("direct"));

        let nested: TurnStarted = serde_json::from_value(json!({"turn":{"id":"nested"}})).unwrap();
        assert_eq!(nested.turn_id().as_deref(), Some("nested"));

        let hidden: TurnStarted =
            serde_json::from_value(json!({"other":{"turnId":"hidden"}})).unwrap();
        assert_eq!(hidden.turn_id(), None);
    }

    #[test]
    fn turn_id_unwraps_the_follower_result_envelope() {
        let wrapped: TurnStarted =
            serde_json::from_value(json!({"result":{"turn":{"id":"wrapped"}}})).unwrap();
        assert_eq!(wrapped.turn_id().as_deref(), Some("wrapped"));
    }
}
