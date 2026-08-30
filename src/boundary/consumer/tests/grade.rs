//! **The certificate grade raising at the scoped intake** (REMOTE §4.2,
//! bl-7ff3): what a foot may say, what it may not, and the sentence the second
//! class earns — in band and naming the grade, never absent-shaped.

use super::*;
use crate::cli_outbound::Cli;
use crate::registry::{Grade, Peer};
use serde_json::json;
use tempfile::tempdir;

/// A foot-grade peer under `name` — what a leaf carrying the grade's own
/// organizational unit reads back as.
fn foot(name: &str) -> Peer {
    Peer {
        client: client(name),
        grade: Grade::Foot,
    }
}

fn quiet(root: &std::path::Path, data: &std::path::Path) -> ConsumerCtx {
    over(
        root,
        world_of(data, &["home"]),
        data.to_path_buf(),
        Cli::new("/no/such/litany"),
    )
}

fn tools() -> serde_json::Value {
    json!([{"name": "Bash", "description": "run a command",
            "input_schema": {"type": "object"}}])
}

/// Whether a reply is the grade's own refusal — the sentence names the grade,
/// which is the half §4's absence rule is deliberately not applied to.
fn refused_for_grade(reply: &serde_json::Value) -> bool {
    reply["ok"] == false && reply["error"] == crate::registry::peer::REFUSAL
}

/// **The three a foot may send**, and it may send them at a server that has
/// never heard of it: advertise its set, take the invocations addressed to it,
/// and complete one. The completion here quotes a handle nobody minted, so what
/// comes back is the mailbox's own sentence — which is the proof it got past
/// the grade rather than being stopped by it.
#[test]
fn a_foot_may_advertise_take_its_invocations_and_complete_one() {
    let (root, data) = (tempdir().unwrap(), tempdir().unwrap());
    let ctx = quiet(root.path(), data.path());
    let host = foot("host");

    let advertised = ctx.answer_as(&host, &json!({"op": "advertise", "tools": tools()}));
    assert_eq!(advertised["kind"], "advertised", "{advertised}");
    assert_eq!(
        crate::registry::tools::read(root.path(), &host.client)[0].name,
        "Bash"
    );

    let queue = ctx.answer_as(&host, &json!({"op": "invocations"}));
    assert_eq!(queue["kind"], "invocations", "{queue}");

    let done = ctx.answer_as(
        &host,
        &json!({"op": "complete", "invocation": "nobody-minted-this",
                "capture": {"stdout": "", "stderr": "", "exit_code": 0}}),
    );
    assert!(!refused_for_grade(&done), "{done}");
    assert!(
        done["error"]
            .as_str()
            .unwrap_or_default()
            .contains("nobody-minted-this"),
        "the mailbox answered, not the grade: {done}"
    );
}

/// **Everything else is refused, in band and naming the grade.** A read about
/// the world, an act on the world, and the routing leg's asking half — a foot
/// is invoked, it never invokes — each earn the one sentence.
#[test]
fn a_foot_may_not_ask_about_the_world_act_on_it_or_invoke() {
    let (root, data) = (tempdir().unwrap(), tempdir().unwrap());
    let ctx = quiet(root.path(), data.path());
    let host = foot("host");
    crate::registry::register(root.path(), &host.client, "home").unwrap();
    for request in [
        json!({"op": "workspaces"}),
        json!({"op": "conversations", "workspace": "home"}),
        json!({"op": "clients", "workspace": "home"}),
        json!({"op": "capture", "invocation": "x"}),
        json!({"op": "invoke", "client": "host", "tool": "Bash", "input": {}}),
    ] {
        let refusal = ctx.answer_as(&host, &request);
        assert!(refused_for_grade(&refusal), "{request} -> {refusal}");
    }
}

/// **A foot never reaches the follow lane**, and the refusal it reads is the
/// same one every other over-reach earns: `follow` answers no stream, and the
/// one function that words refusals words this one too.
#[test]
fn a_foot_gets_no_stream_and_the_grades_sentence_instead() {
    let (root, data) = (tempdir().unwrap(), tempdir().unwrap());
    let ctx = quiet(root.path(), data.path());
    let host = foot("host");
    crate::registry::register(root.path(), &host.client, "home").unwrap();
    let request = json!({"op": "follow", "workspace": "home", "agent": "c-1"});
    assert!(ctx.follow(&host, &request).is_none());
    assert!(
        refused_for_grade(&ctx.answer_as(&host, &request)),
        "refused"
    );
}

/// **Default-operator is the whole of the compatibility story** (REMOTE §4.2):
/// the same identity at operator grade — which is every certificate minted
/// before the grade existed — is unaffected by any of it, and a foot's own
/// three still answer for an operator too.
#[test]
fn an_operator_grade_peer_is_untouched_by_the_raise() {
    let (root, data) = (tempdir().unwrap(), tempdir().unwrap());
    let ctx = quiet(root.path(), data.path());
    let desk = seat("host");
    assert_eq!(desk.grade, Grade::default(), "unstated is operator");
    let listed = ctx.answer_as(&desk, &json!({"op": "workspaces"}));
    assert_eq!(listed["kind"], "workspaces", "{listed}");
    let advertised = ctx.answer_as(&desk, &json!({"op": "advertise", "tools": tools()}));
    assert_eq!(advertised["kind"], "advertised", "{advertised}");
}

/// A foot's undecodable envelope is refused by the codec, not by the grade —
/// the decode is ahead of the raise because a refusal that cannot say what was
/// asked is worse than one that can, and nothing has been dispatched either way.
#[test]
fn an_undecodable_envelope_is_still_the_codecs_refusal() {
    let (root, data) = (tempdir().unwrap(), tempdir().unwrap());
    let ctx = quiet(root.path(), data.path());
    let refusal = ctx.answer_as(&foot("host"), &json!({"op": "teleport"}));
    assert_eq!(refusal["ok"], false);
    assert!(
        refusal["error"]
            .as_str()
            .unwrap_or_default()
            .contains("teleport"),
        "{refusal}"
    );
}
