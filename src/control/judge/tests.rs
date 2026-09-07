//! The judgment fold: the shipped table, the once-answer, and the floor.

use super::*;
use crate::control::policy::Policy;
use crate::opslog::Origin;

/// What `answers` rules for one invocation of `name` by `agent`, on the
/// **shipped** table — the shape every case below one takes, said once so a
/// case reads as the fact it is about rather than as five arguments.
fn ruled(answers: &Answers, id: &str, agent: &str, name: &str, effect: Effect) -> Ruling {
    answers.ruling(id, agent, name, effect, &Policy::default())
}

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
        client: crate::registry::Client::default(),
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
        ruled(&answers, "toolu_1", "amber", "bash", Effect::OpenWorld),
        Ruling::Pass
    );
    assert_eq!(
        ruled(&answers, "toolu_1", "amber", "bash", Effect::Read),
        Ruling::Pass
    );
    // …and it is the *workspace's* table, so the parked default is one line of
    // override away rather than gone.
    assert_eq!(
        answers.ruling(
            "toolu_1",
            "amber",
            "bash",
            Effect::OpenWorld,
            &Policy::parse("table:\n  open-world: hold\n")
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
        ruled(&answers, "toolu_1", "amber", "bash", Effect::Destructive),
        Ruling::Pass,
        "the operator answered this exact invocation"
    );
    assert_eq!(
        ruled(&answers, "toolu_2", "amber", "bash", Effect::Read),
        Ruling::Refuse
    );
    // Another id is untouched: a once-grant needs no consumption because the
    // provider-unique id cannot be asked twice.
    assert_eq!(
        ruled(&answers, "toolu_3", "amber", "bash", Effect::Destructive),
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
        ruled(&answers, "toolu_1", "a", "bash", Effect::Read),
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
