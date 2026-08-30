//! §4.4 terminal reading rules for a settled `response.json`.
//!
//! Once the fd is closed (the classifier reaches here only with no lock
//! held, §3.5), the latest step's `response.json` is classified by its tail
//! (§4.4). **Two facts come off that one walk, and they answer different
//! questions** (bl-fb87):
//!
//! - [`Framing`] — the **transport** question: did the segment close cleanly?
//!   *complete* (last line `end`, last segment a `finish` with no `error`),
//!   *failed* (an `error` in the last segment — retry budget exhausted or
//!   non-retryable, §2.10), *killed* (no trailing `end` — the writer died
//!   mid-stream, §2.9).
//! - [`Ending`] — the **semantic** question: what ended the turn? Read from
//!   the canonical `finish.reason` brazen writes, verbatim.
//!
//! **Transport completion is not task completion.** A segment can keep every
//! transport promise — `finish`, no `error`, a trailing `end` — while the turn
//! it framed was cut off mid-utterance because the request's `max_tokens` ran
//! out. That is `Framing::Complete` and [`Ending::OutputLimit`], and the pair
//! is the whole of it: the framing stays honest (a sealed transcript entry
//! really was committed, which is what `rail::place` pairs against), and the
//! ending says the thing framing has no vocabulary for — that nothing more is
//! coming.
//!
//! Only a *complete and whole* tail is quiescent ([`Settled::whole`]); failed,
//! killed and output-limited are stopped (§3.5).
//! This is a self-delimiting reader over appended attempt segments (§4.4):
//! only the **last** segment decides, because it is the settled outcome.

/// The §4.4 settled outcome a closed `response.json` tail carries. Derived
/// from the **last** segment alone (the settled outcome), by the same
/// self-delimiting reader the live view uses. Widened to `pub(crate)` (the
/// enum + [`settled`] + [`segment_count`]) so the Y13 steps inspector reuses
/// this classifier instead of re-parsing the JSONL (§15 Y13: "reuse
/// git_tree::terminal's segment classification — do NOT duplicate the
/// parser").
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

/// What **ended the turn** — the semantic result the settled segment's
/// canonical `finish` names, beside [`Framing`]'s transport reading (bl-fb87).
///
/// Read from brazen's `FinishReason` verbatim and never inferred from token
/// counts: `usage == max_tokens` would duplicate a fact the reason already
/// states, and duplicated facts drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Ending {
    /// The settled segment carries no `finish` at all — [`Framing::Failed`] or
    /// [`Framing::Killed`], where there is no semantic result to read.
    #[default]
    Unread,
    /// The turn ended **on its own terms**: `stop`, `tool_use`,
    /// `stop_sequence`, `refusal`, `pause`, or any provider word brazen passed
    /// through as `FinishReason::Other`. Whichever it was, the model reached
    /// it rather than running out of room.
    OwnTerms,
    /// `reason: "length"` — the **output limit** ended it
    /// (`FinishReason::Length`): the request's `max_tokens` was reached with
    /// the model still mid-utterance, so whatever the segment holds is a
    /// fragment and there is no continuation to run. A turn that really did
    /// end with a tool call to make reads `tool_use`, not this — the reason is
    /// one value and the provider names it, which is why the reason is the
    /// authority and the content is never sniffed.
    OutputLimit,
}

/// The whole §4.4 reading of a settled tail: transport [`Framing`] and
/// semantic [`Ending`], off one walk. Neither is recoverable from the other,
/// so a caller that needs both reads the file once (§5.1 #10's discipline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Settled {
    pub framing: Framing,
    pub ending: Ending,
}

impl Settled {
    /// A tail with no clean end and nothing to read from it (§4.4 *killed*).
    pub(crate) const KILLED: Self = Self {
        framing: Framing::Killed,
        ending: Ending::Unread,
    };

