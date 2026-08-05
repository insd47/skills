//! snapshot과 revision-contiguous patch만으로 conversation state 복제본을 유지한다.

use super::Settlement;
use super::state::{DesktopPatch, PatchOperation, PathSegment, StateChange};
use anyhow::{Context, Result, bail};
use json_patch::Patch;
use serde_json::{Value, json};

const TERMINAL_STATUSES: &[&str] = &["completed", "interrupted", "failed", "cancelled"];

pub struct Mirror {
    state: Value,
    revision: u64,
    owner: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MirrorUpdate {
    Applied,
    NeedsResync,
}

impl Mirror {
    pub fn from_snapshot(source: String, change: StateChange) -> Option<Self> {
        let StateChange::Snapshot {
            revision,
            conversation_state,
        } = change
        else {
            return None;
        };
        Some(Self {
            state: conversation_state,
            revision,
            owner: source,
        })
    }

    /// change를 반영하거나, owner·revision 연속성이 깨졌으면 resync를 요구한다.
    pub fn apply(&mut self, source: String, change: StateChange) -> Result<MirrorUpdate> {
        match change {
            StateChange::Snapshot {
                revision,
                conversation_state,
            } => {
                self.state = conversation_state;
                self.revision = revision;
                self.owner = source;
            }

            StateChange::Patches {
                base_revision,
                revision,
                patches,
            } => {
                if source != self.owner
                    || base_revision != self.revision
                    || revision <= base_revision
                {
                    return Ok(MirrorUpdate::NeedsResync);
                }
                self.apply_patches(patches)?;
                self.revision = revision;
            }
        }
        Ok(MirrorUpdate::Applied)
    }

    pub fn settlement(&self, turn_id: &str) -> Option<Settlement> {
        let turn = self.turn(turn_id)?;
        let status = turn.get("status")?.as_str()?;

        if !TERMINAL_STATUSES.contains(&status) {
            return None;
        }

        Some(Settlement {
            status: status.into(),
            result: Self::final_message(turn),
        })
    }

    fn turn(&self, turn_id: &str) -> Option<&Value> {
        self.state
            .pointer("/turnHistory/history/entitiesByKey")?
            .as_object()?
            .values()
            .find(|turn| turn.get("turnId").and_then(Value::as_str) == Some(turn_id))
            .or_else(|| {
                self.state.get("turns")?.as_array()?.iter().find(|turn| {
                    turn.get("turnId")
                        .or_else(|| turn.get("id"))
                        .and_then(Value::as_str)
                        == Some(turn_id)
                })
            })
    }

    fn final_message(turn: &Value) -> String {
        turn.get("items")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .rev()
            .find(|item| item.get("type").and_then(Value::as_str) == Some("agentMessage"))
            .and_then(|item| item.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_owned()
    }

    fn apply_patches(&mut self, patches: Vec<DesktopPatch>) -> Result<()> {
        let value = patches
            .into_iter()
            .map(|patch| -> Result<Value> {
                let path = Self::pointer(&patch.path);
                Ok(match patch.op {
                    PatchOperation::Add => json!({
                        "op":"add", "path":path,
                        "value":patch.value.context("Desktop add patch has no value.")?
                    }),
                    PatchOperation::Replace => json!({
                        "op":"replace", "path":path,
                        "value":patch.value.context("Desktop replace patch has no value.")?
                    }),
                    PatchOperation::Remove => {
                        if patch.path.is_empty() {
                            bail!("Desktop state cannot be removed by a root patch.");
                        }
                        json!({"op":"remove", "path":path})
                    }
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let patch: Patch = serde_json::from_value(Value::Array(value))?;
        json_patch::patch(&mut self.state, &patch).context("Failed to apply a Desktop state patch.")
    }

    fn pointer(path: &[PathSegment]) -> String {
        path.iter()
            .map(|segment| match segment {
                PathSegment::Key(key) => key.replace('~', "~0").replace('/', "~1"),
                PathSegment::Index(index) => index.to_string(),
            })
            .fold(String::new(), |mut path, segment| {
                path.push('/');
                path.push_str(&segment);
                path
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_array_paths_apply_as_rfc_6902() {
        let mut mirror = Mirror::from_snapshot(
            "desktop".into(),
            StateChange::Snapshot {
                revision: 1,
                conversation_state: json!({"turns":[{"status":"running"}]}),
            },
        )
        .unwrap();
        mirror
            .apply_patches(vec![DesktopPatch {
                op: PatchOperation::Replace,
                path: vec![
                    PathSegment::Key("turns".into()),
                    PathSegment::Index(0),
                    PathSegment::Key("status".into()),
                ],
                value: Some(json!("completed")),
            }])
            .unwrap();
        assert_eq!(
            mirror.state.pointer("/turns/0/status"),
            Some(&json!("completed"))
        );
    }

    #[test]
    fn revision_gap_requires_resync_without_mutating_the_mirror() {
        let state = json!({"turns":[{"status":"running"}]});
        let mut mirror = Mirror::from_snapshot(
            "desktop".into(),
            StateChange::Snapshot {
                revision: 4,
                conversation_state: state.clone(),
            },
        )
        .unwrap();

        let result = mirror
            .apply(
                "desktop".into(),
                StateChange::Patches {
                    base_revision: 3,
                    revision: 5,
                    patches: Vec::new(),
                },
            )
            .unwrap();

        assert_eq!(result, MirrorUpdate::NeedsResync);
        assert_eq!(mirror.revision, 4);
        assert_eq!(mirror.state, state);
    }
}
