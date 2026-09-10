//! The **floor** (VISION §4.9's fifth rung, bl-94b4): a per-conversation park
//! over a whole descent, the one pair it does not reach (bl-a821), and — since
//! bl-94a5 — the standing answers it suspends. Its own file beside the
//! vocabulary, on the seam main drew for it: what a floor DOES is one subject
//! and the words a decision is made in are another.

use crate::control::classify::Effect;
use crate::control::judge::answers::Standing;
use crate::control::judge::{Answers, Ruling, Scope};
use crate::control::policy::Policy;
use crate::control::wire::Request;
use crate::opslog::{OpEntry, Origin, YOG_CONTROL};

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

/// One invocation to judge.
fn call(id: &str, name: &str, agent: &str) -> Request {
    Request {
        id: id.to_owned(),
        name: name.to_owned(),
        input: serde_json::json!({}),
        role: "worker".to_owned(),
        agent_id: agent.to_owned(),
    }
}

/// What `answers` rules for one invocation, on the shipped table.
fn ruling(answers: &Answers, request: &Request, ws: &str, effect: Effect) -> Standing {
    answers.ruling(request, ws, effect, &Policy::default())
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
        ruling(&answers, &call("t", "bash", "amber"), "w", Effect::Read).ruling,
        Ruling::Pass
    );
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "amber"),
            "w",
            Effect::TargetWrite
        ),
        Standing {
            ruling: Ruling::Hold,
            scope: Scope::Conversation
        }
    );
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "amber-1"),
            "w",
            Effect::Process
        )
        .ruling,
        Ruling::Hold
    );
    // The floor raises; it never lowers.
    assert_eq!(
        ruling(&answers, &call("t", "bash", "amber"), "w", Effect::Secret).ruling,
        Ruling::Refuse
    );
    // And a call answer to this exact invocation still wins over it.
    let answers = Answers::fold(&[
        row(&[YOG_CONTROL, "floor", "amber", "raise"]),
        row(&[YOG_CONTROL, "answer", "toolu_1", "pass", "call"]),
    ]);
    assert_eq!(
        ruling(
            &answers,
            &call("toolu_1", "bash", "amber"),
            "w",
            Effect::TargetWrite
        )
        .ruling,
        Ruling::Pass
    );
}

/// **bl-1772**: a refusal on a routed leg is delivered to the model, which is
/// the one party with an interest in rephrasing it — measured: a bare
/// `rm -f <dir>/*` refused destructive, then `cd <dir> && rm -f -- *` ran. On
/// that leg the table's refusal becomes a hold, and the operator is asked.
/// **bl-a821**: `revoke` reaches a conversation's descendants and the
/// compactor is one, so the floor held `write_summary` — the operator was
/// queued a machinery act they have no basis to judge, and until they answered
/// it the floored conversation could not compact.
#[test]
fn a_floor_does_not_reach_the_compactor_s_checkpoint_pair() {
    let answers = Answers::fold(&[row(&[YOG_CONTROL, "floor", "amber", "raise"])]);
    for pair in ["write_summary", "mark_for_deletion"] {
        assert_eq!(
            ruling(
                &answers,
                &call("t", pair, "amber"),
                "w",
                Effect::TargetWrite
            )
            .ruling,
            Ruling::Pass,
            "{pair} is the compaction procedure's own act, not the agent's"
        );
        assert_eq!(
            ruling(
                &answers,
                &call("t", pair, "amber-1"),
                "w",
                Effect::TargetWrite
            )
            .ruling,
            Ruling::Pass,
            "and the compactor of a floored conversation is a descendant"
        );
    }
    // The exemption is the pair's and the floor's alone: every other target
    // write under the same floor still holds, and the workspace's own table
    // still rules the pair.
    assert_eq!(
        ruling(
            &answers,
            &call("t", "load_skill", "amber"),
            "w",
            Effect::TargetWrite
        )
        .ruling,
        Ruling::Hold
    );
    assert_eq!(
        answers
            .ruling(
                &call("t", "write_summary", "amber"),
                "w",
                Effect::TargetWrite,
                &Policy::parse("table:\n  target-write: hold\n")
            )
            .ruling,
        Ruling::Hold
    );
}

/// **bl-c6f0**: the url reading is a class, not a pass. A routed fetch tool
/// classifies open-world off its own `url` operand and the shipped table lets
/// it through — and the moment the operator raises the floor over that
/// conversation, the very same call parks like every other class above read.
/// The reading widens what the control can *say*; it takes nothing away from
/// what the operator can *impose*.
#[test]
fn the_url_reading_is_a_class_the_floor_still_bites_on() {
    let fetch = Request {
        input: serde_json::json!({"url": "https://example.invalid/x"}),
        ..call("toolu_f", "box2_fetch", "amber")
    };
    let root = crate::control::root::Root {
        writable: vec![std::path::PathBuf::from("/w/agent")],
        cwd: std::path::PathBuf::from("/w/agent"),
        home: std::path::PathBuf::from("/home/op"),
    };
    let effect = crate::control::classify::classify(&fetch, &root, &Policy::default()).effect;
    assert_eq!(effect, Effect::OpenWorld);
    let open = Answers::fold(&[]);
    assert_eq!(ruling(&open, &fetch, "w", effect).ruling, Ruling::Pass);
    let floored = Answers::fold(&[row(&[YOG_CONTROL, "floor", "amber", "raise"])]);
    assert_eq!(
        ruling(&floored, &fetch, "w", effect),
        Standing {
            ruling: Ruling::Hold,
            scope: Scope::Conversation
        }
    );
}
