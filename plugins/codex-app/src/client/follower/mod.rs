//! Desktop task-state broadcast를 revision-contiguous mirror로 유지하며 gap에서는 snapshot을 다시 요구한다.

mod mirror;
mod state;

use super::Client;
use anyhow::{Context, Result};
use mirror::{Mirror, MirrorUpdate};
use serde_json::json;
use state::StateChanged;
use std::time::Duration;

const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(10);

/// Desktop task-state stream의 mirror를 소유한다.
pub struct Follower {
    thread_id: String,
    mirror: Mirror,
}

/// 종결 turn의 status와 final message다.
pub struct Settlement {
    pub status: String,
    pub result: String,
}

impl Follower {
    /// follower 등록 후 첫 snapshot으로 mirror를 만든다.
    pub async fn start(client: &mut Client, thread_id: String) -> Result<Self> {
        let mirror = Self::resync(client, &thread_id).await?;
        Ok(Self { thread_id, mirror })
    }

    /// task-state follower 등록을 해제한다.
    pub async fn close(&self, client: &mut Client) {
        let _ = Self::set_following(client, &self.thread_id, false).await;
    }

    /// 다음 state broadcast를 반영하고 revision gap이면 snapshot으로 복구한다.
    pub async fn next(&mut self, client: &mut Client) -> Result<()> {
        let message = client.next_broadcast().await?;
        let Some((source, change)) = StateChanged::from_broadcast(message, &self.thread_id)? else {
            return Ok(());
        };
        if self.mirror.apply(source, change)? == MirrorUpdate::NeedsResync {
            self.mirror = Self::resync(client, &self.thread_id).await?;
        }
        Ok(())
    }

    /// 현재 mirror에서 turn의 종결 여부를 읽는다.
    pub fn settlement(&self, turn_id: &str) -> Option<Settlement> {
        self.mirror.settlement(turn_id)
    }

    async fn resync(client: &mut Client, thread_id: &str) -> Result<Mirror> {
        Self::set_following(client, thread_id, true).await?;

        tokio::time::timeout(SNAPSHOT_TIMEOUT, async {
            loop {
                let message = client.next_broadcast().await?;

                let Some((source, change)) = StateChanged::from_broadcast(message, thread_id)?
                else {
                    continue;
                };

                if let Some(mirror) = Mirror::from_snapshot(source, change) {
                    return Ok(mirror);
                }
            }
        })
        .await
        .with_context(|| {
            format!(
                "Codex Desktop did not expose task {thread_id} as an owned IPC stream within {}ms.",
                SNAPSHOT_TIMEOUT.as_millis()
            )
        })?
    }

    async fn set_following(client: &mut Client, thread_id: &str, following: bool) -> Result<()> {
        client
            .broadcast(
                "thread-stream-following-changed",
                json!({
                    "conversationId":thread_id, "hostId":state::HOST_ID, "following":following
                }),
            )
            .await
    }
}
