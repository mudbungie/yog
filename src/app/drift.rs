//! What the derivation FOUND about its own fidelity — the §7.2 instrumentation
//! (bl-49f4, extended bl-ee0a).
//!
//! Drift is **divergence between what the frame renders and what is on disk**.
//! There are two ways to get it, and this module names both:
//!
//! 1. *A change nobody announced* — the watcher was armed and silent while disk
//!    moved. The 15 s full sweep exists because a filesystem event can be lost;
//!    that made it self-healing over a defect nobody could see, so **a sweep
//!    that catches something is evidence of a bug** and is written down.
//! 2. *A derivation that arrived late* (bl-ee0a). Since the derivation moved off
//!    the frame thread there is no longer a structural claim that the rendered
//!    snapshot is this instant's disk — it is the last *completed* pass. When a
//!    pass takes longer than the poll cadence it is meant to keep, everything
//!    rendered meanwhile was that far behind, and that is drift too: the same
//!    divergence, a different cause. It is named rather than hidden, because the
//!    thing it used to do instead was freeze the window (bl-ee0a).
//!
//! The signal is almost free, because [`Mark`](crate::watch::Mark) already rides
//! every dirty root: a re-derivation that *changes* a snapshot is only
//! interesting in the light of what claimed the root had changed. Under
//! `Mark::Watch` it is the watcher working. Under `Mark::Poll` it is the
//! liveness re-probe, for which no filesystem event exists at all. Under
//! `Mark::Sweep` **nothing announced it** — that is a dropped event, measured at
//! the one moment it costs something.
//!
//! Nothing here is stored. A [`Drift`] lives for one tick and is folded into
//! `ops.jsonl` (§4.2) — yog's existing durable, two-instance-shared trail — as
//! one line per kind. The operator's count is then a *query* over that tail
//! ([`crate::opslog::activity`]), reachable at the §11 activity accessory with
//! no debugger, no fourth surface, and no counter that could drift from what
//! actually happened.

use crate::opslog::OpEntry;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The §11 ops-surface staleness line, or `None` while the snapshot is fresh
/// (the normal case, and the one that must render nothing at all). Pure over
/// the age so the threshold is provable without a slow machine.
///
/// `stale_after` is the live cadence's bound
/// ([`Cadence::stale_after`](super::Cadence::stale_after), bl-3381): twice the
/// full-sweep period — the worker re-stamps the snapshot on every full sweep
/// even when nothing changed, so exceeding two of them means passes are not
/// completing, not that the world is quiet.
pub(super) fn stale_label(age: Duration, stale_after: Duration) -> Option<String> {
    (age >= stale_after).then(|| format!("derivation {} s behind", age.as_secs()))
}

/// A pass's own lateness, or `None` when it kept its cadence (§7.2). `started`
/// and `finished` come from the injected clock, so the late branch is reachable
/// in a test without a slow machine. `late_pass` is the live cadence's bound
/// for **the sweep this pass ran**
/// ([`Cadence::late_pass`](super::Cadence::late_pass), bl-3381, bl-4b28): the
/// period that pass promised to keep, so a pass that eats its whole interval
/// has already failed to keep it — not a threshold picked for feel, the
/// schedule's own period.
///
/// Whether a *finding* is written is the caller's edge test, not this one's
/// (bl-4b28): this answers "was this pass late", which is a fact about one
/// pass; the trail records the transition into lateness.
pub(super) fn lateness(
    started: std::time::Instant,
    finished: std::time::Instant,
    late_pass: Duration,
) -> Option<u64> {
    let took = finished.saturating_duration_since(started);
    (took >= late_pass).then_some(took.as_secs())
}

/// The finding a pass's own lateness earns, given whether the pass before it
/// was late (§7.2, bl-4b28) — the edge test [`Drift::Late`] is written on.
///
/// Four cases, one line: a pass that misses after one that kept cadence is the
/// event ("passes stopped keeping cadence"); a pass that misses after one that
/// already missed adds nothing a reader did not have; and a pass that keeps
/// cadence is not a finding at either end. Recovery writes nothing on purpose —
/// the trail is a record of what went wrong, and *is it late now* is the §11
/// staleness line's derived answer, not a row anyone has to find.
pub(super) fn late_edge(secs: Option<u64>, was_late: bool) -> Option<u64> {
    secs.filter(|_| !was_late)
}

