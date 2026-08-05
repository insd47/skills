//! thread-stream-state-changed broadcast의 wire 계약과 protocol version gate를 소유한다.

use crate::client::Message;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;

pub const HOST_ID: &str = "local";
const STATE_VERSION: u64 = 11;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChanged {
    conversation_id: String,
    host_id: String,
    change: StateChange,
}

impl StateChanged {
    /// 대상 thread의 state broadcast만 (owner, change)로 해석하고 protocol version을 검증한다.
    pub fn from_broadcast(
        message: Message,
        thread_id: &str,
    ) -> Result<Option<(String, StateChange)>> {
        if message.method.as_deref() != Some("thread-stream-state-changed") {
            return Ok(None);
        }

        let params = message
            .params
            .context("Desktop state broadcast has no params.")?;

        let event: Self = serde_json::from_value(params)?;

        if event.conversation_id != thread_id || event.host_id != HOST_ID {
            return Ok(None);
        }

        if message.version != Some(STATE_VERSION) {
            bail!(
                "Unsupported Codex Desktop thread state protocol version {}; expected {STATE_VERSION}.",
                message
                    .version
                    .map_or_else(|| "null".into(), |version| version.to_string())
            );
        }

        let source_client_id = message
            .source_client_id
            .context("Desktop state broadcast has no owner.")?;

        Ok(Some((source_client_id, event.change)))
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StateChange {
    Snapshot {
        revision: u64,
        #[serde(rename = "conversationState")]
        conversation_state: Value,
    },
    Patches {
        #[serde(rename = "baseRevision")]
        base_revision: u64,
        revision: u64,
        patches: Vec<DesktopPatch>,
    },
}

#[derive(Debug, Deserialize)]
pub struct DesktopPatch {
    pub op: PatchOperation,
    pub path: Vec<PathSegment>,
    #[serde(default)]
    pub value: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchOperation {
    Add,
    Replace,
    Remove,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn protocol_version_mismatch_is_never_accepted() {
        let message = Message {
            kind: "broadcast".into(),
            request_id: None,
            source_client_id: Some("desktop".into()),
            target_client_ids: None,
            version: Some(12),
            method: Some("thread-stream-state-changed".into()),
            params: Some(json!({"conversationId":"thread","hostId":"local","change":{
                "type":"snapshot","revision":1,"conversationState":{}
            }})),
            result: None,
            result_type: None,
            error: None,
        };
        let error = StateChanged::from_broadcast(message, "thread").unwrap_err();
        assert!(error.to_string().contains("expected 11"));
    }
}
