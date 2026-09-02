//! The per-agent predicate: every signal, abandoned suppression, seen-gating
//! oid re-arm, the Free-vs-Unknown mail rule, and the real-`UiState` wiring.

use super::{acked, agent, nothing};
use crate::attention::*;
use crate::git_tree::{Agent, AgentState};
use crate::ui_state::{SeenKind, UiState};

/// An auth-shaped failure sentence: what makes `Agent::refused()` — the
/// reading, not a second stored flag — answer true (bl-9b88).
const AUTH_SHAPED: &str = r#"{"type":"error","status":401,"message":"Unauthorized"}"#;

#[test]
fn no_signals_is_no_attention() {
    let a = attention(&agent("a"), "ws", &nothing);
    assert!(!a.any());
    assert!(a.kinds().is_empty());
    assert_eq!(
        a,
        Attention {
            notify: false,
            stopped: false,
            refused: false,
            budget: false,
            conflicted: false,
            mail: false,
            held: false,
        }
    );
}

#[test]
fn each_marked_signal_fires_when_unseen() {
    type Case = (fn(&mut Agent), AttentionKind);
    let cases: &[Case] = &[
        (|a| a.notify_oid = Some("n".into()), AttentionKind::Notify),
        (|a| a.budget_oid = Some("b".into()), AttentionKind::Budget),
        (
            |a| a.conflicted_oid = Some("c".into()),
            AttentionKind::Conflicted,
        ),
    ];
    for (mutate, kind) in cases {
        let mut ag = agent("a");
        mutate(&mut ag);
        let att = attention(&ag, "ws", &nothing);
        assert!(att.any(), "{kind:?} should fire");
        assert_eq!(att.kinds(), vec![*kind]);
        assert!(!att.mail, "a mark never implies mail");
    }
}

/// Rule 2 fires on **rest**, not on the wound (ruled bl-2194): the clean end
/// (`Quiescent`) and the failed end (`Stopped`) are one queue entry apiece, and
/// a running agent is not in the queue at all.
#[test]
fn rest_fires_however_the_conversation_came_to_rest() {
    for state in [AgentState::Quiescent, AgentState::Stopped] {
        let mut ag = agent("a");
        ag.state = state;
        let att = attention(&ag, "ws", &nothing);
        assert!(att.stopped, "{state:?} is a turn waiting on you");
        assert_eq!(att.kinds(), vec![AttentionKind::Stopped]);
        // The same tip, acknowledged, is silent forever — the muting mechanism
        // the ruling needs none of.
        assert!(!attention(&ag, "ws", &acked(SeenKind::Stopped, "tip-a")).stopped);
    }
    for state in [AgentState::Live, AgentState::InFlight] {
        let mut ag = agent("a");
        ag.state = state;
        assert!(
            !attention(&ag, "ws", &nothing).stopped,
            "{state:?} is still running — nothing is waiting on you"
        );
    }
}

#[test]
fn abandoned_suppresses_the_rest_signal_from_either_rest() {
    for state in [AgentState::Quiescent, AgentState::Stopped] {
        let mut ag = agent("a");
        ag.state = state;
        ag.abandoned_oid = Some("dead".into());
        let att = attention(&ag, "ws", &nothing);
        assert!(
            !att.stopped,
            "abandoned = will-not-retry suppresses rest (§6), {state:?}"
        );
        assert!(!att.any());
    }
}

/// Rule 6 (§8.6): a park is attention, it is not acknowledgeable, and it does
/// not appear in the evidence a `seen` writes — a watermark over a parked drone
/// would hide a conversation nothing but an answer can move.
#[test]
fn a_park_is_attention_no_acknowledgement_can_quiet() {
    let mut ag = agent("a");
    ag.held = Some(crate::control::hold::Held {
        tool_use_id: "toolu_1".into(),
        tool: "bash".into(),
        reason: "bash {\"command\":\"curl x\"} classified open-world".into(),
    });
    let att = attention(&ag, "ws", &nothing);
    assert!(att.held && att.any());
    assert_eq!(att.kinds(), vec![AttentionKind::Held]);
    // Every watermark in the world leaves it firing.
    for kind in [
        SeenKind::Notify,
        SeenKind::Stopped,
        SeenKind::Budget,
        SeenKind::Conflicted,
    ] {
        assert!(attention(&ag, "ws", &acked(kind, "toolu_1")).held);
    }
    assert!(
        evidence(&ag).is_empty(),
        "a park writes no watermark — there is nothing to acknowledge"
    );
}

/// The one home of rule 2's non-watermark gate: the ack (`app::focus`) and the
/// predicate read the same answer, so they cannot drift.
#[test]
fn rest_evidence_is_the_tip_of_an_unabandoned_agent_at_rest() {
    let mut ag = agent("a");
    assert_eq!(rest_evidence(&ag), None, "Live: no rest evidence");
    ag.state = AgentState::Quiescent;
    assert_eq!(rest_evidence(&ag), Some("tip-a".to_string()));
    ag.abandoned_oid = Some("dead".into());
    assert_eq!(rest_evidence(&ag), None, "abandoned: no rest evidence");
}