    /// Is this the §4.4 *complete and whole* reading — the transport closed
    /// clean **and** the turn ended on its own terms? The live classifier's
    /// boolean view (§3.5's quiescent test). A turn cut off at the output
    /// limit is complete on the wire and unfinished as a turn, so it is not
    /// this.
    pub fn whole(self) -> bool {
        self.framing == Framing::Complete && self.ending != Ending::OutputLimit
    }
}

/// Classify a `response.json` payload by its tail (§4.4). Only the last
/// segment decides. A trailing partial line (no `\n` yet) is dropped.
pub(crate) fn settled(bytes: &[u8]) -> Settled {
    // Only fully-terminated lines: drop anything after the final newline.
    let terminated = match bytes.iter().rposition(|&b| b == b'\n') {
        // `idx` is a position within `bytes`, so `..=idx` always yields `Some`.
        Some(idx) => bytes.get(..=idx).unwrap_or(bytes),
        None => return Settled::KILLED,
    };
    let lines: Vec<&[u8]> = terminated
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .collect();
    let Some((last, rest)) = lines.split_last() else {
        return Settled::KILLED;
    };
    if event_type(last) != Some("end") {
        return Settled::KILLED;
    }
    // Walk the last segment backward from just before the final `end` to
    // the previous segment's `end` boundary; require a `finish`, no `error`.
    // Backward, so the first `finish` met is the segment's last one.
    let mut ending = None;
    for line in rest.iter().rev() {
        match event(line) {
            Some(("end", _)) => break,
            Some(("error", _)) => {
                return Settled {
                    framing: Framing::Failed,
                    ending: Ending::Unread,
                };
            }
            Some(("finish", value)) => ending = ending.or_else(|| Some(finish_ending(&value))),
            _ => {}
        }
    }
    match ending {
        Some(ending) => Settled {
            framing: Framing::Complete,
            ending,
        },
        None => Settled::KILLED,
    }
}

/// The `reason` brazen writes beside a `finish` event
/// (`{"type":"finish","reason":"length"}`) that yog reads differently from
/// every other. One word, because it is the one whose consequence differs: the
/// turn did not end, it ran out of room.
const LENGTH_REASON: &str = "length";

/// Read one already-parsed `finish` event as its [`Ending`]. Any reason but
/// [`LENGTH_REASON`] — and a `finish` carrying no readable reason at all —
/// is [`Ending::OwnTerms`], which leaves every pre-existing classification
/// exactly where it was.
fn finish_ending(value: &serde_json::Value) -> Ending {
    if value.get("reason").and_then(serde_json::Value::as_str) == Some(LENGTH_REASON) {
        Ending::OutputLimit
    } else {
        Ending::OwnTerms
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
/// `Some(line)` **iff** [`settled`] would answer [`Framing::Failed`] — the same
/// last-segment traversal, stopping at the first `error` before the previous
/// segment boundary; `None` for complete / killed / empty. The verbatim event
/// line (its `kind`/`message`/`status` fields and all) is what the auth heuristic
/// ([`crate::login::auth`]) scans, so the classifier stays schema-agnostic (bz's
/// and litany's error shapes may differ, §5.1 #10/#13).
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

/// The classifier-relevant `type` field of one JSONL event line **and the
/// event itself**, or `None` if it does not parse as a JSON object with a
/// recognized string `type`. The value rides along so the `finish` arm can
/// read its `reason` without a second `from_slice` of a line that already
/// parsed — a second parse would carry a failure arm nothing can reach.
fn event(line: &[u8]) -> Option<(&'static str, serde_json::Value)> {
    let value: serde_json::Value = serde_json::from_slice(line).ok()?;
    let kind = match value.get("type").and_then(serde_json::Value::as_str)? {
        "end" => "end",
        "finish" => "finish",
        "error" => "error",
        _ => return None,
    };
    Some((kind, value))
}

/// [`event`]'s thin projection, for the two walks that need only the kind.
fn event_type(line: &[u8]) -> Option<&'static str> {
    event(line).map(|(kind, _)| kind)
}

#[cfg(test)]
mod tests;
