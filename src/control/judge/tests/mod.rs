//! The judgment fold: the shipped table, the once-answer, and the floor.

use super::*;
use crate::control::policy::Policy;
use crate::opslog::Origin;

/// The revoke rung's own subject, its own file: what a floor reaches, what it
/// deliberately does not, and when it stops binding.
mod floor;

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

/// bl-1772: the class is the same on both legs; the party the verdict is
/// delivered to is not. A refusal is a sentence handed to the model, and the
/// drive that filed this rephrased past one — `cd <dir> && rm -f -- *` after a
/// bare `rm -f <dir>/*` was declined — and deleted 115 MB from another machine
/// with no hold, no attention item and nothing to answer.
#[test]
fn a_routed_refusal_is_a_hold_because_the_model_can_rephrase_a_refusal() {
    let answers = Answers::default();
    // The engine's own machine: the operator is sitting at it, so loss and
    // credentials decline in band exactly as they did.
    assert_eq!(
        ruled(&answers, "t1", "amber", "bash", Effect::Destructive),
        Ruling::Refuse
    );
    assert_eq!(
        ruled(&answers, "t1", "amber", "bash", Effect::Secret),
        Ruling::Refuse
    );
    // A machine a foot administers: the operator is asked instead, which is
    // also the only way they can say YES to the one destructive act a foot
    // exists to make safe.
    assert_eq!(
        ruled(&answers, "t1", "amber", "box2_shell", Effect::Destructive),
        Ruling::Hold
    );
    assert_eq!(
        ruled(&answers, "t1", "amber", "box2_shell", Effect::Secret),
        Ruling::Hold
    );
    // Nothing else moves: what passes on one leg passes on the other, and a
    // hold stays a hold. The mapping is never the other direction.
    assert_eq!(
        ruled(&answers, "t1", "amber", "box2_shell", Effect::OpenWorld),
        Ruling::Pass
    );
    assert_eq!(
        ruled(&answers, "t1", "amber", "box2_thing", Effect::Opaque),
        Ruling::Hold
    );
    assert_eq!(Ruling::Pass.for_the_operator(), Ruling::Pass);
    assert_eq!(Ruling::Hold.for_the_operator(), Ruling::Hold);
}

/// The once-answer stands ahead of the leg: an operator who answered THIS
/// invocation has made the decision, and re-parking it would ask them the
/// question they just answered.
#[test]
fn an_operators_own_refusal_of_a_routed_call_is_not_turned_back_into_a_park() {
    let answers = Answers::fold(&[row(&[YOG_CONTROL, "answer", "toolu_9", "refuse"])]);
    assert_eq!(
        ruled(
            &answers,
            "toolu_9",
            "amber",
            "box2_shell",
            Effect::Destructive
        ),
        Ruling::Refuse
    );
}
