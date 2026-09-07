//! The trail fold: the once-answer and what it is not. The scoped standing
//! answers are [`standing`] and the floor is `judge::tests::floor`, each its
//! own file at §12's cap on the seam the fold is cut along.

use super::*;
use crate::control::judge::Table;
use crate::control::policy::Policy;
use crate::opslog::Origin;
use serde_json::json;

/// A `yog-control` ops row, written in the workspace `cwd` names.
fn at(cwd: &str, words: &[&str]) -> OpEntry {
    OpEntry {
        ts: "TS".to_owned(),
        argv: words.iter().map(|s| (*s).to_owned()).collect(),
        cwd: cwd.to_owned(),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::World,
        client: crate::registry::Client::default(),
    }
}

fn row(words: &[&str]) -> OpEntry {
    at("", words)
}

/// One invocation to judge.
fn call(id: &str, name: &str, agent: &str) -> Request {
    Request {
        id: id.to_owned(),
        name: name.to_owned(),
        input: json!({}),
        role: "worker".to_owned(),
        agent_id: agent.to_owned(),
    }
}

fn ruling(answers: &Answers, request: &Request, ws: &str, effect: Effect) -> Standing {
    answers.ruling(request, ws, effect, &Policy::default())
}

#[test]
fn an_unanswered_invocation_is_the_table_s_verdict_at_workspace_scope() {
    let answers = Answers::fold(&[]);
    let call = call("toolu_1", "bash", "amber");
    assert_eq!(
        ruling(&answers, &call, "w", Effect::OpenWorld),
        Standing {
            ruling: Ruling::Pass,
            scope: Scope::Workspace
        }
    );
    // …and it is the *workspace's* table, so the parked default is one line of
    // override away rather than gone.
    assert_eq!(
        answers
            .ruling(
                &call,
                "w",
                Effect::OpenWorld,
                &Policy::parse("table:\n  open-world: hold\n")
            )
            .ruling,
        Ruling::Hold
    );
}

#[test]
fn a_call_answer_is_scoped_to_the_held_tool_use_id() {
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass", "call"]),
        // A four-word row is an answer written before the scope existed, and
        // it reads as the call it was.
        row(&[YOG_CONTROL, "answer", "toolu_2", "refuse"]),
    ]);
    assert_eq!(
        ruling(
            &answers,
            &call("toolu_1", "bash", "amber"),
            "w",
            Effect::Destructive
        ),
        Standing {
            ruling: Ruling::Pass,
            scope: Scope::Call
        }
    );
    assert_eq!(
        ruling(
            &answers,
            &call("toolu_2", "bash", "amber"),
            "w",
            Effect::Read
        )
        .ruling,
        Ruling::Refuse
    );
    // Another id is untouched: a once-grant needs no consumption because the
    // provider-unique id cannot be asked twice.
    assert_eq!(
        ruling(
            &answers,
            &call("toolu_3", "bash", "amber"),
            "w",
            Effect::Destructive
        ),
        Standing {
            ruling: Table::ruling(Effect::Destructive),
            scope: Scope::Workspace
        }
    );
}

#[test]
fn the_last_row_for_a_key_wins_at_every_scope() {
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass", "call"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "refuse", "call"]),
        at(
            "ws",
            &[YOG_CONTROL, "answer", "bash@read", "pass", "workspace"],
        ),
        at(
            "ws",
            &[YOG_CONTROL, "answer", "bash@read", "refuse", "workspace"],
        ),
    ]);
    assert_eq!(
        ruling(&answers, &call("toolu_1", "bash", "a"), "ws", Effect::Read).ruling,
        Ruling::Refuse
    );
    assert_eq!(
        ruling(&answers, &call("toolu_2", "bash", "a"), "ws", Effect::Read).ruling,
        Ruling::Refuse,
        "the standing answer is superseded, not doubled"
    );
}

#[test]
fn a_routed_refusal_is_addressed_to_the_operator_as_a_hold() {
    let answers = Answers::fold(&[]);
    let at = |name: &str, effect| ruling(&answers, &call("t1", name, "amber"), "w", effect).ruling;
    // The engine's own leg keeps the in-band decline: the operator is sitting
    // at that machine and the blast radius is in front of them.
    assert_eq!(at("bash", Effect::Destructive), Ruling::Refuse);
    assert_eq!(at("bash", Effect::Secret), Ruling::Refuse);
    assert_eq!(at("box2_shell", Effect::Destructive), Ruling::Hold);
    assert_eq!(at("box2_shell", Effect::Secret), Ruling::Hold);
    // Nothing else moves: what passes on one leg passes on the other, and a
    // hold stays a hold. The mapping is never the other direction.
    assert_eq!(at("box2_shell", Effect::OpenWorld), Ruling::Pass);
    assert_eq!(at("box2_thing", Effect::Opaque), Ruling::Hold);
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
        ruling(
            &answers,
            &call("toolu_9", "box2_shell", "amber"),
            "w",
            Effect::Destructive
        )
        .ruling,
        Ruling::Refuse
    );
}

#[test]
fn every_other_ops_row_folds_to_nothing() {
    // The trail is shared: a `bl claim`, a drift line, an off-grammar control
    // row, a scope word nobody knows and a truncated one must all leave the
    // fold untouched.
    let answers = Answers::fold(&[
        row(&["bl", "claim", "bl-1a2b", "--as", "amber"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "maybe"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass", "everywhere"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "maybe", "call"]),
        // A conversation row with no class beside its conversation names no
        // class, so it stands for nothing.
        row(&[YOG_CONTROL, "answer", "amber", "pass", "conversation"]),
        row(&[YOG_CONTROL, "floor", "amber", "sideways"]),
        row(&[YOG_CONTROL, "answer"]),
        row(&[]),
    ]);
    assert_eq!(answers, Answers::default());
}

/// The scopes wider than one call (bl-94a5).
mod standing;
