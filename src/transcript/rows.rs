//! The one-line row projection of a transcript (DESIGN §11 transcript
//! density rule).
//!
//! Vertical space is the scarce resource: **every** transcript row — a
//! delivered message, one model text block, one thinking block, one tool
//! call, one tool result, the live tail — renders as exactly ONE line that
//! folds open to its full payload. A row is therefore a *block*, not a file:
//! a model message that says something and then calls two tools is three
//! rows, because it is three things.
//!
//! **Expansion is derived, never stored per row.** The auto-state is a pure
//! function of the row's class and the two durable knobs ([`AutoExpand`],
//! `ui.json` §4.1): the conversation expands, the machinery around it
//! contracts. The caller's fold set holds *explicit overrides only* — the
//! `ui.json.collapsed` discipline applied to RAM (§5.3): `expanded = auto
//! XOR overridden`. That dissolves "state on arrival" as a special case —
//! there is no arrival event, and a row that appears mid-frame is already in
//! its auto-state without anyone having to notice it appeared.
//!
//! Keys are the row's identity — `tx/<entry filename>#<block index>` — so
//! they survive the stateless re-read of the whole transcript each frame and
//! never collide with the jsonview collapse paths sharing the caller's set.
//!
//! Cut at the real seams: **what a row is** (the vocabulary below — the
//! classes, the tones, the auto-state rule), **what an entry becomes**
//! ([`project`] — the exhaustive per-variant match and the preview/body
//! split), and **what a finished turn becomes** ([`turns`] — the rollup of a
//! turn's machinery to one aggregate row). The vocabulary is read by the
//! render and by every test; the other two are read by nothing but [`rows`].

use std::collections::HashSet;

use super::Transcript;

mod project;
mod turns;

pub(crate) use project::key;

/// The two auto-state knobs (§4.1 `ui.json`, §11): whether a class expands on
/// its own. Defaults are the operator's ruling — the conversation open, all
/// else folded — and both are knobs so the policy is config, not code
/// (severability).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoExpand {
    /// The conversation itself: delivered messages, model text, the live tail.
    pub responses: bool,
    /// Everything else: thinking, tool calls/results, raw bytes.
    pub others: bool,
}

impl Default for AutoExpand {
    fn default() -> Self {
        Self {
            responses: true,
            others: false,
        }
    }
}

/// Which auto-knob a row answers to. The split is **conversation vs
/// machinery**, not model vs everyone: a message delivered *to* the agent is
/// the other half of the exchange the operator came to read, so it arrives
/// expanded beside the reply it provoked (bl-6ec6 — a user turn folded shut
/// is the operator's own words hidden from them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowClass {
    /// Someone talking: a delivered message, a model text block, the live tail.
    Response,
    /// Machinery: thinking, tool calls, tool results, raw bytes.
    Other,
}

/// The paint hue a row asks for; the render maps each to a `theme` constant
/// (never an RGB restated here — §11 single colour authority).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Weak,
    Good,
    Bad,
    /// The live streaming tail (spectral blue).
    Live,
    /// A tool call with no result yet — pulses.
    InFlight,
}

/// What a row's `▶` opens onto — the two things a fold can reveal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    /// The row's own `body`, printed beneath it. A row whose body is empty has
    /// nothing to reveal and shows no toggle at all.
    Payload,
    /// A finished turn's step rows, which follow this row while it is expanded
    /// and are absent from the projection while it is not — each of them
    /// folding on its own, all the way down.
    Steps,
}

/// One transcript line. `prefix` is the always-visible label (`sender:`,
/// `⚙ Read`, `✔ tool result — ok`); `preview` is the payload's first line, shown
/// while contracted; `body` is the full payload, shown while expanded and
/// **empty when the payload already fits the one line** (such a row has
/// nothing to fold and shows no toggle); `hover` is what the prefix stands for,
/// empty when the label says everything it has to say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    pub prefix: String,
    pub preview: String,
    pub body: String,
    pub hover: String,
    pub class: RowClass,
    pub tone: Tone,
    /// Who the row speaks for (§11 role stripe, bl-3acb) — derived from the
    /// entry's committed bytes by the projection, `None` on machinery rows
    /// (nobody is speaking). The render paints it through the one mapping
    /// ([`crate::theme::role_badge`]), exactly as tones map to theme hues.
    pub role: Option<crate::theme::Role>,
    pub fold: Fold,
    pub expanded: bool,
}

/// Project `transcript` into one-line rows. `speaker` is the conversation's
/// §3.3 display name — **who** the model turns are, since a speaker is an agent
/// and not a model id (bl-2335); `auto` is the durable knob pair; `folds` is the
/// caller's RAM override set (§5.3) — membership *flips* a row's auto-state, so
/// an empty set means "everything as configured".
pub fn rows(
    transcript: &Transcript,
    speaker: &str,
    auto: AutoExpand,
    folds: &HashSet<String>,
) -> Vec<Row> {
    let mut flat = Vec::new();
    let mut steps = Vec::new();
    let mut usage = Vec::new();
    for entry in &transcript.entries {
        let before = flat.len();
        project::push_entry(transcript, entry, speaker, &mut flat);
        for block in 0..flat.len() - before {
            steps.push(turns::step_of(&entry.kind, block));
            usage.push(turns::usage_of(&entry.kind));
        }
    }
    let mut out = turns::group(&flat, &steps, &usage, auto, folds);
    for row in &mut out {
        let expanded = expanded_for(row, auto, folds.contains(&row.key));
        row.expanded = expanded;
    }
    out
}

/// The auto-state, flipped by an explicit override (the whole expansion rule).
/// A row that is **in flight** auto-expands whatever its class knob says: while
/// a step is happening it is the show, and completion returns it to its class
/// auto-state with no event to notice and nothing to store.
fn expanded_for(row: &Row, auto: AutoExpand, overridden: bool) -> bool {
    let auto_on = in_flight(row)
        || match row.class {
            RowClass::Response => auto.responses,
            RowClass::Other => auto.others,
        };
    auto_on != overridden
}

/// Is this row a step happening **right now** — the live streaming tail, or a
/// tool call no result has retired yet? Already said by the tone the
/// projection gave it, so in-flightness stays the query it always was.
fn in_flight(row: &Row) -> bool {
    matches!(row.tone, Tone::Live | Tone::InFlight)
}
