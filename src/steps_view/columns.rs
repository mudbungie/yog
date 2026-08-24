//! The Steps list's **column table** (§11): one entry per rendered field,
//! carrying the operator-facing header, the one-line explanation that header
//! hands over on hover, and the cell the field paints for a step.
//!
//! Header and cell share one home for the same reason glyph, hue and words
//! share [`super::render::framing_badge`]: two carriers of one fact drift.
//! Before bl-3ffc the list painted seven bare values in a row — a badge, a
//! number, a number, a number, a sha and two timestamps — and nothing on
//! screen said which was which. A field cannot now be rendered without its
//! name, because the name is the only way to reach the cell.
//!
//! The hover idiom is the `Workspaces:` label's (bl-2d87): plain sentences for
//! an operator meeting the word for the first time — what the column *is*, not
//! how it is computed.

use super::StepSummary;
use super::render::summary_badge;

/// Short-oid width for the per-step commit (matches git's default).
const SHORT_OID: usize = 7;

/// A painted cell — the value plus the weight the column asks for.
pub(super) enum Cell {
    /// The outcome badge: glyph and words in the framing's hue.
    Colored(egui::Color32, String),
    /// A fixed-width value (a sequence number, a sha).
    Mono(String),
    /// A plain figure.
    Plain(String),
    /// A dimmed value (the timestamps).
    Weak(String),
    /// The step never recorded this field — the cell holds the column open so
    /// the row below still reads under its header.
    Empty,
}

/// One column of the step list. `cell` takes the row's selection flag because
/// the ▶ marker rides the Step number rather than taking a nameless column of
/// its own; every other column ignores it.
pub(super) struct Column {
    pub(super) header: &'static str,
    pub(super) hint: &'static str,
    pub(super) cell: fn(&StepSummary, bool) -> Cell,
}

/// Every field the step list paints, left to right.
pub(super) const COLUMNS: &[Column] = &[
    Column {
        header: "Outcome",
        hint: "How this step ended, read back from its own response bytes: \
               complete, failed, no clean end — a kill, a crash and a call still \
               in flight leave the same trace on disk, so the row claims only \
               that it never ended cleanly — or no response at all, which means \
               the step produced nothing and nobody is driving the agent, or \
               the output limit ended the turn, which means the reply framed \
               cleanly but stops where the model ran out of room.",
        cell: |step, _| {
            let (glyph, color, phrase) = summary_badge(step);
            Cell::Colored(color, format!("{glyph} {phrase}"))
        },
    },
    Column {
        header: "Step",
        hint: "Where this step falls in the agent's run — 001 is the first \
               prompt sent, and each later number is one more turn. ▶ marks the \
               step whose records are opened below.",
        cell: |step, selected| {
            let marker = if selected { "▶" } else { " " };
            Cell::Mono(format!("{marker} {}", step.seq))
        },
    },
    Column {
        header: "Attempts",
        hint: "How many tries finished inside this one step. More than one \
               means the model was asked again before the step ended.",
        cell: |step, _| Cell::Plain(step.attempts.to_string()),
    },
    Column {
        header: "Tokens",
        hint: "Tokens this step billed — its prompt plus its output, with a \
               cached slice counted once however its provider reports it. The \
               same figure the budget line spends against the conversation's \
               ceiling.",
        cell: |step, _| Cell::Plain(step.tokens.total_tokens().to_string()),
    },
    Column {
        header: "Commit",
        hint: "The agent branch tip when the step started, shortened — the code \
               and history this step actually saw. Empty when the step wrote no \
               record of itself.",
        cell: |step, _| {
            step.commit
                .as_deref()
                .map_or(Cell::Empty, |commit| Cell::Mono(short(commit)))
        },
    },
    Column {
        header: "Started",
        hint: "When the step began, as the step itself recorded it. Empty when \
               that record is missing or unreadable.",
        cell: |step, _| step.started_at.clone().map_or(Cell::Empty, Cell::Weak),
    },
    Column {
        header: "Ended",
        hint: "When the step finished. Empty while it is still running — and \
               also when it died before it could write the time down, which is \
               why the Outcome column, not this one, says whether it ended well.",
        cell: |step, _| step.ended_at.clone().map_or(Cell::Empty, Cell::Weak),
    },
];

fn short(oid: &str) -> String {
    oid.chars().take(SHORT_OID).collect()
}
