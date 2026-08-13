//! §4.4 terminal reading rules for a settled `response.json`.
//!
//! Once the fd is closed (the classifier reaches here only with no lock
//! held, §3.5), the latest step's `response.json` is classified by its tail
//! (§4.4):
//!
//! - *complete* — last line `end`, and the last segment carries a `finish`
//!   with no `error`.
//! - *failed* — last segment carries an `error` (retry budget exhausted or
//!   non-retryable, §2.10).
//! - *killed* — closed with no trailing `end` (writer died mid-stream,
//!   §2.9).
//!
//! Only *complete* is quiescent; *failed* and *killed* are stopped (§3.5).
//! This is a self-delimiting reader over appended attempt segments (§4.4):
//! only the **last** segment decides, because it is the settled outcome.

/// The §4.4 settled outcome a closed `response.json` tail carries. Derived
/// from the **last** segment alone (the settled outcome), by the same
/// self-delimiting reader the live view uses. Widened to `pub(crate)` (the
/// enum + [`framing`] + [`segment_count`]) so the Y13 steps inspector reuses
/// this classifier instead of re-parsing the JSONL (§15 Y13: "reuse
/// git_tree::terminal's segment classification — do NOT duplicate the
/// parser"); [`last_segment_complete`] stays the live classifier's thin view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// Last line `end`, last segment a `finish` with no `error` (§4.4).
    Complete,
    /// Last segment carries an `error` — retry budget exhausted or
    /// non-retryable (§2.10).
    Failed,
    /// Closed with no trailing `end`, empty, or an `end` with no `finish`
    /// — the writer died mid-stream, the step never ran, or a call in
    /// flight right now (kill/crash/in-flight are indistinguishable on
    /// disk, §2.9).
    Killed,
}

/// Classify a `response.json` payload by its tail (§4.4). Only the last
/// segment decides. A trailing partial line (no `\n` yet) is dropped.
pub(crate) fn framing(bytes: &[u8]) -> Framing {
    // Only fully-terminated lines: drop anything after the final newline.
    let terminated = match bytes.iter().rposition(|&b| b == b'\n') {
        // `idx` is a position within `bytes`, so `..=idx` always yields `Some`.
        Some(idx) => bytes.get(..=idx).unwrap_or(bytes),
        None => return Framing::Killed,
    };
    let lines: Vec<&[u8]> = terminated
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .collect();
    let Some((last, rest)) = lines.split_last() else {
        return Framing::Killed;
    };
    if event_type(last) != Some("end") {
        return Framing::Killed;
    }
    // Walk the last segment backward from just before the final `end` to
    // the previous segment's `end` boundary; require a `finish`, no `error`.
    let mut saw_finish = false;
    for line in rest.iter().rev() {
        match event_type(line) {
            Some("end") => break,
            Some("error") => return Framing::Failed,
            Some("finish") => saw_finish = true,
            _ => {}
        }
    }
    if saw_finish {
        Framing::Complete
    } else {
        Framing::Killed
    }
}

/// Number of completed attempt segments — the count of `end` events (§4.4:
/// every attempt segment terminates with an `end`). A still-open final
/// segment (no trailing `end`) is not yet counted; this is the "attempts"
/// figure the steps inspector shows.
pub(crate) fn segment_count(bytes: &[u8]) -> usize {
    bytes
        .split(|&b| b == b'\n')
        .filter(|line| event_type(line) == Some("end"))
        .count()
}

/// The raw JSONL text of the last settled segment's `error` event, when it
/// carries one (§4.4 *failed* framing; §5.1 #13 "response/error text"). Returns
/// `Some(line)` **iff** [`framing`] would return [`Framing::Failed`] — the same
/// last-segment traversal, stopping at the first `error` before the previous
/// segment boundary; `None` for complete / killed / empty. The verbatim event
/// line (its `kind`/`message`/`status` fields and all) is what the auth heuristic
/// ([`crate::login::auth`]) scans, so the classifier stays schema-agnostic (bz's
/// and lernie's error shapes may differ, §5.1 #10/#13).
pub(crate) fn error_text(bytes: &[u8]) -> Option<String> {
    let terminated = match bytes.iter().rposition(|&b| b == b'\n') {
        // `idx` is a position within `bytes`, so `..=idx` always yields `Some`.
        Some(idx) => bytes.get(..=idx).unwrap_or(bytes),
        None => return None,
    };
    let lines: Vec<&[u8]> = terminated
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .collect();
    let (last, rest) = lines.split_last()?;
    if event_type(last) != Some("end") {
        return None;
    }
    for line in rest.iter().rev() {
        match event_type(line) {
            Some("end") => return None,
            Some("error") => return Some(String::from_utf8_lossy(line).into_owned()),
            _ => {}
        }
    }
    None
}

/// Is the payload a §4.4 *complete* model call? `false` for failed, killed,
/// and empty files — the live classifier's boolean view of [`framing`].
pub(super) fn last_segment_complete(bytes: &[u8]) -> bool {
    framing(bytes) == Framing::Complete
}

/// The classifier-relevant `type` field of one JSONL event line, or `None`
/// if it does not parse as a JSON object with a recognized string `type`.
fn event_type(line: &[u8]) -> Option<&'static str> {
    let value: serde_json::Value = serde_json::from_slice(line).ok()?;
    match value.get("type")?.as_str()? {
        "end" => Some("end"),
        "finish" => Some("finish"),
        "error" => Some("error"),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