#[test]
fn acked_oid_clears_but_moved_oid_re_arms() {
    let mut ag = agent("a");
    ag.notify_oid = Some("v2".into());
    // Watermark still on the old oid "v1": the current oid "v2" is unseen.
    assert!(
        attention(&ag, "ws", &acked(SeenKind::Notify, "v1")).notify,
        "a moved ref re-notifies (§4.1)"
    );
    // Watermark caught up to "v2": acknowledged, no attention.
    assert!(!attention(&ag, "ws", &acked(SeenKind::Notify, "v2")).notify);
}

#[test]
fn stopped_tip_watermark_gates_on_the_branch_tip() {
    let mut ag = agent("a");
    ag.state = AgentState::Stopped;
    // Seen the current tip oid -> stop acknowledged.
    assert!(!attention(&ag, "ws", &acked(SeenKind::Stopped, "tip-a")).stopped);
    // A stale watermark (old tip) -> re-armed.
    assert!(attention(&ag, "ws", &acked(SeenKind::Stopped, "tip-old")).stopped);
}

#[test]
fn mail_fires_only_on_pending_and_definite_free() {
    // (state, uncertain, pending, expect_mail)
    let cases = [
        (AgentState::Quiescent, false, 2, true),  // Free
        (AgentState::Stopped, false, 1, true),    // Free
        (AgentState::Quiescent, true, 2, false),  // Unknown, not Free
        (AgentState::Stopped, true, 3, false),    // Unknown, not Free
        (AgentState::Live, false, 5, false),      // Held
        (AgentState::InFlight, false, 5, false),  // Held
        (AgentState::Quiescent, false, 0, false), // Free but no mail
    ];
    for (state, uncertain, pending, expect) in cases {
        let mut ag = agent("a");
        ag.state = state;
        ag.state_uncertain = uncertain;
        ag.pending = vec![crate::inboxview::InboxEntry::default(); pending];
        // Abandon any stop so this isolates the mail bit.
        ag.abandoned_oid = Some("x".into());
        assert_eq!(
            attention(&ag, "ws", &nothing).mail,
            expect,
            "{state:?} uncertain={uncertain} pending={pending}"
        );
    }
}

#[test]
fn all_signals_together_list_in_badge_order() {
    let mut ag = agent("a");
    ag.notify_oid = Some("n".into());
    ag.state = AgentState::Stopped;
    ag.budget_oid = Some("b".into());
    ag.conflicted_oid = Some("c".into());
    ag.pending = vec![crate::inboxview::InboxEntry::default()]; // Stopped + Free -> mail too
    assert_eq!(
        attention(&ag, "ws", &nothing).kinds(),
        vec![
            AttentionKind::Notify,
            AttentionKind::Stopped,
            AttentionKind::Budget,
            AttentionKind::Conflicted,
            AttentionKind::Mail,
        ]
    );
}

#[test]
fn wires_through_a_real_ui_state() {
    // An unwritable path: the record still holds in RAM (the write-through
    // failure is swallowed, `UiState::save`) and the query answers from it.
    let mut ui = UiState::open(std::path::PathBuf::from("/nonexistent/dir/ui.json"));
    ui.record_seen("ws", "a", &[(SeenKind::Notify, "oid1".to_string())]);
    let mut ag = agent("a");
    ag.notify_oid = Some("oid1".into());
    let seen = |k: SeenKind, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    // Acked -> no notify.
    assert!(!attention(&ag, "ws", &seen).notify);
    // A moved ref -> re-armed.
    ag.notify_oid = Some("oid2".into());
    assert!(attention(&ag, "ws", &seen).notify);
}

/// **Rule 2's rest, said in the word that is true of it** (bl-b43b). A
/// conversation refused at the provider rung comes to rest `Stopped` exactly
/// as an operator's own `/stop` does — the badge set is frozen at four — so
/// `stopped` would tell the operator they did a thing they did not do.
#[test]
fn a_refused_rest_earns_the_refused_word_and_not_stoppeds() {
    let mut ag = agent("a");
    ag.state = AgentState::Stopped;
    let mine = attention(&ag, "ws", &nothing);
    assert_eq!(mine.kinds(), vec![AttentionKind::Stopped]);

    ag.failure = Some(AUTH_SHAPED.to_owned());
    let theirs = attention(&ag, "ws", &nothing);
    assert_eq!(theirs.kinds(), vec![AttentionKind::Refused]);

    // One firing, not two: the refinement changes the word, never the count,
    // so a refused conversation is not asked about twice.
    assert_eq!(mine.any(), theirs.any());
    assert!(theirs.stopped, "it is still rule 2 that fired");
}

/// The word carries the remedy, because the remedy is not a gesture on the
/// conversation at all — which is what the desktop escalation says out loud.
#[test]
fn the_refused_word_names_the_act() {
    let said = AttentionKind::Refused.says();
    assert!(said.contains("refused at the provider"), "{said}");
    assert!(said.contains("sign a provider in"), "{said}");
}

/// The refinement is **gated on the rest firing**: an acknowledged tip stirs
/// nothing, refused or not, so a refusal cannot smuggle a signal past the
/// watermark.
#[test]
fn an_acked_refused_rest_still_stirs_nothing() {
    let mut ag = agent("a");
    ag.state = AgentState::Stopped;
    ag.failure = Some(AUTH_SHAPED.to_owned());
    assert!(
        attention(&ag, "ws", &acked(SeenKind::Stopped, "tip-a"))
            .kinds()
            .is_empty()
    );
}
