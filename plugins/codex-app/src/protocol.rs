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

/// blocking `ask`의 단일 반환 형태. MCP outputSchema는 최상위 object를 요구하므로
/// steer 즉시 반환과 turn 종결을 `status`로 구분하고 union schema를 만들지 않는다.
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[schemars(rename_all = "camelCase")]
pub struct AskResult {
    pub thread_id: String,
    pub turn_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

impl AskResult {
    /// 실행 중인 turn에 prompt를 전달한 즉시 반환할 결과를 만든다.
    pub fn steered(reference: TurnReference) -> Self {
        Self {
            thread_id: reference.thread_id,
            turn_id: reference.turn_id,
            status: "steered".into(),
            result: None,
        }
    }
}

impl From<Completion> for AskResult {
    fn from(completion: Completion) -> Self {
        Self {
            thread_id: completion.thread_id,
            turn_id: completion.turn_id,
            status: completion.status,
            result: Some(completion.result),
        }
    }
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[schemars(rename_all = "camelCase")]
pub struct Completion {
    pub thread_id: String,
    pub turn_id: String,
    pub status: String,
    pub result: String,
}

impl Completion {
    /// 종결된 turn의 반환 결과를 만든다. 빈 결과는 안내 문구로 대체한다.
    pub fn settled(reference: TurnReference, status: impl Into<String>, result: String) -> Self {
        Self {
            thread_id: reference.thread_id,
            turn_id: reference.turn_id,
            status: status.into(),
            result: if result.is_empty() {
                "Codex completed without a final agent message.".into()
            } else {
                result
            },
        }
    }

    /// 실패한 turn의 반환 결과를 만든다.
    pub fn failed(reference: &TurnReference, error: &Error) -> Self {
        Self {
            thread_id: reference.thread_id.clone(),
            turn_id: reference.turn_id.clone(),
            status: "failed".into(),
            result: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ask_result_schema_stays_a_single_object() {
        let value = serde_json::to_value(schemars::schema_for!(AskResult)).unwrap();
        assert_eq!(value["type"], "object");
    }

    #[test]
    fn steered_ask_result_omits_the_result_field() {
        let value = serde_json::to_value(AskResult::steered(TurnReference {
            thread_id: "thread".into(),
            turn_id: "turn".into(),
        }))
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({"threadId":"thread", "turnId":"turn", "status":"steered"})
        );
    }
}
