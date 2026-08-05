//! persistent Desktop task의 turn 하나를 소유한다. 완료 판정은 Desktop follower stream만 사용한다.

use crate::client::{Client, Follower};
use crate::outbox::Outbox;
use crate::protocol::{Completion, TurnReference};
use crate::turns::registry::{Command, Registry};
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};

pub struct Task {
    pub cwd: PathBuf,
    pub outbox: Outbox,
    pub registry: Registry,
    pub thread_id: String,
    pub prompt: String,
    pub commands: mpsc::UnboundedSender<Command>,
    pub receiver: mpsc::UnboundedReceiver<Command>,
    pub started: Option<oneshot::Sender<Result<TurnReference>>>,
}

impl Task {
    pub async fn run(mut self) {
        let mut reference = None;

        if let Err(error) = self.try_run(&mut reference).await {
            if let Some(sender) = self.started.take() {
                let _ = sender.send(Err(anyhow!("{error:#}")));
            }

            if let Some(reference) = &reference
                && let Err(e) = self.outbox.write(&Completion::failed(reference, &error))
            {
                tracing::error!(error = format!("{e:#}"), "Failed to write completion");
            }

            tracing::error!(error = format!("{error:#}"), "Codex ask task failed");
        }

        self.registry.remove(&self.thread_id, &self.commands).await;
    }

    async fn try_run(&mut self, reference: &mut Option<TurnReference>) -> Result<()> {
        let mut client = Client::connect().await?;
        let mut follower = Follower::start(&mut client, self.thread_id.clone()).await?;

        let result = self.drive(&mut client, &mut follower, reference).await;
        follower.close(&mut client).await;

        result
    }

    async fn drive(
        &mut self,
        client: &mut Client,
        follower: &mut Follower,
        reference: &mut Option<TurnReference>,
    ) -> Result<()> {
        let mut turn_id = client
            .start(&self.thread_id, &self.prompt, &self.cwd)
            .await?;

        let mut current = TurnReference {
            thread_id: self.thread_id.clone(),
            turn_id: turn_id.clone(),
        };

        *reference = Some(current.clone());

        self.registry
            .set_turn(&self.thread_id, &self.commands, turn_id.clone())
            .await;

        if let Some(sender) = self.started.take() {
            let _ = sender.send(Ok(current.clone()));
        }

        loop {
            if let Some(settlement) = follower.settlement(&turn_id) {
                let status = if settlement.status == "cancelled" {
                    "interrupted"
                } else {
                    &settlement.status
                };

                self.outbox
                    .write(&Completion::settled(current, status, settlement.result))?;

                return Ok(());
            }

            tokio::select! {
                command = self.receiver.recv() => if let Some(command) = command {
                    turn_id = self.handle(client, &turn_id, command).await?;
                    self.registry.set_turn(&self.thread_id, &self.commands, turn_id.clone()).await;
                    current.turn_id.clone_from(&turn_id);
                    *reference = Some(current.clone());
                },
                update = follower.next(client) => update?,
            }
        }
    }

    async fn handle(&self, client: &mut Client, turn_id: &str, command: Command) -> Result<String> {
        match command {
            Command::Steer {
                expected,
                prompt,
                reply,
            } => {
                if let Some(expected) = expected.as_deref().filter(|expected| *expected != turn_id)
                {
                    let error = anyhow!("Codex turn {expected} is no longer active.");
                    let _ = reply.send(Err(error));
                    return Ok(turn_id.to_owned());
                }

                match client.steer(&self.thread_id, &prompt, &self.cwd).await {
                    Ok(response) => {
                        let next = response.unwrap_or_else(|| turn_id.to_owned());

                        let _ = reply.send(Ok(TurnReference {
                            thread_id: self.thread_id.clone(),
                            turn_id: next.clone(),
                        }));

                        Ok(next)
                    }
                    Err(error) => {
                        let _ = reply.send(Err(anyhow!("{error:#}")));
                        Ok(turn_id.to_owned())
                    }
                }
            }
            Command::Interrupt { expected, reply } => {
                if expected != turn_id {
                    let _ = reply.send(Err(anyhow!("Codex turn {expected} is no longer active.")));
                    return Ok(turn_id.to_owned());
                }

                let result = client.interrupt(&self.thread_id, turn_id).await;
                let response = result.map(|_| TurnReference {
                    thread_id: self.thread_id.clone(),
                    turn_id: turn_id.to_owned(),
                });

                let _ = reply.send(response);
                Ok(turn_id.to_owned())
            }
        }
    }
}
