//! S19-T2 affordance-composes-dispatch: each group-card click becomes composer
//! text through one pure function — the judge and synthesizer goals carry every
//! candidate's exact refs (V3.1: handle, attempt tip, base, target), the
//! deliver line awaits the operator's summary, and a cohort with nothing to
//! cite composes nothing.

use crate::science::compose::{Intent, draft};
use crate::science::tests::candidate;
use crate::workdiff::Change;

/// The judge goal is a new-conversation dispatch naming the ask, the
/// obligation, and one citation line per candidate with its exact refs.
#[test]
fn the_judge_goal_carries_every_candidates_exact_refs() {
    let rows = vec![
        candidate("at-1", Some("alpha")),
        candidate("at-2", Some("beta")),
    ];
    let d = draft(&Intent::Judge, &rows).expect("two candidates compose");
    assert!(d.new_conversation);
    assert!(
        d.text.starts_with("Judge the candidate attempts"),
        "{}",
        d.text
    );
    assert!(
        d.text
            .contains("Project proj, ball bl-1, target work/bl-1 at aaaa1111bbbb2222"),
        "{}",
        d.text
    );
    assert!(
        d.text.contains(
            "- candidate at-1: branch attempt/at-1 at at-100tip, forked from base basebase1234beef"
        ),
        "{}",
        d.text
    );
    assert!(
        d.text
            .contains("- candidate at-2: branch attempt/at-2 at at-200tip"),
        "{}",
        d.text
    );
}

/// The synthesizer differs in the ask and nothing else — same refs, same shape.
#[test]
fn the_synthesize_goal_is_the_same_refs_under_a_different_ask() {
    let rows = vec![candidate("at-1", None)];
    let d = draft(&Intent::Synthesize, &rows).expect("a candidate composes");
    assert!(d.new_conversation);
    assert!(d.text.starts_with("Synthesize one result"), "{}", d.text);
    assert!(
        d.text.contains("- candidate at-1: branch attempt/at-1"),
        "{}",
        d.text
    );
}

/// A candidate with no resolved base cites "unknown" rather than inventing a
/// commit; the ordinary claim row (no handle) is never cited.
#[test]
fn an_unknown_base_is_said_and_the_claim_row_is_not_cited() {
    let mut fanned = candidate("at-1", None);
    fanned.base = None;
    let mut claim = candidate("ignored", None);
    claim.diff.handle = None;
    let d = draft(&Intent::Judge, &[claim, fanned]).expect("one candidate composes");
    assert!(d.text.contains("forked from base unknown"), "{}", d.text);
    assert!(!d.text.contains("ignored"), "{}", d.text);
}

/// Nothing to cite composes nothing: no candidates at all, and candidates whose
/// branches did not resolve — both are `None`, never an empty dispatch.
#[test]
fn nothing_to_cite_composes_nothing() {
    assert_eq!(draft(&Intent::Judge, &[]), None);
    let mut unresolved = candidate("at-1", None);
    unresolved.diff.change = Change::Unreadable;
    assert_eq!(draft(&Intent::Judge, &[unresolved.clone()]), None);
    assert_eq!(draft(&Intent::Synthesize, &[unresolved]), None);
}

/// The deliver line stops at the handle and a trailing space — the summary is
/// the operator's statement of what landed, and yog does not invent one. The
/// retire line is complete as composed.
#[test]
fn deliver_awaits_a_summary_and_retire_is_complete() {
    let rows = vec![candidate("at-1", None)];
    let deliver = draft(
        &Intent::Deliver {
            handle: "at-1".to_owned(),
        },
        &rows,
    )
    .expect("deliver composes");
    assert_eq!(deliver.text, "/deliver at-1 ");
    assert!(!deliver.new_conversation);
    let retire = draft(
        &Intent::Retire {
            handle: "at-1".to_owned(),
        },
        &rows,
    )
    .expect("retire composes");
    assert_eq!(retire.text, "/retire at-1");
    assert!(!retire.new_conversation);
}
