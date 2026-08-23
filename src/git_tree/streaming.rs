//! Live-streaming accumulation for in-flight steps.
//!
//! Per ARCH §2.3 / §3.5 / §4.4: the harness writes
//! `<conv-repo>/steps/<conv-id>/<NNN>/response.json` as a JSONL stream
//! of §4.4 events as the model's adapter emits them, then closes the fd
//! at completion. The frontend tails this file from disk on every tick
//! (§3.5: re-read, no in-memory accumulator that could drift).
//!
//! **One read, one value** (§5.1 #10, #28b). The fold yields a [`Stream`]:
//! the accumulated answer text, the accumulated reasoning text, and the
//! **kind of the last content delta** that landed. The last is what splits a
//! live model call into the three things it can be doing — nothing back yet,
//! thinking, answering — for the per-agent `Doing` derivation the §11 live
//! mark paints. Reading the file twice would cost a second syscall per agent
//! per tick and could catch two different mid-write states of one file, so the
//! facts come off one pass or they are not one file's answer at all — which is
//! why they travel as one value ([`Agent::stream`](super::Agent::stream))
//! rather than as fields that could be filled from two reads.
//!
//! The fold is **resumable** ([`Stream::absorb`], §7.2 live tail): folding the
//! bytes appended since the last read and absorbing the result is the same
//! `Stream` as folding the file whole, so the frame-cadence follower
//! (`app::live`) reuses this parser on a growing suffix instead of growing a
//! second one.
//!
//! Functions here are pure over an on-disk `<conv-repo>/steps/` tree.
//! Events outside the `content_delta` seam (`message_start`, tool-argument
//! deltas, etc.) are ignored at this layer — pulsing tool indicators (bl-23d9)
//! and branch-state badges (bl-de6b) read their own signals.
//!
//! Deltas are read from brazen's `v=1` `content_delta` (§4.4). The v0.6 legacy
//! vocabulary is retired (bl-56ee).

use std::path::Path;

use super::STEPS_DIR;

/// The fold's own JSON spelling (REMOTE §3, bl-73e7) — the follow lane's frame
/// body, beside the type rather than at the boundary that carries it.
pub mod wire;

/// Width of the zero-padded step sequence in on-disk paths
/// (`steps/<conv-id>/001`, `…/002`, ...). Mirrors
/// `src/prompt/step::STEP_SEQ_WIDTH` — duplicated here to keep the UI
/// crate free of a dep on the harness binary.
const STEP_SEQ_WIDTH: usize = 3;

/// The live tail the harness appends stream events to and closes at
/// completion. `pub(super)` so the §11 recency derivation
/// (`enumerate::last_action_from_disk`) names the same file this folds.
pub(super) const RESPONSE_FILE: &str = "response.json";

/// The kind of one `content_delta` — the brazen `v=1` `Delta` arms yog reads
/// (§4.4). `json_delta` (tool arguments) is not one of them: it is the model
/// composing a tool call, which the §5.1 #10 tool records already say better.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delta {
    /// `text_delta` — answer text, the part the live tail displays.
    Text,
    /// `thinking_delta` — reasoning. It is *also* display text (§7.2 the
    /// thinking ruling): a badge that never grows cannot tell a model thinking
    /// hard from a driver that has hung, and "nothing is happening on screen"
    /// is the complaint this whole seam exists to answer.
    Thinking,
}

/// What one read of the latest step's `response.json` says (§5.1 #10, #28b) —
/// every fact off one pass, so they cannot describe two different mid-write
/// states of the same file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stream {
    /// The accumulated answer text: `Some` once at least one `text_delta`
    /// has landed, `None` while the file is absent, empty, or carries only
    /// deltas that are not answer text.
    pub text: Option<String>,
    /// The accumulated reasoning text, on the same terms — `Some` once a
    /// `thinking_delta` has landed. Held apart from [`text`](Self::text)
    /// because they are two different things being said and the transcript
    /// paints them as two rows, exactly as a *committed* model entry's
    /// `Block::Thinking` and `Block::Text` already are.
    pub thinking: Option<String>,
    /// The kind of the **last** content delta seen. `None` means the stream
    /// has produced nothing yet — which, under an open `response.json` fd
    /// (§5.1 #9), is exactly "waiting for the API".
    pub last_delta: Option<Delta>,
}

impl Stream {
    /// Absorb the fold of the bytes that landed **after** this one's — the
    /// resumability the §7.2 live-tail follower rides. Text accretes in stream
    /// order and the newer delta kind wins when the suffix had one at all, so
    /// `fold(a).absorb(fold(b)) == fold(a ++ b)` for any split on a line
    /// boundary. That equality is the whole contract: the follower never has a
    /// second parser, only a second *place* to start reading.
    pub(crate) fn absorb(&mut self, later: Self) {
        append(&mut self.text, later.text);
        append(&mut self.thinking, later.thinking);
        self.last_delta = later.last_delta.or(self.last_delta);
    }
}

