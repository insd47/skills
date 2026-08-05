use crate::client::lookup::ThreadSummary;
use anyhow::{Result, bail};

pub fn target_thread(
    explicit: Option<&str>,
    title: Option<&str>,
    threads: &[ThreadSummary],
    cwd: &str,
) -> Result<String> {
    if let Some(thread_id) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(thread_id.to_owned());
    }

    let Some(title) = title.map(normalize).filter(|value| !value.is_empty()) else {
        bail!(
            "CODEX_APP_TITLE is unavailable. Set CODEX_APP_THREAD_ID to the target Codex task id and retry."
        );
    };

    let matching = threads
        .iter()
        .filter(|thread| {
            normalize(
                thread
                    .name
                    .as_deref()
                    .or(thread.preview.as_deref())
                    .unwrap_or(""),
            ) == title
        })
        .collect::<Vec<_>>();

    if let [thread] = matching.as_slice() {
        return Ok(thread.id.clone());
    }

    let details = if matching.is_empty() {
        String::new()
    } else {
        let labels = matching
            .iter()
            .take(5)
            .map(|thread| {
                let title = thread
                    .name
                    .as_deref()
                    .or(thread.preview.as_deref())
                    .unwrap_or("untitled");
                format!("{} ({})", thread.id, shorten(title, 60))
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(" Matches: {labels}.")
    };

    bail!(
        "CODEX_APP_TITLE did not identify exactly one unarchived Codex task in {cwd}: {title}.{details} Set CODEX_APP_THREAD_ID to the task id and retry."
    )
}

fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn shorten(value: &str, limit: usize) -> String {
    let value = normalize(value);

    if value.chars().count() <= limit {
        return value;
    }

    let prefix = value
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();

    format!("{prefix}...")
}

pub fn required(value: String, label: &str) -> Result<String> {
    let value = value.trim().to_owned();

    if value.is_empty() {
        bail!("{label} must not be empty.");
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thread(id: &str, name: &str) -> ThreadSummary {
        ThreadSummary {
            id: id.to_owned(),
            name: Some(name.to_owned()),
            preview: None,
        }
    }

    #[test]
    fn explicit_thread_bypasses_title_lookup_globally() {
        let selected = target_thread(Some(" thread-global "), None, &[], "/other/project").unwrap();

        assert_eq!(selected, "thread-global");
    }

    #[test]
    fn normalized_title_requires_exactly_one_match() {
        let threads = [thread("one", "Claude handoff"), thread("two", "Other")];
        let selected = target_thread(None, Some(" Claude   handoff "), &threads, "/work").unwrap();

        assert_eq!(selected, "one");
    }

    #[test]
    fn missing_and_ambiguous_titles_preserve_actionable_errors() {
        let missing = target_thread(None, None, &[], "/work").unwrap_err();
        assert!(
            missing
                .to_string()
                .contains("CODEX_APP_TITLE is unavailable")
        );

        let threads = [thread("one", "Same"), thread("two", "Same")];
        let ambiguous = target_thread(None, Some("Same"), &threads, "/work").unwrap_err();
        assert!(
            ambiguous
                .to_string()
                .contains("did not identify exactly one")
        );
        assert!(ambiguous.to_string().contains("one (Same), two (Same)"));
    }

    #[test]
    fn required_values_are_trimmed_and_reject_blank_input() {
        assert_eq!(required("  prompt  ".into(), "prompt").unwrap(), "prompt");
        assert!(required("  ".into(), "prompt").is_err());
    }
}
