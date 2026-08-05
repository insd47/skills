use crate::outbox::Outbox;
use crate::protocol::Completion;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::io::{self, Write};
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(250);

pub async fn run(outbox: Outbox) -> Result<()> {
    let mut interval = tokio::time::interval(POLL_INTERVAL);

    loop {
        interval.tick().await;
        flush(&outbox)?;
    }
}

fn flush(outbox: &Outbox) -> Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    for completion in outbox.take()? {
        let notification = notification(&completion)?;
        serde_json::to_writer(&mut output, &notification)?;
        output.write_all(b"\n")?;
    }

    output.flush()?;
    Ok(())
}

fn notification(completion: &Completion) -> Result<Value> {
    let mut notification = serde_json::to_value(completion)?;

    notification
        .as_object_mut()
        .context("Completion JSON must be an object.")?
        .insert(
            "event".into(),
            json!(format!("codex-app.turn.{}", completion.status)),
        );

    Ok(notification)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::TurnKind;

    #[test]
    fn monitor_event_keeps_the_existing_wire_shape() {
        let value = notification(&Completion {
            thread_id: "thread".into(),
            turn_id: "turn".into(),
            kind: TurnKind::Ask,
            status: "completed".into(),
            result: "done".into(),
        })
        .unwrap();
        assert_eq!(
            value,
            json!({
                "event":"codex-app.turn.completed", "threadId":"thread", "turnId":"turn",
                "kind":"ask", "status":"completed", "result":"done"
            })
        );
    }
}
