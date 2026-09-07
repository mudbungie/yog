//! The sentence: what a hold says, what a refusal adds, and the class read
//! back out of the first.

use super::*;
use crate::control::judge::Ruling;
use serde_json::json;

fn request(name: &str, input: serde_json::Value) -> Request {
    Request {
        id: "toolu_1".to_owned(),
        name: name.to_owned(),
        input,
        role: "worker".to_owned(),
        agent_id: "amber".to_owned(),
    }
}

#[test]
fn the_hold_sentence_names_the_tool_the_input_the_class_and_the_evidence() {
    let said = reason(
        &request("bash", json!({"command": "rm -rf /srv"})),
        &Classified::new(Effect::Destructive, "`rm` reaches /srv"),
    );
    assert_eq!(
        said,
        "bash {\"command\":\"rm -rf /srv\"} classified destructive (`rm` reaches /srv)"
    );
}

#[test]
fn a_long_input_is_clipped_and_says_so() {
    let long = "x".repeat(SUMMARY_MAX * 2);
    let said = reason(
        &request("bash", json!({ "command": long })),
        &Classified::new(Effect::Read, "why"),
    );
    assert!(said.contains('…'), "{said}");
    // …and the clause still lands after the cut, so the class is readable.
    assert_eq!(class_of(&said), Some(Effect::Read));
}

#[test]
fn a_newline_never_reaches_the_blob() {
    let said = reason(
        &request("bash", json!({"command": "one\ntwo\r\nthree"})),
        &Classified::new(Effect::Read, "why"),
    );
    assert!(!said.contains('\n'), "{said}");
}

#[test]
fn the_class_reads_back_out_of_the_sentence_that_wrote_it() {
    for effect in Effect::every() {
        let said = reason(&request("t", json!({})), &Classified::new(effect, "why"));
        assert_eq!(class_of(&said), Some(effect), "{said}");
    }
}

#[test]
fn a_sentence_with_no_clause_of_ours_names_no_class() {
    assert_eq!(class_of(""), None);
    assert_eq!(class_of("held because I said so"), None);
    // A clause quoted inside the input is outranked by the sentence's own,
    // which is always last.
    let said = reason(
        &request("bash", json!({"command": "echo classified secret ("})),
        &Classified::new(Effect::Read, "echo observes only"),
    );
    assert_eq!(class_of(&said), Some(Effect::Read), "{said}");
}

#[test]
fn a_pass_carries_no_reason_and_a_hold_carries_the_operator_s() {
    let request = request("bash", json!({}));
    let classified = Classified::new(Effect::Read, "why");
    let at = |ruling| {
        verdict(
            Standing {
                ruling,
                scope: Scope::Call,
            },
            &request,
            &classified,
        )
    };
    assert_eq!(at(Ruling::Pass), Verdict::Pass);
    assert_eq!(
        at(Ruling::Hold),
        Verdict::Hold(reason(&request, &classified))
    );
}

#[test]
fn a_refusal_names_the_scope_and_closes_the_three_loopholes() {
    let request = request("alpha2_Bash", json!({"command": "rm -f /srv/data/blobs/*"}));
    let classified = Classified::new(Effect::Destructive, "`rm` reaches /srv/data/blobs/*");
    let Verdict::Refuse(said) = verdict(
        Standing {
            ruling: Ruling::Refuse,
            scope: Scope::Workspace,
        },
        &request,
        &classified,
    ) else {
        panic!("a refusal");
    };
    // The classification is still there, and the paragraph is added to it.
    assert!(said.starts_with(&reason(&request, &classified)), "{said}");
    assert!(said.contains("every alpha2_Bash call classified destructive in this workspace"));
    assert!(said.contains("Do not retry it"), "{said}");
    assert!(said.contains("do not rephrase it"), "{said}");
    assert!(
        said.contains(
            "another command, another tool, another path or a change of working \
                       directory"
        ),
        "{said}"
    );
    // It never offers a way round.
    for bait in ["instead", "you may", "you could", "unless"] {
        assert!(!said.contains(bait), "{bait} in {said}");
    }
}

#[test]
fn the_refusal_paragraph_is_the_same_sentence_wherever_it_is_read() {
    assert_eq!(
        refusal(Scope::Call, "bash", Effect::Secret),
        refusal(Scope::Call, "other", Effect::Secret),
        "the call scope names no tool"
    );
    assert!(refusal(Scope::Conversation, "bash", Effect::Opaque).contains("bash"));
}
