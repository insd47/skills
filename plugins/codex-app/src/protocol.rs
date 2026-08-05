use anyhow::Error;
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AskParams {
    #[schemars(length(min = 1))]
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[schemars(rename_all = "camelCase")]
pub struct TurnReference {
    #[schemars(length(min = 1))]
    pub thread_id: String,
    #[schemars(length(min = 1))]
    pub turn_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename_all = "camelCase")]
pub struct SteerParams {
    #[schemars(length(min = 1))]
    pub thread_id: String,
    #[schemars(length(min = 1))]
    pub turn_id: String,
    #[schemars(length(min = 1))]
    pub prompt: String,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[schemars(rename_all = "camelCase")]
pub struct Started {
    pub thread_id: String,
    pub turn_id: String,
    pub kind: TurnKind,
    pub accepted: Accepted,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnKind {
    Ask,
}

#[derive(Clone, Copy, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Accepted {
    Started,
    Steered,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub thread_id: String,
    pub turn_id: String,
    pub kind: TurnKind,
    pub status: String,
    pub result: String,
}

impl Completion {
    /// 종결된 turn의 완료 기록을 만든다. 빈 결과는 안내 문구로 대체한다.
    pub fn settled(reference: TurnReference, status: impl Into<String>, result: String) -> Self {
        Self {
            thread_id: reference.thread_id,
            turn_id: reference.turn_id,
            kind: TurnKind::Ask,
            status: status.into(),
            result: if result.is_empty() {
                "Codex completed without a final agent message.".into()
            } else {
                result
            },
        }
    }

    /// 실패한 turn의 외부 완료 기록을 만든다.
    pub fn failed(reference: &TurnReference, error: &Error) -> Self {
        Self {
            thread_id: reference.thread_id.clone(),
            turn_id: reference.turn_id.clone(),
            kind: TurnKind::Ask,
            status: "failed".into(),
            result: error.to_string(),
        }
    }
}
