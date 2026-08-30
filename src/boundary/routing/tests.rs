//! The routing leg's four arms: who may ask each, what each answers, and the
//! two facts that make a call cross at all (REMOTE §3, §5).

use super::*;
use crate::boundary::dispatch::Caller;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::registry::mailbox::{Capture, Mailbox};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

fn tool(name: &str) -> tools::Tool {
    tools::Tool {
        name: name.to_owned(),
        description: "does a thing".to_owned(),
        input_schema: json!({"type": "object"}),
    }
}

/// A `Deps` for `client`, over `state_root`, sharing `mailbox` — the whole
/// point being that two callers of different identities share one map.
fn deps(state_root: &Path, client: &str, mailbox: &Mailbox) -> Deps {
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::xdg::Env::from_env(),
        snapshot: Arc::new(snapshot(
            Path::new("/names/alba"),
            "alba",
            Vec::new(),
            Vec::new(),
        )),
        caller: Caller {
            client: Client::parse(client).unwrap_or_default(),
            mailbox: mailbox.clone(),
            ..Caller::default()
        },
    }
}

fn call(client: &str, tool: &str) -> Call {
    Call {
        client: client.to_owned(),
        tool: tool.to_owned(),
        input: json!({"command": "ls"}),
    }
}

fn ran() -> Capture {
    Capture {
        stdout: "hello\n".to_owned(),
        stderr: String::new(),
        exit_code: 0,
    }
}

/// The handle an `invoke` answered with.
fn handle(reply: Result<Reply, String>) -> String {
    match reply {
        Ok(Reply::Routed { invocation, .. }) => invocation,
        other => panic!("not a routed reply: {other:?}"),
    }
}

/// **The whole leg, from four sides.** A driver queues a call, a tool host
/// drains it, runs it and posts the capture, and the driver's poll collects it
/// — one mailbox, four arms, and nothing that waits on the other's thread.
#[test]
fn a_call_crosses_and_the_capture_comes_back() {
    let root = tempdir().expect("tmp");
    let mail = Mailbox::holding(2, Duration::from_millis(1));
    let laptop = Client::parse("laptop").expect("identity");
    tools::store(root.path(), &laptop, &[tool("Bash")]).expect("advertise");

    let driver = deps(root.path(), crate::registry::LOCAL, &mail);
    let id = handle(invoke(&driver, "100", &call("laptop", "Bash")));

    let host = deps(root.path(), "laptop", &mail);
    let taken = invocations(&host);
    assert_eq!(
        taken,
        Ok(Reply::Invocations(vec![
            crate::registry::mailbox::Invocation {
                id: id.clone(),
                tool: "Bash".to_owned(),
                input: json!({"command": "ls"}),
            }
        ]))
    );

    assert_eq!(
        capture(&driver, &id),
        Ok(Reply::Routed {
            invocation: id.clone(),
            capture: None,
        }),
        "nothing captured while it runs"
    );
    let done = Completion {
        invocation: id.clone(),
        capture: ran(),
    };
    assert_eq!(
        complete(&host, &done),
        Ok(Reply::Routed {
            invocation: id.clone(),
            capture: Some(ran()),
        }),
        "the receipt is the slot re-read"
    );
    assert_eq!(
        capture(&driver, &id),
        Ok(Reply::Routed {
            invocation: id,
            capture: Some(ran()),
        })
    );
}

/// **REMOTE §5's staleness correction, asked where it is cheap**: a machine
/// that does not advertise the tool refuses in band, naming both.
#[test]
fn a_tool_the_client_does_not_advertise_refuses_naming_it() {
    let root = tempdir().expect("tmp");
    let mail = Mailbox::default();
    let laptop = Client::parse("laptop").expect("identity");
    tools::store(root.path(), &laptop, &[tool("Bash")]).expect("advertise");
    let driver = deps(root.path(), crate::registry::LOCAL, &mail);

    let refused = invoke(&driver, "1", &call("laptop", "Rm"));
    assert!(
        refused.is_err_and(|e| e.contains("\"laptop\"") && e.contains("\"Rm\"")),
        "names the machine and the tool"
    );
    // And a machine nobody ever seated advertises nothing at all, so it refuses
    // on the same rule rather than a second one.
    assert!(invoke(&driver, "1", &call("phone", "Bash")).is_err());
    // A name the layout has already spent is not an identity (REMOTE §4.1).
    assert!(invoke(&driver, "1", &call("local", "Bash")).is_err());
}

/// **The two verbs whose authorization is the certificate** (REMOTE §5.1's
/// precedent): an intake carrying no client identity is refused in band, with a
/// sentence — never silently handed another machine's work.
#[test]
fn an_in_world_caller_has_no_invocations_and_completes_nothing() {
    let root = tempdir().expect("tmp");
    let mail = Mailbox::holding(1, Duration::from_millis(1));
    let inbox = deps(root.path(), crate::registry::LOCAL, &mail);

    for refusal in [
        invocations(&inbox),
        complete(
            &inbox,
            &Completion {
                invocation: "inv-1".to_owned(),
                capture: ran(),
            },
        ),
    ] {
        assert!(
            refusal.is_err_and(|e| e.contains("carries no client identity")),
            "the category error is named"
        );
    }
}

/// A hold that ends with nothing waiting answers the empty set, and a poll on a
/// handle this caller never posted is **absent** rather than forbidden.
#[test]
fn an_empty_hold_and_an_unheld_handle_are_both_ordinary_answers() {
    let root = tempdir().expect("tmp");
    let mail = Mailbox::holding(1, Duration::from_millis(1));
    let host = deps(root.path(), "laptop", &mail);
    assert_eq!(invocations(&host), Ok(Reply::Invocations(Vec::new())));
    assert!(
        capture(&host, "inv-404").is_err_and(|e| e.contains("inv-404")),
        "the handle is named"
    );
}