/// Accrete `more` onto an accumulator that may not exist yet. Absent stays
/// absent — a stream that has said nothing has said nothing, and an empty
/// `Some("")` would read as "it spoke" to every seat downstream.
fn append(slot: &mut Option<String>, more: Option<String>) {
    if let Some(more) = more {
        slot.get_or_insert_default().push_str(&more);
    }
}

/// Read the latest step's live stream from disk.
///
/// `pub` because it completes the view-model boundary: [`Transcript::with_live`]
/// (crate::transcript::Transcript::with_live) takes a [`Stream`], and a public
/// surface that consumes one owes a public way to obtain one off a workspace.
///
/// "Latest step" is the highest `<NNN>` directory under
/// `<conv-repo>/steps/<conv-id>/`. Re-derived on every call from the
/// directory listing so the view-model has no in-memory state to drift
/// out of sync with disk (§3.5). An absent conversation, an absent file and
/// an unreadable one all read as the default [`Stream`] — nothing has come
/// back — which is the general path with empty input, not a case.
pub fn stream_from_disk(workspace: &Path, agent_id: &str) -> Stream {
    let agent_steps = workspace.join(STEPS_DIR).join(agent_id);
    let Some(latest) = latest_step_dir(&agent_steps) else {
        return Stream::default();
    };
    match std::fs::read(latest.join(RESPONSE_FILE)) {
        Ok(bytes) => fold_stream(&bytes),
        Err(_) => Stream::default(),
    }
}

/// The latest step's open response file, or `None` when no step has opened
/// one — the path the §7.2 live-tail follower holds its offset into.
/// `pub(crate)` because the follower is the only caller that wants the *file*
/// rather than the fold.
pub(crate) fn latest_response_path(workspace: &Path, agent_id: &str) -> Option<std::path::PathBuf> {
    let agent_steps = workspace.join(STEPS_DIR).join(agent_id);
    Some(latest_step_dir(&agent_steps)?.join(RESPONSE_FILE))
}

/// Find the highest-numbered `<NNN>/` directory under `conv_steps`.
/// Entries that don't match the zero-padded step shape are ignored —
/// `tools/` lives one level deeper, so it's structurally not at risk
/// here, but staying strict keeps us robust to stray files.
pub(super) fn latest_step_dir(conv_steps: &Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(conv_steps).ok()?;
    let mut best: Option<(u32, std::path::PathBuf)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        if name_str.len() != STEP_SEQ_WIDTH {
            continue;
        }
        let Ok(seq) = name_str.parse::<u32>() else {
            continue;
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if best.as_ref().is_none_or(|(s, _)| seq > *s) {
            best = Some((seq, path));
        }
    }
    best.map(|(_, p)| p)
}

/// Fold a JSONL `response.json` payload into its [`Stream`]. Each line is a
/// §4.4 stream event; text fragments accumulate in stream order across all
/// block indices, and every recognised delta moves `last_delta` — so the field
/// ends on the newest one, which is what the operator is watching happen.
/// Lines that fail to parse, and events outside the delta seam, are skipped —
/// partial-write tolerance is structural (the harness may be mid-line on disk).
/// `pub(crate)` so the §7.2 follower folds its appended suffix through this one
/// parser and absorbs the result ([`Stream::absorb`]).
pub(crate) fn fold_stream(bytes: &[u8]) -> Stream {
    let (mut text, mut thinking) = (String::new(), String::new());
    let mut last_delta = None;
    for line in bytes.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let Ok(value): Result<serde_json::Value, _> = serde_json::from_slice(line) else {
            continue;
        };
        if let Some((kind, fragment)) = delta_of(&value) {
            match kind {
                Delta::Text => text.push_str(fragment),
                Delta::Thinking => thinking.push_str(fragment),
            }
            last_delta = Some(kind);
        }
    }
    Stream {
        text: said(text),
        thinking: said(thinking),
        last_delta,
    }
}

/// An accumulator as the fact it is: nothing said reads as absent, never as an
/// empty utterance — every seat downstream branches on `Some`.
fn said(accumulated: String) -> Option<String> {
    (!accumulated.is_empty()).then_some(accumulated)
}

/// The brazen `v=1` delta seam:
/// `{"type":"content_delta","delta":{"<arm>":"…"}}` — the externally-tagged
/// `Delta`'s two arms yog reads, with their payloads. A `json_delta` (tool
/// arguments) or any other event yields `None`.
fn delta_of(value: &serde_json::Value) -> Option<(Delta, &str)> {
    if value.get("type").and_then(|v| v.as_str())? != "content_delta" {
        return None;
    }
    let delta = value.get("delta")?;
    let arm = |key: &str| delta.get(key).and_then(|v| v.as_str());
    arm("text_delta")
        .map(|t| (Delta::Text, t))
        .or_else(|| arm("thinking_delta").map(|t| (Delta::Thinking, t)))
}

#[cfg(test)]
mod tests;
