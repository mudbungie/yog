//! Which failures are still **live** — the ambient error surface's retirement
//! rule (DESIGN §6, §4.2, §11).
//!
//! `ops.jsonl` is append-only and complete: a failure never leaves it unless
//! the operator asks for a fresh trail ([`super::clear`]). But a
//! failure a later clean run of the same verb has superseded is *history*, not
//! a live wound, and painting it with the same prominence sends diagnosis down
//! a false trail — the proven wound: a three-day-old `litany prime` failure,
//! since fixed and re-run green, read as THE error when an unrelated action
//! failed.
//!
//! So prominence is a **projection over the tail at read time, never a stored
//! flag**: walking newest-first, a failed row is live unless a *later* row with
//! the same (`cwd`, verb) did not fail. The verb is the leading two argv tokens
//! — binary plus subcommand (`bl close`, `litany prime`, `yog-step mint`) —
//! because the argv tail carries per-run operands (a ball id, a composed goal)
//! that never repeat, so keying on the whole argv would retire nothing; `cwd`
//! scopes it, so a clean `bl close` in one project leaves a failed one in
//! another alone. Success is the pane's own classifier negated
//! ([`OpRow::failed`]), so no second definition of success can drift from the
//! one the surface paints.
//!
//! A retired failure keeps its row and its ⚠ in the expanded accessory — it
//! loses only ichor and the chip's count. **Absence of a live failure is the
//! record; the log is the history.**

use std::collections::HashSet;

use super::OpRow;

/// What a row *is* at read time: the retirement rule's three outcomes, and the
/// one fact both §11 activity seats paint (the chip's failure count, the
/// per-row marker). Derived by [`outcomes`], never stored — and its glyph, hue
/// and **words** have a single home, `theme::op_badge`, so no seat invents its
/// own spelling of "failed" (DESIGN §11, the badge-seat pattern).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpOutcome {
    /// The op exited clean.
    Clean,
    /// It failed and nothing has re-run its verb clean since — a live wound.
    Failed,
    /// It failed, but a later clean run of the same verb retired it (§6).
    Retired,
    /// A detached spawn that handed off, with nothing said against it yet
    /// (`OpRow::detached`, bl-8433). Neither `Clean` nor `Failed`: nobody
    /// observed an exit, so it must not read as one — but it must not vanish
    /// into the failure count either, hence its own bucket. It **does**
    /// retire an earlier failure of the same verb exactly like `Clean` does
    /// (§6): the handoff is the newest fact about that verb in this `cwd`,
    /// and a stale failure under it is no longer the live story.
    /// **Since bl-b95e it is the whole of a live handoff.** A fifth bucket
    /// stood here — `Notice`, a driver whose sink held nothing but litany's own
    /// benign lines (bl-1296) — because the sink was folded in unconditionally
    /// and a byte in it meant death. The fold is now gated on the state the
    /// launch produced (`opslog::launch::stillborn`), so a driver that filed a
    /// notice and carried on is a handoff like any other and needs no bucket of
    /// its own to be spared the alarm.
    Detached,
}

/// The §11 activity-accessory summary — the demoted ops pane's collapsed chip:
/// how many ops the tail holds, how many are **live** failures (retired ones
/// excluded, per this module's rule), and how many are **drift** observations
/// (§7.2: a change nobody announced). Pure over the rows the pane would paint,
/// so chip and expansion never diverge — and `drifts` is a *query over the tail*,
/// never a counter yog keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Activity {
    pub total: usize,
    pub errors: usize,
    pub drifts: usize,
}

