//! User-message preview extraction.
//!
//! The preview is the **operator's payload**, and the payload's one home on
//! disk is `<workspace>/agents/<agent-id>/goal.md` (DESIGN §3.3) — read by
//! [`super::enumerate`] in the same pass that reads the goal's two stamps.
//! What lernie *sent* the model is not that text: an assembled context leads
//! with the §3.7 pinned-instruction frame and wraps a deposit in its envelope,
//! so a request record previews as `<file path="…">` or `---` rather than as
//! anything the operator wrote (bl-368d).
//!
//! What the goal carries is the *composed* goal — the harness's identity stamp
//! above the operator's payload — so the preview is the payload's
//! **headline**: the stamp comes off first (line-wise, by the compose's own
//! inverse), then the first non-blank payload line, capped at [`PREVIEW_MAX`]
//! chars after whitespace normalization so the render layer can size
//! predictably. That order is what keeps the §3.3 display ladder's first two
//! rungs from being the same string.

pub(super) const PREVIEW_MAX: usize = 80;

/// The §3.3 ladder's second rung, at its source: the composed goal with the
/// identity stamp stripped **before** anything collapses (the strip is
/// line-wise — after [`truncate_preview`] there are no lines left to strip),
/// reduced to the payload's headline. Every prefill yog composes leads with one
/// (§3.3: `Ball <id>: <title>`, `Working directory: <dir>`, or the operator's
/// own ask), so the first non-blank line is the preview and the body never runs
/// on into it.
pub(super) fn payload_headline(goal: &str) -> String {
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