/// One thing a sweep or the watch backend found that the watcher should have
/// announced and did not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Drift {
    /// The backend announced that it **lost** events under this root: inotify
    /// `IN_Q_OVERFLOW`, or a watch it could not arm mid-tree (descriptor
    /// exhaustion). A real drop, and the only class the kernel tells us about —
    /// yog re-derives the root at once rather than waiting on the sweep.
    Desync(PathBuf),
    /// A full-sweep re-derivation **changed** this root's snapshot, and nothing
    /// had marked it. The watcher was armed and silent while disk moved: a
    /// dropped event, caught by the backstop it was meant to make unnecessary.
    Unannounced(PathBuf),
    /// A sweep's re-enumeration found this workspace appearing or vanishing,
    /// and no enumeration-root event announced it (§7.1 NamesRoot /
    /// WorkspacesRoot).
    Unenumerated(PathBuf),
    /// Passes **stopped keeping cadence**: this one took `secs`, at or past the
    /// period it promised, so the frame rendered a snapshot that old while it
    /// ran (§7.2, bl-ee0a). Attributed to the yog state root: the observation is
    /// yog's about itself, and it is the root the `ops.jsonl` it lands in lives
    /// under.
    ///
    /// **An edge, not a level** (bl-4b28). It is raised on the pass that first
    /// misses, and not again until one keeps cadence in between — a permanently
    /// late derivation is one event with a timestamp the operator can correlate
    /// against what changed, never a row per sweep. *Are* passes late right now
    /// is a different question with a different answer already: the §11
    /// staleness line, derived from the published snapshot's age
    /// ([`stale_label`]). A state belongs to a query; the trail records events.
    Late(PathBuf, u64),
}

impl Drift {
    /// The `argv[1]` kind token — the drift's name on the ops surface.
    fn kind(&self) -> &'static str {
        match self {
            Drift::Desync(_) => "desync",
            Drift::Unannounced(_) => "unannounced",
            Drift::Unenumerated(_) => "unenumerated",
            Drift::Late(..) => "late",
        }
    }

    /// The root the drift is attributed to.
    fn root(&self) -> &Path {
        match self {
            Drift::Desync(p)
            | Drift::Unannounced(p)
            | Drift::Unenumerated(p)
            | Drift::Late(p, _) => p,
        }
    }

    /// The drift's `stderr` line: the root it names, plus — for a late pass —
    /// how late. The duration rides the attribution line rather than earning a
    /// field, because `ops.jsonl` has no schema to grow and the expandable row
    /// already shows this text (§4.2).
    fn line(&self) -> String {
        match self {
            Drift::Late(root, secs) => format!("{} ({secs} s pass)\n", root.display()),
            other => format!("{}\n", other.root().display()),
        }
    }
}

/// Fold a tick's findings into `ops.jsonl` lines (§4.2): **one line per kind**,
/// with every root it names newline-joined in `stderr`.
///
/// Per-kind rather than per-root on purpose. A systematic drift source affects
/// every workspace at once, and a line each would flood the 256-line tail the
/// §11 accessory reads — burying the evidence under itself. One line per kind
/// bounds a sweep's output to three lines however wide the damage, while the
/// attribution stays complete in the field the accessory already expands.
///
/// `ts` is the caller's wall-clock stamp (the injected clock mints it, §4.2 —
/// this module reads no time) and `cwd` the yog state root the observation was
/// made from.
pub(super) fn entries(ts: &str, cwd: &str, found: &[Drift]) -> Vec<OpEntry> {
    let mut sorted: Vec<&Drift> = found.iter().collect();
    sorted.sort();
    sorted.dedup();
    let mut out: Vec<OpEntry> = Vec::new();
    for drift in sorted {
        let line = drift.line();
        match out.last_mut() {
            Some(entry) if entry.argv.last().map(String::as_str) == Some(drift.kind()) => {
                entry.stderr.push_str(&line);
            }
            _ => out.push(OpEntry::drift(
                ts.to_string(),
                drift.kind(),
                cwd.to_string(),
                line,
            )),
        }
    }
    out
}

#[cfg(test)]
mod tests;
