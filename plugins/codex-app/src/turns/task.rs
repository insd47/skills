//! persistent Desktop task의 turn 하나를 소유한다. 완료 판정은 Desktop follower stream만 사용한다.

use crate::client::{Client, Follower};
use crate::protocol::{Completion, TurnReference};
use crate::turns::registry::{Command, Registry};
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct Task {
    pub cwd: PathBuf,
    pub registry: Registry,
    pub thread_id: String,
    pub prompt: String,
    pub commands: mpsc::UnboundedSender<Command>,
    pub receiver: mpsc::UnboundedReceiver<Command>,
}

impl Task {
    pub async fn run(mut self) -> Result<Completion> {
        let mut reference = None;
        let result = self.try_run(&mut reference).await;

        let result = result.or_else(|error| {
            tracing::error!(error = format!("{error:#}"), "Codex ask task failed");

            reference
                .as_ref()
                .map(|reference| Completion::failed(reference, &error))
                .ok_or_else(|| anyhow!("{error:#}"))
        });

        self.registry.remove(&self.thread_id, &self.commands).await;
        result
    }

    async fn try_run(&mut self, reference: &mut Option<TurnReference>) -> Result<Completion> {
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
    ) -> Result<Completion> {
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

        loop {
            if let Some(settlement) = follower.settlement(&turn_id) {
                return Ok(Self::completion(
                    current,
                    settlement.status,
                    settlement.result,
                ));
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

    fn completion(reference: TurnReference, status: String, result: String) -> Completion {
        let status = if status == "cancelled" {
            "interrupted"
        } else {
            &status
        };

        Completion::settled(reference, status, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::AskResult;

    fn reference() -> TurnReference {
        TurnReference {
            thread_id: "thread".into(),
            turn_id: "turn".into(),
        }
    }

    #[test]
    fn completed_settlement_becomes_the_blocking_ask_result() {
        let result = AskResult::Completed(Task::completion(
            reference(),
            "completed".into(),
            "done".into(),
        ));

        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::json!({
                "threadId":"thread", "turnId":"turn", "status":"completed", "result":"done"
            })
        );
    }

    #[test]
    fn cancelled_settlement_resolves_the_blocking_ask_as_interrupted() {
        let result = AskResult::Completed(Task::completion(
            reference(),
            "cancelled".into(),
            "stopped".into(),
        ));

        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::json!({
                "threadId":"thread", "turnId":"turn", "status":"interrupted",
                "result":"stopped"
            })
        );
    }
}
