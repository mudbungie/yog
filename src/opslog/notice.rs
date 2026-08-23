//! Which detached-driver stderr is an **operator notice** rather than a death
//! (DESIGN §4.2, §13.3, bl-1296) — the third reading of a `-2` row's folded
//! tail, beside "said nothing" and "died in the handoff".
//!
//! A `-2` row carries no observed exit, so [`OpRow::failed`](super::OpRow::failed)
//! had to substitute something for one, and what it substituted was *"the
//! driver said anything at all"*. That is the wrong reading of lernie's own
//! contract. A detached driver's stderr is where it states what it **declined**
//! — a compaction landing declined or superseded, a retarget declined, a launch
//! that fell into the accepted crash class, a crashed tool window settled, a §6
//! budget stop — and lernie ARCH pins that file as the channel those lines are
//! *"addressed to an operator"* on, every one of them printed on a path that
//! returns `Ok(())`. Worse, the sink is append-only for the driver's whole life
//! and [`fold`](super::detached::fold) re-reads its tail on every sweep, so one
//! benign line held the newest row of its origin in ichor — §7.3 banner, ⚠ chip
//! — until the operator acked it.
//!
//! **Deliberately narrow, and it fails toward alarming.** The shape and the
//! discipline are [`looks_config`](crate::config_edit::fault::looks_config)'s
//! and [`looks_auth`](crate::login::auth::looks_auth)'s: a case-insensitive
//! substring table over text somebody else wrote, holding only phrases that
//! belong to this class and to nothing else. Two rules keep the error on the
//! loud side. A line must carry lernie's own `lernie: ` prefix **and** a marker;
//! and the whole tail must be notices — **one unrecognized line makes the row a
//! failure again**, because a driver that files a notice and then dies has died,
//! and silence is the one failure mode this classifier must never have.
//!
//! **The phrase table is the fragile part and is meant to be temporary.** It is
//! keyed on sentences lernie is free to reword, which is exactly the fragility
//! `config_edit::fault` records about its own markers. An upstream lernie ball
//! asks for a stable `lernie: notice:` prefix stamped on every line of this
//! class; when that lands the table collapses to the one marker and the phrases
//! go. Until then a reworded line reads as a failure, which is the safe way for
//! this to break.

/// The prefix lernie writes ahead of every driver line. Structural rather than
/// a phrase, so a marker can never fire on text some other tool put in the sink.
const LERNIE: &str = "lernie: ";

/// Case-insensitive markers of an **operator-notice** line — one per benign
/// class the driver reports on stderr, each a phrase lernie writes for that
/// class and for nothing else.
///
/// Each is a *fragment* of the line it belongs to, cut short of the ARCH
/// section reference the line ends with: a section mark in a string is a
/// citation the operator would end up reading, which the crate forbids, and
/// the fragment is no wider without it.
///
/// `; the branch continues` is the tail of all three *declined-but-carried-on*
/// lines (compaction landing declined, compaction landing superseded, retarget
/// declined) — it is the sentence that says the decline was not fatal, so one
/// marker covers them without widening to `compaction`, a word a real failure
/// could also carry. `(accepted crash class` is lernie's own name for a launch
/// error it recorded and continued past. The other two are whole phrases of
/// their line: the crashed-tool-window settlement, and the §6 budget stop.
const NOTICE_MARKERS: &[&str] = &[
    "; the branch continues",
    "(accepted crash class",
    "settling a crashed tool window",
    "; stopping (arch ",
];

/// Is `tail` **entirely** operator notices?
///
/// Every non-blank line must be one ([`is_notice`]), and there must be at least
/// one — an empty tail is a driver that has said nothing, which is
/// [`OpRow::detached`](super::OpRow::detached)'s fact and not this one. A tail
/// that mixes a notice with anything unrecognized answers `false`: the notice
/// does not vouch for the line beside it.
pub fn looks_notice(tail: &str) -> bool {
    let mut lines = tail
        .lines()
        .filter(|line| !line.trim().is_empty())
        .peekable();
    lines.peek().is_some() && lines.all(is_notice)
}

/// One line: lernie's [`LERNIE`] prefix plus a [`NOTICE_MARKERS`] hit. Pure and
/// case-insensitive, the same shape as the two sibling classifiers.
fn is_notice(line: &str) -> bool {
    let lower = line.trim_start().to_ascii_lowercase();
    lower.starts_with(LERNIE) && NOTICE_MARKERS.iter().any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests;
