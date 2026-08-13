//! User-message preview extraction.
//!
//! The preview is read from disk at
//! `<workspace>/steps/<agent-id>/001/request.json` — step records are
//! never in a git tree (§2.3 "Step records are not committed to git").
//! What was sent is the *composed* goal — the harness's identity stamp above
//! the operator's payload (DESIGN §3.3) — so the preview is the payload's
//! **headline**: the stamp comes off first (line-wise, by the compose's own
//! inverse), then the first non-blank payload line, capped at [`PREVIEW_MAX`]
//! chars after whitespace normalization so the render layer can size
//! predictably. That order is what keeps the §3.3 display ladder's first two
//! rungs from being the same string.

pub(super) const PREVIEW_MAX: usize = 80;

pub(super) fn extract_request_preview(json_bytes: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(json_bytes).ok()?;
    let content = value.get("messages")?.as_array()?.first()?.get("content")?;
    Some(payload_headline(content_text(content)?))
}

/// The sent goal's text, from either content shape the API accepts: a plain
/// string, or a block array (lernie children's step-001 requests) whose first
/// `text` block carries the goal. An array with no text block has no text —
/// the row floors as today.
fn content_text(content: &serde_json::Value) -> Option<&str> {
    match content {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Array(blocks) => blocks
            .iter()
            .find(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))?
            .get("text")?
            .as_str(),
        _ => None,
    }
}

/// The §3.3 ladder's second rung, at its source: the sent goal with the identity
/// stamp stripped **before** anything collapses (the strip is line-wise — after
/// [`truncate_preview`] there are no lines left to strip), reduced to the
/// payload's headline. Every prefill yog composes leads with one (§3.3: `Ball
/// <id>: <title>`, `Working directory: <dir>`, or the operator's own ask), so
/// the first non-blank line is the preview and the body never runs on into it.
fn payload_headline(goal: &str) -> String {
    let payload = crate::start::strip_identity_stamp(goal);
    let headline = payload
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();
    truncate_preview(headline)
}

pub(super) fn truncate_preview(s: &str) -> String {
    let collapsed: String = s
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    let trimmed = collapsed.trim();
    if trimmed.chars().count() <= PREVIEW_MAX {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(PREVIEW_MAX - 1).collect();
    format!("{head}…")
}
