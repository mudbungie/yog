//! The **floor** (VISION §4.9's fifth rung, bl-94b4): a per-conversation park
//! over a whole descent, and the one pair it does not reach.

use super::{row, ruled};
use crate::control::classify::Effect;
use crate::control::judge::{Answers, Ruling};
use crate::control::policy::Policy;
use crate::opslog::YOG_CONTROL;

#[test]
fn a_floor_holds_every_class_above_read_across_the_whole_subtree() {
    let answers = Answers::fold(&[row(&[YOG_CONTROL, "floor", "amber", "raise"])]);
    assert!(answers.floored("amber"));
    assert!(
        answers.floored("amber-1-2"),
        "the descent prefix carries it"
    );
    assert!(
        !answers.floored("amberine"),
        "a longer name is another agent"
    );
    assert!(!answers.floored("other"));
    assert_eq!(
        ruled(&answers, "t", "amber", "bash", Effect::Read),
        Ruling::Pass
    );
    assert_eq!(
        ruled(&answers, "t", "amber", "bash", Effect::TargetWrite),
        Ruling::Hold
    );
    assert_eq!(
        ruled(&answers, "t", "amber-1", "bash", Effect::Process),
        Ruling::Hold
    );
    // The floor raises; it never lowers.
    assert_eq!(
        ruled(&answers, "t", "amber", "bash", Effect::Secret),
        Ruling::Refuse
    );
    // And a once-answer to this exact invocation still wins over it.
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "floor", "amber", "raise"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass"]),
    ]);
    assert_eq!(
        ruled(&answers, "toolu_1", "amber", "bash", Effect::TargetWrite),
        Ruling::Pass
    );
}

/// **bl-a821**: `revoke` reaches a conversation's descendants and the
/// compactor is one, so the floor held `write_summary` — the operator was
/// queued a machinery act they have no basis to judge, and until they answered
/// it the floored conversation could not compact.
#[test]
fn a_floor_does_not_reach_the_compactor_s_checkpoint_pair() {
    let answers = Answers::fold(&[row(&[YOG_CONTROL, "floor", "amber", "raise"])]);
    for pair in ["write_summary", "mark_for_deletion"] {
        assert_eq!(
            ruled(&answers, "t", "amber", pair, Effect::TargetWrite),
            Ruling::Pass,
            "{pair} is the compaction procedure's own act, not the agent's"
        );
        assert_eq!(
            ruled(&answers, "t", "amber-1", pair, Effect::TargetWrite),
            Ruling::Pass,
            "and the compactor of a floored conversation is a descendant"
        );
    }
    // The exemption is the pair's and the floor's alone: every other target
    // write under the same floor still holds, and the workspace's own table
    // still rules the pair.
    assert_eq!(
        ruled(&answers, "t", "amber", "load_skill", Effect::TargetWrite),
        Ruling::Hold
    );
    assert_eq!(
        answers.ruling(
            "t",
            "amber",
            "write_summary",
            Effect::TargetWrite,
            &Policy::parse("table:\n  target-write: hold\n")
        ),
        Ruling::Hold
    );
}

#[test]
fn a_lowered_floor_stops_binding() {
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "floor", "amber", "raise"]),
        row(&[YOG_CONTROL, "floor", "amber", "lower"]),
    ]);
    assert!(!answers.floored("amber"));
    assert_eq!(
        ruled(&answers, "t", "amber", "bash", Effect::Process),
        Ruling::Pass
    );
}
