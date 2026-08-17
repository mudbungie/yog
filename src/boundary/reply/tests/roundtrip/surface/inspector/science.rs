//! The §3.9 projection's fixture (bl-40ab) — split from the inspector family's
//! file at §12's budget on the seam its own doc draws: the projection *composes*
//! the work-diff row, so its rows carry every one of `workdiff`'s arms plus the
//! outcome arms and the five optional columns, present and absent.

use crate::science::{Attempt, Outcome, Verdict};

/// One row per [`Outcome`] arm, the first with every optional column populated
/// and every later one bare — so a decoder that dropped an absence or a value
/// fails on the row that carries the other reading.
pub(super) fn science() -> Vec<Attempt> {
    let diffs = super::workdiff::attempts();
    [
        Outcome::Accepted {
            commit: "ccc".to_owned(),
        },
        Outcome::Rejected {
            by: Some("at-0badcafe".to_owned()),
        },
        Outcome::Rejected { by: None },
        Outcome::Reworked,
        Outcome::Pending,
    ]
    .into_iter()
    // `cycle` rather than an index fallback: the outcome list is longer than the
    // diff list on purpose, and a fallback row would be a fixture nothing
    // reaches — dead weight the coverage floor would refuse.
    .zip(diffs.into_iter().cycle())
    .enumerate()
    .map(|(i, (outcome, diff))| row(diff, i, outcome))
    .collect()
}

/// Row `i`: the first carries every optional column and a verdict, the rest
/// carry none — the two readings a decoder must keep apart.
fn row(diff: crate::workdiff::Attempt, i: usize, outcome: Outcome) -> Attempt {
    let full = i == 0;
    let some = |text: &str| full.then(|| text.to_owned());
    Attempt {
        diff,
        base: some("f00dbeef"),
        conversation: some("20260815T101112Z-abcd1234"),
        goal: some("ship it"),
        pins: if full {
            vec!["instructions/00-AGENTS.md=/p/AGENTS.md".to_owned()]
        } else {
            Vec::new()
        },
        governing: some("deadbeef"),
        usage: super::super::spend(),
        wall_secs: 90,
        steps: 4,
        response: some("done, tests green"),
        verdicts: if full {
            vec![Verdict {
                sender: "judge-one".to_owned(),
                body: "candidate B reads cleaner".to_owned(),
            }]
        } else {
            Vec::new()
        },
        // The full row is a compacted record saying so; the rest are intact,
        // whose encoding omits the column (bl-fde5).
        compacted: if full { 12 } else { 0 },
        outcome,
    }
}
