//! Turn rollup (DESIGN §11): what the agent *did* between two things it
//! *said*, folded to one aggregate line.
//!
//! The ruling: the moment each step is done it collapses down, until all that
//! is left is a single line — "3150 thinking tokens, 9 inference calls, 14
//! tool calls", or whatever the turn actually was — expandable to see each
//! step in flight. When it is done and the agent is responding, just one line
//! before the response.
//!
//! A **turn** is derived, never stored. Delivered messages delimit the row
//! sequence — a message *to* the agent is the other half of the exchange, so
//! it is a boundary and never a step inside a turn — and within a segment the
//! turn's **answer** is its last row, when the model ended by talking.
//! Everything before that answer is the turn's machinery: thinking, tool
//! calls, tool results, and the model's own intermediate remarks. It rolls up
//! into ONE aggregate row whose fold opens onto those very step rows, each
//! still folding on its own.
//!
//! Three conditions gate the rollup, all read off the rows themselves:
//!
//! - the turn **ended by talking** — an unfinished turn keeps its steps on
//!   screen, because that is the work in progress the operator came to watch;
//! - **nothing in it is in flight** ([`super::in_flight`]) — a live tail or an
//!   unretired tool call makes the whole turn the show;
//! - it holds **at least one inference call** — a run of stray entries with no
//!   model output is not a turn, which is also why the aggregate line can
//!   never come out empty.

use std::collections::HashSet;

use super::{AutoExpand, Fold, Row, RowClass, Tone, expanded_for, in_flight};
use crate::transcript::{Block, EntryKind, Usage};

/// Key suffix of a turn's aggregate row, where a block ordinal would sit. An
/// ordinal is always a number, so the two can never collide, and the key is
/// the turn's first entry — stable across the stateless re-read.
const TURN_SUFFIX: &str = "turn";
/// The machinery glyph, as the tool-call rows already wear it.
const TURN_GLYPH: &str = "⚙";
/// Separator between the aggregate's terms.
const TERM_SEP: &str = " · ";

/// What a projected row is *to the grouping*: the boundary it must not cross,
/// and the provenance its aggregate counts. Derived from the entry the row
/// came from ([`step_of`]) — nothing new is stored on the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    /// A delivered message, or a compaction marker: a turn boundary.
    Boundary,
    /// Model-authored output with no term of its own — an intermediate text
    /// block, or the one row an entry that committed no blocks gets. It still
    /// witnesses the inference call its entry stands for.
    Model,
    /// One thinking block.
    Thinking,
    /// One tool call.
    ToolCall,
    /// Not model-authored: a tool result, raw bytes, the live tail.
    Plain,
}

/// Which [`Step`] the `block`-th row of an entry of this kind is — the
/// one-row-per-block correspondence [`super::project`] emits.
pub(super) fn step_of(kind: &EntryKind, block: usize) -> Step {
    match kind {
        // A compaction marker is a **boundary** for the same reason a
        // delivered message is: it is not something the agent did between two
        // things it said. It says the record was rewritten here, and a turn
        // that swallowed it into its aggregate would hide the one row saying
        // so behind a fold — which is the very silence bl-7bd2 closed.
        EntryKind::Delivered { .. } | EntryKind::Compacted { .. } => Step::Boundary,
        EntryKind::Model { blocks, .. } => match blocks.get(block) {
            Some(Block::Thinking(_)) => Step::Thinking,
            Some(Block::ToolUse { .. }) => Step::ToolCall,
            Some(Block::Text(_)) | None => Step::Model,
        },
        EntryKind::ToolResult { .. } | EntryKind::Streaming { .. } | EntryKind::Raw => Step::Plain,
    }
}

/// The committed usage record every row of a model entry points back to —
/// the third parallel projection [`super::rows`] builds beside the rows and
/// their steps. `None` for anything not model-authored; the empty record is
/// a model entry whose bytes carried no report (the legacy shape).
pub(super) fn usage_of(kind: &EntryKind) -> Option<&Usage> {
    match kind {
        EntryKind::Model { usage, .. } => Some(usage),
        _ => None,
    }
}

/// Roll every finished turn's machinery up into its aggregate row. `steps`
/// runs parallel to `flat`; `auto`/`folds` decide whether an aggregate is open,
/// because a shut one leaves its step rows out of the projection entirely.
pub(super) fn group(
    flat: &[Row],
    steps: &[Step],
    usage: &[Option<&Usage>],
    auto: AutoExpand,
    folds: &HashSet<String>,
) -> Vec<Row> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, step) in steps.iter().enumerate() {
        if *step != Step::Boundary {
            continue;
        }
        push_turn(
            span(flat, start, i),
            span(steps, start, i),
            span(usage, start, i),
            auto,
            folds,
            &mut out,
        );
        if let Some(boundary) = flat.get(i) {
            out.push(boundary.clone());
        }
        start = i + 1;
    }
    push_turn(
        span(flat, start, flat.len()),
        span(steps, start, steps.len()),
        span(usage, start, usage.len()),
        auto,
        folds,
        &mut out,
    );
    out
}

/// `slice[from..to]`, checked — an out-of-range span is empty rather than a
/// panic path (the house rule against unchecked slicing).
fn span<T>(slice: &[T], from: usize, to: usize) -> &[T] {
    slice.get(from..to).unwrap_or_default()
}

