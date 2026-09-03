//! **What an ops row stands at** (DESIGN §7.3, §6, §4.2) — the one fold that
//! makes the failure banner an *answer* rather than a derivation every seat
//! re-implements.
//!
//! §7.3 rules that a failed action is a stated fact, said exactly once, at the
//! surface its `origin` names, and that its alarm ends two ways: **retirement**
//! by a newer clean op of the same verb (§6, [`super::outcomes`]) or the
//! operator's **ack** watermark ([`super::since_ack`]). Both readings lived
//! here and neither crossed the §8.5 boundary, so a seat wanting the banner had
//! to re-derive the retirement key, the ack scan and the sentinel table — five
//! duplications whose failure mode is silent divergence rather than a compile
//! error (bl-4d81).
//!
//! [`Standing`] is those two readings folded into one **total** vocabulary, and
//! [`standings`] is the projection that produces it. "One banner per origin" is
//! then the rows whose standing is [`Standing::Live`], grouped by the `origin`
//! the row already carries — a seat renders and never classifies.
//!
//! **Total, not three-valued, and never absent.** §7.3's three words —
//! live / retired / acked — describe a *failure*'s standing; a clean row and a
//! handoff have one too, and leaving the field off for them would make a reader
//! tell "ran clean" from "handed off, no exit observed" by re-reading the `exit`
//! integer, which is the whole defect. So the vocabulary is [`OpOutcome`]'s
//! four arms — the crate's one classification, the same one `badge::op_badge`
//! is keyed on — with the ack's [`Standing::Acked`] added to split the failed
//! one. Nothing is stored: this is a read over a tail, like everything else in
//! [`super::live`].

use super::{OpOutcome, OpRow, outcomes, since_ack};

/// Where one row of the trail stands **right now** — its §6 outcome folded with
/// the §4.2 ack watermark. Total over every row, so a reader never has to ask
/// the `exit` field anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// The op ran clean ([`OpOutcome::Clean`]).
    Clean,
    /// A detached spawn that handed off with nothing said against it
    /// ([`OpOutcome::Detached`]) — neither clean nor failed, because nobody
    /// observed an exit.
    Detached,
    /// A failure nothing has superseded and the operator has not acked: **the
    /// live wound**, and the only standing that banners (§7.3).
    Live,
    /// A failure a later clean run — or handoff — of the same verb retired
    /// (§6). It keeps its row and its ⚠; it loses prominence.
    Retired,
    /// A failure the operator has **seen**: it lies at or before the newest
    /// `ack-failures` line, so every failure-derived alarm passes over it
    /// (bl-c417). Not amnesia — a newer failure lands after the watermark and
    /// stands [`Live`](Self::Live).
    Acked,
}

impl Standing {
    /// The wire's word for this standing — the boundary codec's one spelling,
    /// beside the enum it names so the two cannot drift.
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Detached => "detached",
            Self::Live => "live",
            Self::Retired => "retired",
            Self::Acked => "acked",
        }
    }
}

/// One row of the trail **as a reader is answered it** (§8.5): the durable line
/// and what it stands at. The row's own per-row readings — `failed`,
/// `exit_label` — stay methods on [`OpRow`], because they are recomputable from
/// the line itself and a second copy of a derivation is what this crate keeps
/// refusing to grow; the standing cannot be, since it is a fact about the row's
/// *position* in the tail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpView {
    pub row: OpRow,
    pub standing: Standing,
}

/// The tail with each row's [`Standing`] beside it — the §7.3 carrier, and the
/// one composition of §6's retirement projection with §4.2's ack watermark.
///
/// **A prefix may be sliced off before this runs.** Retirement looks only at
/// rows *later* than the one it judges, and an ack line dropped with the prefix
/// leaves every remaining row after it, which is what those rows already were —
/// so answering the last `max` of a longer tail gives each row the standing it
/// had over the whole of it ([`since_ack`] states the same argument).
pub fn standings(rows: &[OpRow]) -> Vec<OpView> {
    let unacked = since_ack(rows).len();
    let seen = rows.len().saturating_sub(unacked);
    rows.iter()
        .zip(outcomes(rows))
        .enumerate()
        .map(|(i, (row, outcome))| OpView {
            row: row.clone(),
            standing: match outcome {
                OpOutcome::Clean => Standing::Clean,
                OpOutcome::Detached => Standing::Detached,
                OpOutcome::Retired => Standing::Retired,
                OpOutcome::Failed if i < seen => Standing::Acked,
                OpOutcome::Failed => Standing::Live,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests;
