//! turn의 시작·제어를 담당하는 facade다. turn의 수명은 이 MCP session 안의 task가 소유한다.

mod registry;
mod task;

use crate::client::lookup;
use crate::outbox::Outbox;
use crate::protocol::{Accepted, Started, SteerParams, TurnKind, TurnReference};
use crate::validate::{required, target_thread};
use anyhow::{Result, anyhow};
use registry::Registry;
use std::env;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct Turns {
    cwd: PathBuf,
    outbox: Outbox,
    registry: Registry,
}

impl Turns {
    pub fn new(cwd: PathBuf, outbox: Outbox) -> Self {
        Self {
            cwd,
            outbox,
            registry: Registry::default(),
        }
    }

    pub async fn ask(&self, prompt: String) -> Result<Started> {
        let prompt = required(prompt, "prompt")?;
        let thread_id = self.resolve_ask_thread().await?;

        if let Some(handle) = self.registry.find(&thread_id).await {
            let reference = handle.steer(None, prompt).await?;

            return Ok(Started {
                thread_id: reference.thread_id,
                turn_id: reference.turn_id,
                kind: TurnKind::Ask,
                accepted: Accepted::Steered,
            });
        }

        let (commands, receiver) = mpsc::unbounded_channel();
        let (started_tx, started_rx) = oneshot::channel();
        self.registry.register(&thread_id, &commands, None).await;

        tokio::spawn(
            task::Task {
                cwd: self.cwd.clone(),
                outbox: self.outbox.clone(),
                registry: self.registry.clone(),
                thread_id,
                prompt,
                commands,
                receiver,
                started: Some(started_tx),
            }
            .run(),
        );

        let reference = started_rx
            .await
            .map_err(|_| anyhow!("Codex ask task stopped before dispatch."))??;

        Ok(Started {
            thread_id: reference.thread_id,
            turn_id: reference.turn_id,
            kind: TurnKind::Ask,
            accepted: Accepted::Started,
        })
    }

    pub async fn status(&self, reference: TurnReference) -> Result<TurnReference> {
        self.registry.expect(&reference).await?;
        Ok(reference)
    }

    pub async fn steer(&self, params: SteerParams) -> Result<TurnReference> {
        let prompt = required(params.prompt, "prompt")?;

        let reference = TurnReference {
            thread_id: params.thread_id,
            turn_id: params.turn_id,
        };

        let handle = self.registry.expect(&reference).await?;
        handle.steer(Some(reference.turn_id), prompt).await
    }

    pub async fn interrupt(&self, reference: TurnReference) -> Result<TurnReference> {
        let handle = self.registry.expect(&reference).await?;
        handle.interrupt(reference.turn_id).await
    }

    async fn resolve_ask_thread(&self) -> Result<String> {
        let explicit = env::var("CODEX_APP_THREAD_ID").ok();

        if explicit
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            return target_thread(explicit.as_deref(), None, &[], &self.cwd.to_string_lossy());
        }

        let listed = lookup::threads(&self.cwd).await?;

        target_thread(
            None,
            env::var("CODEX_APP_TITLE").ok().as_deref(),
            &listed,
            &self.cwd.to_string_lossy(),
        )
    }
}
