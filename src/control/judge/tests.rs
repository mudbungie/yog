//! The judgment fold: the shipped table, the once-answer, and the floor.

use super::*;
use crate::opslog::Origin;

/// A `yog-control` ops row.
fn row(words: &[&str]) -> OpEntry {
    OpEntry {
        ts: "TS".to_owned(),
        argv: words.iter().map(|s| (*s).to_owned()).collect(),
        cwd: String::new(),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::World,
    }
}

#[test]
fn the_shipped_table_passes_everything_but_loss_and_credentials() {
    // The four classes that are the job pass, open-world among them; only
    // irreversible loss and credential access decline in band.
    assert_eq!(Table::ruling(Effect::Read), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::TargetWrite), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::Process), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::OpenWorld), Ruling::Pass);
    assert_eq!(Table::ruling(Effect::Destructive), Ruling::Refuse);
    assert_eq!(Table::ruling(Effect::Secret), Ruling::Refuse);
}

#[test]
fn a_ruling_spells_itself_the_same_way_both_directions() {
    for ruling in [Ruling::Pass, Ruling::Hold, Ruling::Refuse] {
        assert_eq!(Ruling::of(ruling.word()), Some(ruling));
    }
    assert_eq!(Ruling::of("maybe"), None);
}

#[test]
fn a_verdict_carries_the_reason_except_a_pass() {
    // litany's parser rejects a pass that carries one.
    assert_eq!(Ruling::Pass.verdict("why"), Verdict::Pass);
    assert_eq!(Ruling::Hold.verdict("why"), Verdict::Hold("why".to_owned()));
    assert_eq!(
        Ruling::Refuse.verdict("why"),
        Verdict::Refuse("why".to_owned())
    );
}

#[test]
fn an_unanswered_invocation_is_the_table_s_verdict() {
    let answers = Answers::fold(&[]);
    assert_eq!(
        answers.ruling(
            "toolu_1",
            "amber",
            Effect::OpenWorld,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Pass
    );
    assert_eq!(
        answers.ruling(
            "toolu_1",
            "amber",
            Effect::Read,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Pass
    );
    // …and it is the *workspace's* table, so the parked default is one line of
    // override away rather than gone.
    assert_eq!(
        answers.ruling(
            "toolu_1",
            "amber",
            Effect::OpenWorld,
            &crate::control::policy::Policy::parse("table:\n  open-world: hold\n")
        ),
        Ruling::Hold
    );
}

#[test]
fn a_once_answer_is_scoped_to_the_held_tool_use_id() {
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass"]),
        row(&[YOG_CONTROL, "answer", "toolu_2", "refuse"]),
    ]);
    assert_eq!(
        answers.ruling(
            "toolu_1",
            "amber",
            Effect::Destructive,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Pass,
        "the operator answered this exact invocation"
    );
    assert_eq!(
        answers.ruling(
            "toolu_2",
            "amber",
            Effect::Read,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Refuse
    );
    // Another id is untouched: a once-grant needs no consumption because the
    // provider-unique id cannot be asked twice.
    assert_eq!(
        answers.ruling(
            "toolu_3",
            "amber",
            Effect::Destructive,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Refuse
    );
}

#[test]
fn the_last_row_for_a_key_wins() {
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "refuse"]),
    ]);
    assert_eq!(
        answers.ruling(
            "toolu_1",
            "a",
            Effect::Read,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Refuse
    );
}

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
        answers.ruling(
            "t",
            "amber",
            Effect::Read,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Pass
    );
    assert_eq!(
        answers.ruling(
            "t",
            "amber",
            Effect::TargetWrite,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Hold
    );
    assert_eq!(
        answers.ruling(
            "t",
            "amber-1",
            Effect::Process,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Hold
    );
    // The floor raises; it never lowers.
    assert_eq!(
        answers.ruling(
            "t",
            "amber",
            Effect::Secret,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Refuse
    );
    // And a once-answer to this exact invocation still wins over it.
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "floor", "amber", "raise"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass"]),
    ]);
    assert_eq!(
        answers.ruling(
            "toolu_1",
            "amber",
            Effect::TargetWrite,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Pass
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
        answers.ruling(
            "t",
            "amber",
            Effect::Process,
            &crate::control::policy::Policy::default()
        ),
        Ruling::Pass
    );
}

#[test]
fn every_other_ops_row_folds_to_nothing() {
    // The trail is shared: a `bl claim`, a drift line, an off-grammar control
    // row and a truncated one must all leave the fold untouched.
    let answers = Answers::fold(&[
        row(&["bl", "claim", "bl-1a2b", "--as", "amber"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "maybe"]),
        row(&[YOG_CONTROL, "floor", "amber", "sideways"]),
        row(&[YOG_CONTROL, "answer"]),
        row(&[]),
    ]);
    assert_eq!(answers, Answers::default());
}
