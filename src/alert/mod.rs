//! **The attention strip, escalated to the desktop** (DESIGN §6 as amended,
//! bl-e160). §6 is yog's core promise — *does anything need me?* — and it is
//! invisible whenever the window is buried or minimized. When a new thing needs
//! you and you are not looking at yog, the desktop says so.
//!
//! **No second attention model.** An alert is a projection of
//! [`QueueRow`](crate::boundary::answer::queue::QueueRow) — the §6 decision
//! queue, which is already "what needs you" made addressable (VISION §5 V5.2).
//! The predicate, the roster order and the wording are all read from there, so
//! a rule the strip counts and a rule the desktop announces cannot diverge.
//!
//! **Dedupe rides the acknowledgement that already exists.** A row leaves the
//! queue exactly when the operator acknowledges it — focusing the conversation
//! (§6) or `/seen` (§8.5) — so the alert set is the queue set, and "announce
//! what is new" is a set difference against what this window last saw. Nothing
//! is stored, no watermark is written (an alert is *render output*, never a
//! mutation — I7 untouched), and a signal that re-arms because a ref moved
//! re-arms here too, for free, because a moved ref puts the row back.
//!
//! **The identity is the row's own sentence.** A conversation is announced once
//! per *set of firing rules*, not once per frame and not once per rule: a
//! second rule firing on the same conversation is a changed sentence and says
//! itself, while the same sentence again is the same unanswered ask. That is
//! the flag's behaviour too — it stays raised, it does not re-raise.
//!
//! **Nothing at boot.** A window that has just opened has witnessed no
//! *arrival*: everything already waiting was waiting before it existed. So the
//! first fold is the baseline and announces nothing —
//! [`Announced::arrivals`]'s general path with no prior observation, not a
//! first-run branch.

/// Handing an alert to the desktop — the one spawn, and the argv it becomes.
pub mod send;

use crate::attention::AttentionKind;
use crate::boundary::answer::queue::QueueRow;
use std::collections::BTreeSet;

/// One thing to tell the desktop about (§6): which conversation, in which
/// workspace, and why it is asking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    /// The notification's title: the workspace wall and the conversation in it.
    pub summary: String,
    /// Why it is asking — the firing §6 rules, in words.
    pub body: String,
}

impl Alert {
    /// What makes this the *same* ask as one already announced. Derived from
    /// the two rendered halves rather than from a parallel encoding of them, so
    /// there is no second representation to drift: if the operator would read a
    /// different sentence, it is a different ask.
    fn key(&self) -> String {
        format!("{}\n{}", self.summary, self.body)
    }
}

/// The alerts one decision queue implies — one per row, in queue order.
///
/// A row whose signals somehow list none is dropped rather than announced with
/// an empty reason: the queue holds only attention-bearing rows, so this cannot
/// arise from the derivation, and a silent alert would say nothing anyway.
pub fn of_queue(rows: &[QueueRow]) -> Vec<Alert> {
    rows.iter()
        .filter(|row| !row.signals.is_empty())
        .map(|row| Alert {
            summary: format!("{} · {}", wall_of(row), row.display),
            body: rules(&row.signals),
        })
        .collect()
}

/// The §3.1 leaf naming the row's wall — what the operator calls that sphere.
/// A workspace path with no leaf is its own whole path, the same floor every
/// other seat uses rather than an invented placeholder.
fn wall_of(row: &QueueRow) -> String {
    row.workspace.file_name().map_or_else(
        || row.workspace.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// The firing rules as one sentence, in the badge order the queue lists them.
fn rules(signals: &[AttentionKind]) -> String {
    signals
        .iter()
        .map(|k| k.says())
        .collect::<Vec<_>>()
        .join("; ")
}

/// **The whole decision**, in one pure place: what this pass should hand the
/// desktop, given the §6 queue, what this window has already said, whether the
/// window has focus, and the §4.1 knob.
///
/// The fold runs unconditionally and the *announcing* is what the two gates
/// suppress — so a signal that landed while the operator was looking at yog, or
/// while the knob was off, is absorbed into the baseline rather than saved up.
/// Losing focus must not replay the news you already had.
pub fn announce(
    announced: &mut Announced,
    rows: &[QueueRow],
    focused: bool,
    enabled: bool,
) -> Vec<Alert> {
    let arrived = announced.arrivals(of_queue(rows));
    if focused || !enabled {
        return Vec::new();
    }
    arrived
}

/// What this window has already told the desktop (§5.3 RAM — viewport
/// ephemera, per instance and never durable: two windows each own their own
/// desktop, and a restart is a new window, not a missed one).
#[derive(Debug, Default)]
pub struct Announced {
    /// The last observed alert set, or `None` before the first observation —
    /// which is what makes a freshly-opened window announce nothing.
    baseline: Option<BTreeSet<String>>,
}

impl Announced {
    /// Fold this pass's alerts in and return the ones that **arrived** since the
    /// last fold.
    ///
    /// The baseline advances whether or not the caller goes on to announce
    /// them, and that is the point: a signal that landed while the window had
    /// focus — or while the knob was off — was not missed, so it must not be
    /// announced later as though it had just arrived.
    fn arrivals(&mut self, alerts: Vec<Alert>) -> Vec<Alert> {
        let keys: BTreeSet<String> = alerts.iter().map(Alert::key).collect();
        let arrived = match self.baseline.take() {
            None => Vec::new(),
            Some(before) => alerts
                .into_iter()
                .filter(|a| !before.contains(&a.key()))
                .collect(),
        };
        self.baseline = Some(keys);
        arrived
    }
}

#[cfg(test)]
mod tests;
