use crate::protocol::Completion;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Outbox {
    directory: PathBuf,
}

impl Outbox {
    pub fn from_env(cwd: &Path) -> Self {
        let root = env::var_os("CLAUDE_PLUGIN_DATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".codex-app")
            });

        Self::new(root, cwd)
    }

    pub fn new(root: impl AsRef<Path>, cwd: &Path) -> Self {
        Self {
            directory: root
                .as_ref()
                .join("completions")
                .join(Self::key(cwd.as_os_str().as_encoded_bytes())),
        }
    }

    pub fn write(&self, completion: &Completion) -> Result<()> {
        fs::create_dir_all(&self.directory)?;
        let identity = format!("{}\0{}", completion.thread_id, completion.turn_id);

        let destination = self
            .directory
            .join(format!("{}.json", Self::key(identity.as_bytes())));

        let temporary = destination.with_extension(format!("{}.tmp", std::process::id()));
        let mut json = serde_json::to_vec(completion).context("Failed to serialize completion.")?;

        json.push(b'\n');
        fs::write(&temporary, json).context("Failed to write a temporary completion file.")?;
        fs::rename(&temporary, &destination).context("Failed to publish completion atomically.")?;

        Ok(())
    }

    pub fn take(&self) -> Result<Vec<Completion>> {
        fs::create_dir_all(&self.directory)?;
        let mut completions = Vec::new();

        for entry in fs::read_dir(&self.directory)? {
            let path = entry?.path();

            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let Ok(bytes) = fs::read(&path) else { continue };

            let Ok(completion) = serde_json::from_slice(&bytes) else {
                continue;
            };

            fs::remove_file(&path).context("Failed to remove a consumed completion file.")?;
            completions.push(completion);
        }

        Ok(completions)
    }

    fn key(value: &[u8]) -> String {
        format!("{:x}", Sha256::digest(value))[..24].to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::TurnKind;

    #[test]
    fn completion_is_consumed_once_with_compatible_shape() {
        let root = std::env::temp_dir().join(format!("codex-app-outbox-{}", uuid::Uuid::new_v4()));
        let outbox = Outbox::new(&root, Path::new("/workspace"));
        let completion = Completion {
            thread_id: "thread-1".into(),
            turn_id: "turn-1".into(),
            kind: TurnKind::Ask,
            status: "completed".into(),
            result: "done".into(),
        };

        outbox.write(&completion).unwrap();
        let taken = outbox.take().unwrap();
        assert_eq!(taken.len(), 1);
        assert_eq!(
            serde_json::to_value(&taken[0]).unwrap(),
            serde_json::json!({
                "threadId":"thread-1", "turnId":"turn-1", "kind":"ask",
                "status":"completed", "result":"done"
            })
        );
        assert!(outbox.take().unwrap().is_empty());
        fs::remove_dir_all(root).unwrap();
    }
}
