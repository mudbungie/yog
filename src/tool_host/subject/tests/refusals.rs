//! The lane's sentences: a name no machine consents to and the engine does
//! not implement, a config ambiguity, and the two transport failures — each
//! in band, non-zero, and naming the way out.
//!
//! `deploy` stands for the operator-granted pool name throughout: a name with
//! no engine implementation behind it, so the last rung (`performs`) does not
//! catch it and the refusal is what the model reads. `bash` would be caught,
//! which is bl-5710's whole point.

use ::litany::cmd::{RoutedCall, ToolInjection as _};

use super::{advertised, injection, roster};
use crate::test_support::FakeClock;
use crate::tool_host::tests::scripted;
use serde_json::json;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

/// **Advertised without consent is a refusal naming the key** — the box must
/// opt in before it executes at a path the conversation names, and the
/// sentence carries the ONE way out that is one: the operator's config edit,
/// by key, file and box.
///
/// It also names the loaded lane as what it is *not* (bl-68e1). The old
/// sentence offered it first — *"load it with the clients tool to run it in
/// that machine's own directory"* — and a drive took that offer, wrote its
/// whole deliverable into the far foot's inherited process directory, checked
/// itself there, and reported success over an empty bound directory. So the
/// assertion is two-directional: the clients tool must be mentioned, and it
/// must be mentioned as not a way to do this work.
#[test]
fn an_unconsenting_advertiser_is_refused_naming_the_remedy() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[roster("laptop", &[advertised("deploy", false)])],
    );
    let input = json!({"target": "staging"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("deploy", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.contains("laptop advertises deploy"), "{said}");
    assert!(
        said.contains("no machine of this workspace consents"),
        "{said}"
    );
    assert!(said.contains("\"subject_cwd\": true"), "{said}");
    assert!(
        said.contains("clients tool is not a way to do this work"),
        "{said}"
    );
    assert!(
        said.contains("never this conversation's"),
        "the sentence must say where a loaded instance would run: {said}"
    );
}

/// **Two consenting machines is a config ambiguity, refused naming both** —
/// one adjudication decision must stand for exactly one execution on one
/// machine (REMOTE §5, no broadcast). It is refused for a name the engine
/// implements too: an ambiguity the operator authored is a config defect to
/// tell them about, never a reason to quietly execute somewhere third.
#[test]
fn two_consenting_machines_are_an_ambiguity_refused_naming_them() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[json!({"ok": true, "kind": "clients", "rows": [
            {"client": "laptop", "present": true,
             "tools": [advertised("bash", true)]},
            {"client": "tower", "present": false,
             "tools": [advertised("bash", true)]},
        ]})],
    );
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("bash", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.contains("2 machines consent"), "{said}");
    assert!(said.contains("laptop, tower"), "{said}");
    assert!(said.contains("exactly one entry"), "{said}");
}

/// A consenting machine that never answers the routing leg is the transport
/// sentence every other ask renders — in band, non-zero, never a hang. The
/// scripted engine answers the roster and then goes silent, which is the
/// invoke ask running out its own bound.
#[test]
fn a_lane_whose_invoke_is_never_answered_refuses_in_band() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[roster("laptop", &[advertised("bash", true)])],
    );
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("bash", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("bash: "), "{said}");
    assert!(said.contains("no engine answered"), "{said}");
}

/// **The lane's first ask can fail too**, and it is refused in band on the same
/// terms the routing leg is: a lane whose *roster* read never lands has no set
/// to select from, so the transport's own sentence is what the model reads —
/// never a hang, and never a silent empty roster read, which would take the
/// engine rung on a workspace whose consenting machine simply went unread.
#[test]
fn a_lane_whose_roster_is_never_answered_refuses_in_band() {
    let root = TempDir::new().expect("tmp");
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let site = crate::tool_host::Site {
        state_root: root.path().to_path_buf(),
        workspace: "home".to_owned(),
        agent: "dulcet-mongoose".to_owned(),
        budget: crate::tool_host::tests::impatient(),
        patience: crate::tool_host::tests::impatient(),
        clock: FakeClock::new().arc(),
    };
    let capture = crate::tool_host::subject::answer(
        std::path::Path::new("/nonexistent/litany"),
        crate::tool_host::tests::impatient().span(),
        &site,
        &call!("bash", &input, &stop),
    );

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("bash: "), "{said}");
    assert!(said.contains("no engine answered"), "{said}");
    assert!(
        !said.contains("advertises"),
        "a failed roster read is a transport fact, not a selection one: {said}"
    );
}
