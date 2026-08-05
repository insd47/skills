//! steer와 interrupt는 여기 등록된 command 채널로만 전달된다. 다른 세션의 turn은 보이지 않는다.

use crate::protocol::TurnReference;
use anyhow::{Context, Result, anyhow, bail};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};

/// MCP session이 소유한 활성 turn 대장이다.
#[derive(Clone, Default)]
pub struct Registry {
    active: Arc<Mutex<HashMap<String, Entry>>>,
}

#[derive(Clone)]
struct Entry {
    turn_id: Option<String>,
    commands: mpsc::UnboundedSender<Command>,
}

/// 실행 중인 turn task로 보내는 제어 명령이다.
pub enum Command {
    Steer {
        expected: Option<String>,
        prompt: String,
        reply: oneshot::Sender<Result<TurnReference>>,
    },
    Interrupt {
        expected: String,
        reply: oneshot::Sender<Result<TurnReference>>,
    },
}

/// 활성 turn 하나에 명령을 전달하는 손잡이다.
pub struct Handle {
    commands: mpsc::UnboundedSender<Command>,
}

impl Registry {
    /// thread에 활성 entry가 있으면 turn 일치 여부와 무관하게 손잡이를 돌려준다.
    pub async fn find(&self, thread_id: &str) -> Option<Handle> {
        self.active.lock().await.get(thread_id).map(|entry| Handle {
            commands: entry.commands.clone(),
        })
    }

    /// reference와 정확히 일치하는 활성 turn의 손잡이를 돌려주고, 아니면 오류를 낸다.
    pub async fn expect(&self, reference: &TurnReference) -> Result<Handle> {
        let entry = self.active.lock().await.get(&reference.thread_id).cloned();

        match entry {
            Some(entry) if entry.turn_id.as_deref() == Some(&reference.turn_id) => Ok(Handle {
                commands: entry.commands,
            }),
            _ => bail!(
                "Codex turn {} in thread {} is not active in this MCP session.",
                reference.turn_id,
                reference.thread_id
            ),
        }
    }

    pub async fn register(
        &self,
        thread_id: &str,
        commands: &mpsc::UnboundedSender<Command>,
        turn_id: Option<String>,
    ) {
        self.active.lock().await.insert(
            thread_id.to_owned(),
            Entry {
                turn_id,
                commands: commands.clone(),
            },
        );
    }

    /// 같은 task의 entry일 때만 현재 turn을 갱신한다.
    pub async fn set_turn(
        &self,
        thread_id: &str,
        commands: &mpsc::UnboundedSender<Command>,
        turn_id: String,
    ) {
        let mut active = self.active.lock().await;

        if let Some(entry) = active
            .get_mut(thread_id)
            .filter(|entry| entry.commands.same_channel(commands))
        {
            entry.turn_id = Some(turn_id);
        }
    }

    /// 같은 task의 entry일 때만 제거한다. 후속 task가 등록한 entry는 보존된다.
    pub async fn remove(&self, thread_id: &str, commands: &mpsc::UnboundedSender<Command>) {
        let mut active = self.active.lock().await;

        if active
            .get(thread_id)
            .is_some_and(|entry| entry.commands.same_channel(commands))
        {
            active.remove(thread_id);
        }
    }
}

impl Handle {
    pub async fn steer(self, expected: Option<String>, prompt: String) -> Result<TurnReference> {
        let (reply, result) = oneshot::channel();

        self.commands
            .send(Command::Steer {
                expected,
                prompt,
                reply,
            })
            .map_err(|_| anyhow!("Codex turn command channel is closed."))?;

        result
            .await
            .context("Codex turn stopped before steer completed.")?
    }

    pub async fn interrupt(self, expected: String) -> Result<TurnReference> {
        let (reply, result) = oneshot::channel();

        self.commands
            .send(Command::Interrupt { expected, reply })
            .map_err(|_| anyhow!("Codex turn command channel is closed."))?;

        result
            .await
            .context("Codex turn stopped before interrupt completed.")?
    }
}