/// One boundary-delimited segment: the machinery run, then the answer that
/// ended it. Rolls up when the three conditions hold, else passes through.
fn push_turn(
    rows: &[Row],
    steps: &[Step],
    usage: &[Option<&Usage>],
    auto: AutoExpand,
    folds: &HashSet<String>,
    out: &mut Vec<Row>,
) {
    let (Some((answer, run)), Some((_, run_steps))) = (rows.split_last(), steps.split_last())
    else {
        out.extend_from_slice(rows);
        return;
    };
    let run_usage = span(usage, 0, run.len());
    let counts = Counts::of(run, run_steps, run_usage);
    let rolls_up =
        answer.class == RowClass::Response && counts.inference > 0 && !rows.iter().any(in_flight);
    match run.first() {
        Some(first) if rolls_up => {
            let parent = aggregate(&first.key, &counts, auto, folds);
            let open = parent.expanded;
            out.push(parent);
            if open {
                out.extend_from_slice(run);
            }
            out.push(answer.clone());
        }
        _ => out.extend_from_slice(rows),
    }
}

/// The turn's aggregate row: machinery, so it answers the same auto-knob every
/// other machinery row does — one line by default, and the operator who set
/// that knob open gets every turn opened, steps and all.
fn aggregate(first_key: &str, counts: &Counts, auto: AutoExpand, folds: &HashSet<String>) -> Row {
    let key = turn_key(first_key);
    let mut row = Row {
        expanded: false,
        prefix: format!("{TURN_GLYPH} {}", counts.say()),
        preview: String::new(),
        body: String::new(),
        hover: "what the agent did before answering — open it for each step".to_string(),
        class: RowClass::Other,
        tone: Tone::Weak,
        // Machinery rolled up is still machinery: no one is speaking (§11
        // role stripe), so the aggregate wears the empty seat.
        role: None,
        fold: Fold::Steps,
        key,
    };
    row.expanded = expanded_for(&row, auto, folds.contains(&row.key));
    row
}

/// The aggregate's identity: the turn's first row's entry, with the block
/// ordinal replaced by [`TURN_SUFFIX`].
fn turn_key(first_key: &str) -> String {
    let entry = first_key
        .rsplit_once('#')
        .map_or(first_key, |(head, _)| head);
    format!("{entry}#{TURN_SUFFIX}")
}

/// What a turn's folded machinery holds, counted from the committed bytes and
/// nothing else (bl-8433: a badge claiming more than its data knows is a
/// filed bug class). Token sums come **only** from the entries' committed
/// `usage` records (lernie ≥0.0.4) — never estimated: a legacy entry with no
/// record contributes nothing, and [`Counts::say`] words a mixed turn for it.
struct Counts {
    /// Distinct model entries the run's rows came from.
    inference: usize,
    tools: usize,
    thinking: usize,
    /// Per-counter token sums over the counted entries' committed `usage`
    /// records, under the counters' own committed names.
    tokens: Usage,
    /// How many of the counted entries carried a `usage` record at all.
    reported: usize,
}

impl Counts {
    /// Count one machinery run. Rows of one entry are contiguous (the
    /// projection walks entries in order), so a new inference call is exactly
    /// a model-authored row whose entry differs from the previous one's —
    /// and that is also the once-per-entry moment its usage record folds in.
    fn of(rows: &[Row], steps: &[Step], usage: &[Option<&Usage>]) -> Self {
        let mut counts = Self {
            inference: 0,
            tools: 0,
            thinking: 0,
            tokens: Usage::new(),
            reported: 0,
        };
        let mut entry = String::new();
        for ((row, step), report) in rows.iter().zip(steps).zip(usage) {
            match step {
                Step::ToolCall => counts.tools += 1,
                Step::Thinking => counts.thinking += 1,
                Step::Model | Step::Boundary | Step::Plain => {}
            }
            if matches!(step, Step::Model | Step::Thinking | Step::ToolCall) {
                let seen = turn_key(&row.key);
                if seen != entry {
                    counts.inference += 1;
                    entry = seen;
                    if let Some(report) = report {
                        counts.fold(report);
                    }
                }
            }
        }
        counts
    }

    /// Fold one counted entry's committed counters into the running sums.
    fn fold(&mut self, report: &Usage) {
        if report.is_empty() {
            return;
        }
        self.reported += 1;
        for (counter, n) in report {
            *self.tokens.entry(counter.clone()).or_default() += n;
        }
    }

    /// The aggregate line in the operator's own phrasing — `9 inference calls
    /// · 14 tool calls · 3 thinking blocks · 3150 output tokens` — with a zero
    /// term left unsaid. The inference term always survives, so the line is
    /// never empty. Token terms state each committed counter's sum under its
    /// own name; **a mixed turn** (some counted entries usage-bearing, some
    /// legacy) suffixes each sum with `+` — *at least this many*, because the
    /// bytes carry no more than the reporting entries said.
    fn say(&self) -> String {
        let terms = [
            (self.inference, "inference call"),
            (self.tools, "tool call"),
            (self.thinking, "thinking block"),
        ];
        let mut said: Vec<String> = terms
            .iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, word)| format!("{n} {word}{}", if *n == 1 { "" } else { "s" }))
            .collect();
        let at_least = if self.reported < self.inference {
            "+"
        } else {
            ""
        };
        for (counter, sum) in &self.tokens {
            if *sum > 0 {
                said.push(format!("{sum}{at_least} {}", counter.replace('_', " ")));
            }
        }
        said.join(TERM_SEP)
    }
}
