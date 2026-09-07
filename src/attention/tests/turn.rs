//! **Whose turn a rest is** (§6 rule 2 as amended, bl-3592): the operator's
//! only when nobody dispatched the conversation.
//!
//! Ids here are litany's grammar (`<ts>-<short>` segments, ARCH §2.3) rather
//! than the one-token names the other beats use, because the whole question is
//! the descent the id encodes.

use super::{agent, nothing, tips_acked};
use crate::attention::*;
use crate::git_tree::{Agent, AgentState};
use crate::inboxview::{Deposit, InboxEntry};

const ROOT: &str = "20260906T090000Z-r001";
const CHILD: &str = "20260906T090000Z-r001-20260906T091000Z-c001";

/// One agent at rest at a tip nobody has acknowledged — rule 2's firing shape.
fn resting(id: &str) -> Agent {
    let mut a = agent(id);
    a.state = AgentState::Quiescent;
    a
}

/// **The beat this ball exists for.** A compactor — or a reviewer, or any other
/// conversation the harness forked — comes to rest and says nothing to the
/// operator: its result lands in its dispatcher's inbox, and the dispatcher's
/// driver takes it. Three ordinary goals used to answer twelve queue rows, of
/// which eight were this.
#[test]
fn a_dispatched_conversation_s_rest_is_not_the_operator_s_turn() {
    let agents = vec![resting(ROOT), resting(CHILD)];
    let child = attention(&agents[1], &agents, "ws", &nothing);
    assert!(
        !child.stopped,
        "the parent takes this rest, not the operator"
    );
    assert!(!child.any(), "so the child is not on the queue at all");

    let root = attention(&agents[0], &agents, "ws", &nothing);
    assert!(root.stopped, "and the conversation the operator started is");
}

/// **No turn is lost by the suppression.** If the dispatcher's driver does not
/// take the deposit, the dispatcher is at rest with mail nobody is driving —
/// rule 5, on the row that can act. The same turn, at the altitude that can
/// answer it.
#[test]
fn a_deposit_nobody_drives_raises_the_same_turn_on_the_parent() {
    let mut root = resting(ROOT);
    root.pending = vec![InboxEntry {
        name: "c001-001.md".into(),
        raw: Vec::new(),
        deposit: Deposit::default(),
    }];
    let agents = vec![root, resting(CHILD)];
    // The root's own rest is acknowledged, so rule 2 cannot be what fires.
    let seen = tips_acked(&[ROOT]);
    let parent = attention(&agents[0], &agents, "ws", &seen);
    assert!(!parent.stopped, "the rest itself is old news");
    assert!(parent.mail, "but the undriven result is the turn");
    assert_eq!(parent.kinds(), vec![AttentionKind::Mail]);
}

/// Only rule 2 is the parent's to take. A dispatched conversation that raised a
/// notify mark, parked a tool call at the capability boundary or was flagged
/// still fires — none of those is a turn a parent can take, and a compactor
/// holding a `rm -rf` for an answer is exactly what the queue is for.
#[test]
fn every_other_signal_still_fires_on_a_dispatched_conversation() {
    type Case = (fn(&mut Agent), AttentionKind);
    let cases: &[Case] = &[
        (|a| a.notify_oid = Some("n".into()), AttentionKind::Notify),
        (|a| a.budget_oid = Some("b".into()), AttentionKind::Budget),
        (
            |a| a.conflicted_oid = Some("c".into()),
            AttentionKind::Conflicted,
        ),
        (
            |a| {
                a.held = Some(crate::control::hold::Held {
                    tool_use_id: "toolu_1".into(),
                    tool: "Bash".into(),
                    reason: "destructive".into(),
                });
            },
            AttentionKind::Held,
        ),
    ];
    for (mutate, kind) in cases {
        let mut child = resting(CHILD);
        mutate(&mut child);
        let agents = vec![resting(ROOT), child];
        let att = attention(&agents[1], &agents, "ws", &nothing);
        assert!(att.any(), "{kind:?} is nobody's to take but the operator's");
        assert_eq!(att.kinds(), vec![*kind], "and rule 2 stays silent");
    }
}

/// The membership rule is the descent tree's, so the three shapes it renders at
/// depth 0 are the three this reads as nobody's child: a root, an id outside
/// litany's grammar, and a descendant whose dispatcher's ref is gone. The last
/// is the one that matters — nothing is left to take that rest, so it is the
/// operator's after all.
#[test]
fn a_conversation_whose_dispatcher_is_gone_is_the_operator_s_again() {
    let orphan = vec![resting(CHILD)];
    assert!(
        attention(&orphan[0], &orphan, "ws", &nothing).stopped,
        "its dispatcher holds no ref, so nobody is coming for this rest"
    );

    let outside = vec![resting(&format!("{ROOT}-c0ffee"))];
    assert!(
        attention(&outside[0], &outside, "ws", &nothing).stopped,
        "an id litany would never mint is nobody's child here either"
    );
}