/// Summarize the tail for the collapsed chip (§11): every op counted, the live
/// failures ([`outcomes`]) counted, the drift rows ([`OpRow::drift`]) counted on
/// their own axis.
///
/// **The two alarm axes stop at the operator's ack** (bl-c417,
/// [`super::since_ack`]): `errors` and `drifts` are counted over the rows after
/// the newest ack line only, so a dismissal quiets the chip's ichor exactly as
/// it quiets the §7.3 banners — one watermark, read by both. Drift is quieted
/// with them deliberately: §7.2 files it as an *alarm* ("its catches are
/// alarms, not routine"), and an alarm the operator has said they have seen is
/// what an ack is for.
///
/// `total` is **not** quieted — it counts the whole tail, because it names the
/// rows the expansion renders and an ack removes none of them. A chip that said
/// fewer ops than the pane below it lists would be the one number the operator
/// could catch lying.
pub fn activity(rows: &[OpRow]) -> Activity {
    let live = super::since_ack(rows);
    Activity {
        total: rows.len(),
        errors: outcomes(live)
            .into_iter()
            .filter(|o| *o == OpOutcome::Failed)
            .count(),
        drifts: live.iter().filter(|r| r.drift()).count(),
    }
}

impl Activity {
    /// **Whether anything on the trail is still alarming** — the §11 predicate
    /// that decides whether the ops pane offers its Dismiss at all (a control
    /// that would write a line and change nothing is a control that should not
    /// be there), and the same test the chip's ichor is painted by.
    ///
    /// It lives on the summary rather than beside the pane (bl-296f, moved off
    /// `AppModel`) because it is a reading of these two counts and nothing
    /// else: the button and the ichor cannot come apart if there is one home
    /// for what "alarming" means.
    pub fn alarming(&self) -> bool {
        self.errors > 0 || self.drifts > 0
    }

    /// The chip label: `activity · N ops`, with `· M failed ⚠` appended when
    /// live failures are present (the shell paints that count in ichor) and
    /// `· K drift` when the tail holds drift observations. Drift is its own
    /// word, not a ⚠: it accuses the watcher, not the operator's last action.
    ///
    /// §11 glyph doctrine: the chip has room, so it says the outcome **outright**
    /// — and takes both the word and the glyph from the one badge mapping
    /// (`badge::op_badge`), so the chip can never spell an outcome differently from the rows
    /// it summarizes.
    pub fn chip(&self) -> String {
        let mut parts = vec![format!("activity · {} ops", self.total)];
        if self.errors > 0 {
            let (glyph, phrase) = crate::badge::op_badge(OpOutcome::Failed);
            parts.push(format!("{} {phrase} {glyph}", self.errors));
        }
        if self.drifts > 0 {
            parts.push(format!("{} drift", self.drifts));
        }
        parts.join(" · ")
    }
}

/// One [`OpOutcome`] per row, positionally aligned with `rows`: a failed row
/// ([`OpRow::failed`]) is `Failed` unless a *later* row with the same
/// [`verb_key`] retired it (ran clean or handed off — see
/// [`OpOutcome::Detached`]), which makes it `Retired`; a handoff with nothing
/// said against it is `Detached`; everything else is `Clean`. Both of those
/// mark the verb retired going forward (bl-8433: a handoff is the newest fact
/// about the verb, same as a clean run). The whole retirement rule lives here;
/// nothing is stored.
pub fn outcomes(rows: &[OpRow]) -> Vec<OpOutcome> {
    let mut retired: HashSet<(String, String)> = HashSet::new();
    let mut out: Vec<OpOutcome> = rows
        .iter()
        .rev()
        .map(|row| {
            if row.failed() {
                return if retired.contains(&verb_key(row)) {
                    OpOutcome::Retired
                } else {
                    OpOutcome::Failed
                };
            }
            retired.insert(verb_key(row));
            if row.detached() {
                OpOutcome::Detached
            } else {
                OpOutcome::Clean
            }
        })
        .collect();
    out.reverse();
    out
}

/// The identity a retirement keys on: the row's `cwd` and its **verb** — the
/// leading two tokens of the joined argv, i.e. binary plus subcommand. The
/// operand tail is deliberately dropped (see the module note).
fn verb_key(row: &OpRow) -> (String, String) {
    let verb = row
        .argv
        .split_whitespace()
        .take(2)
        .collect::<Vec<&str>>()
        .join(" ");
    (row.cwd.clone(), verb)
}

#[cfg(test)]
mod tests;
