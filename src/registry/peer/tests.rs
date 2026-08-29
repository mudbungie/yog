//! The grade's admission set, argued as a set: the three a foot may say, and
//! the classes it may not.

use super::*;
use crate::boundary::codec;
use crate::registry::Client;
use serde_json::json;

/// Decode an envelope the way the intake does, so the table below is written in
/// the operator's own spelling rather than in constructors.
fn gesture(request: &serde_json::Value) -> Gesture {
    codec::decode(request).expect("a gesture the codec knows")
}

/// **The foot set is exactly three, and it is enumerated rather than
/// subtracted.** Advertise what this box runs, wait for work addressed to it,
/// hand back what happened.
#[test]
fn a_foot_admits_the_three_tool_host_gestures() {
    for request in [
        json!({"op": "advertise", "tools": []}),
        json!({"op": "invocations"}),
        json!({"op": "complete", "invocation": "i-1",
               "capture": {"stdout": "", "stderr": "", "exit_code": 0}}),
    ] {
        assert!(Grade::Foot.admits(&gesture(&request)), "{request}");
    }
}

/// It admits nothing else — a read about the world, an act on it, and the
/// routing leg's asking half, which is the verb whose absence is the whole
/// sentence: a foot is invoked, it never invokes.
#[test]
fn a_foot_admits_nothing_else() {
    for request in [
        json!({"op": "workspaces"}),
        json!({"op": "conversations", "workspace": "home"}),
        json!({"op": "follow", "workspace": "home", "agent": "c-1"}),
        json!({"op": "capture", "invocation": "i-1"}),
        json!({"op": "invoke", "client": "host", "tool": "Bash", "input": {}}),
        json!({"op": "prepare", "workspace": "home", "payload": {"rung": "bare"}}),
    ] {
        assert!(!Grade::Foot.admits(&gesture(&request)), "{request}");
    }
}

/// Operator grade is the whole boundary and the default — a certificate minted
/// before the grade existed says nothing about it and must keep working.
#[test]
fn operator_is_the_default_and_admits_everything() {
    assert_eq!(Grade::default(), Grade::Operator);
    for request in [
        json!({"op": "workspaces"}),
        json!({"op": "invocations"}),
        json!({"op": "invoke", "client": "host", "tool": "Bash", "input": {}}),
    ] {
        assert!(Grade::Operator.admits(&gesture(&request)), "{request}");
    }
}

/// A peer is a value: the identity and the grade travel together, and two
/// grades under one name are two peers.
#[test]
fn a_peer_carries_both_facts() {
    let client = Client::parse("host").expect("a usable identity");
    let host = Peer {
        client: client.clone(),
        grade: Grade::Foot,
    };
    assert_eq!(host, host.clone());
    assert_ne!(
        host,
        Peer {
            client,
            grade: Grade::Operator,
        }
    );
    assert!(REFUSAL.contains(FOOT), "the sentence names the grade");
}
