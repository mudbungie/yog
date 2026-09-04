//! The compactor's procedure pair, performed as engine acts (REMOTE §5.4,
//! bl-dfce): what reaches the engine's front door, what comes back, and the two
//! bounded waits.

use super::*;
use crate::boundary::deposit;
use crate::test_support::FakeClock;
use crate::tool_host::tests::{budget, front_door, impatient, tool};
use crate::tool_host::{Injection, loaded};
use serde_json::json;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

use ::litany::cmd::ToolInjection as _;

/// One engine act, as litany's executor hands it over. A macro rather than a
/// function because three independent borrows cannot be elided and a named
/// lifetime is banned (AGENTS.md rule 1).
macro_rules! act {
    ($name:expr, $workspace:expr, $input:expr, $stop:expr) => {
        RoutedCall {
            id: "toolu_9",
            name: $name,
            input: $input,
            workspace: $workspace,
            agent: "dulcet-mongoose",
            cwd: $workspace,
            stop: $stop,
        }
    };
}

/// **The closed set itself** (bl-fe43): which names are engine acts, and the
/// two proofs that a machine cannot take one — the enrolled thrall whose
/// mailbox stays empty, and the id every spawn carries. Its own file at §12's
/// per-file budget, sharing this module's fixtures and `act!`.
mod set;

/// **The pair reaches the engine's own front door**, with the verb litany
/// answers built-ins under, the caller identity off the CALL rather than off
/// this process, the `tool_use` input on stdin — and the child's product back
/// untouched. Nothing of the compactor's semantics is restated by yog: the
/// front door is the one definition.
#[test]
fn the_pair_is_performed_at_the_engines_own_front_door() {
    let root = TempDir::new().expect("tmp");
    // A sleep first, so the drain is entered with nothing buffered and the
    // waiting arm is the one that runs — the ordinary case, not a corner.
    let door = front_door(
        root.path(),
        "sleep 0.1\nprintf '%s|%s|%s|%s|%s' \"$1\" \"$2\" \
         \"$LITANY_CONV_REPO\" \"$LITANY_CONV_BRANCH\" \"$(cat)\"",
    );
    let input = json!({"content": "a summary"});
    let stop = AtomicBool::new(false);

    let capture = perform(
        &door,
        budget().span(),
        &act!("write_summary", root.path(), &input, &stop),
    );

    assert_eq!(capture.exit_code, 0);
    assert!(capture.stderr.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&capture.stdout),
        format!(
            "tool|write_summary|{}|dulcet-mongoose|{{\"content\":\"a summary\"}}",
            root.path().display()
        )
    );
}

/// A front door that refused answers as itself — its own exit code and its own
/// stderr, passed through — because a compactor tool that declines is an
/// ordinary in-band result the model reads, not a fault of yog's.
#[test]
fn a_refusing_front_door_comes_back_verbatim() {
    let root = TempDir::new().expect("tmp");
    let door = front_door(
        root.path(),
        "printf 'the dispatch entry may not be nominated' >&2\nexit 7",
    );
    let input = json!({"path": "messages/001-user.md"});
    let stop = AtomicBool::new(false);

    let capture = perform(
        &door,
        budget().span(),
        &act!("mark_for_deletion", root.path(), &input, &stop),
    );

    assert_eq!(capture.exit_code, 7);
    assert!(capture.stdout.is_empty());
    assert_eq!(capture.stderr, b"the dispatch entry may not be nominated");
}

/// A front door that cannot be spawned at all is yog's own sentence, and it is
/// in band and non-zero — never a hang and never a harness fault.
#[test]
fn an_unspawnable_front_door_refuses_in_band() {
    let root = TempDir::new().expect("tmp");
    let input = json!({});
    let stop = AtomicBool::new(false);

    let capture = perform(
        &root.path().join("no-such-front-door"),
        budget().span(),
        &act!("write_summary", root.path(), &input, &stop),
    );

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("write_summary: "), "{said}");
}

/// Both waits are bounded, and each is its own sentence: a stop landing while
/// the engine works ends it at once, and a deadline running out ends it too.
/// Returning drops the stream, and the drop is the kill.
#[test]
fn a_working_engine_is_ended_by_the_stop_flag_and_by_the_deadline() {
    let root = TempDir::new().expect("tmp");
    let door = front_door(root.path(), "sleep 30");
    let input = json!({});

    let stopped = AtomicBool::new(true);
    let capture = perform(
        &door,
        budget().span(),
        &act!("write_summary", root.path(), &input, &stopped),
    );
    assert_eq!(capture.exit_code, 1);
    assert!(
        String::from_utf8_lossy(&capture.stderr).contains("stopped while the engine was working"),
        "{:?}",
        String::from_utf8_lossy(&capture.stderr)
    );

    let running = AtomicBool::new(false);
    let capture = perform(
        &door,
        impatient().span(),
        &act!("write_summary", root.path(), &input, &running),
    );
    assert_eq!(capture.exit_code, 1);
    assert!(
        String::from_utf8_lossy(&capture.stderr).contains("did not answer within"),
        "{:?}",
        String::from_utf8_lossy(&capture.stderr)
    );
}

/// **The four conversation-subject grants perform at the same front door**
/// (bl-77be) — the defect's own probe, answered: `dispatch`, `message`,
/// `load_skill` and `cd` each reach `<driver_target> tool <name>` with the
/// caller identity on the environment and the input on stdin, where before
/// the audit every one of them earned the loadless refusal. The engine's own
/// semantics stay upstream's; what is proven here is that the router now
/// takes these names to the engine instead of to a machine or a refusal.
#[test]
fn the_worker_grants_perform_at_the_engines_own_front_door() {
    let root = TempDir::new().expect("tmp");
    let door = front_door(
        root.path(),
        "printf '%s|%s|%s|%s|%s' \"$1\" \"$2\" \
         \"$LITANY_CONV_REPO\" \"$LITANY_CONV_BRANCH\" \"$(cat)\"",
    );
    let stop = AtomicBool::new(false);
    for (name, input) in [
        ("dispatch", json!({"role": "worker", "goal": "g"})),
        ("message", json!({"agent": "amber", "content": "hi"})),
        ("load_skill", json!({"name": "review"})),
        ("cd", json!({"path": "sub"})),
    ] {
        let capture = perform(
            &door,
            budget().span(),
            &act!(name, root.path(), &input, &stop),
        );
        assert_eq!(capture.exit_code, 0, "{name}");
        assert_eq!(
            String::from_utf8_lossy(&capture.stdout),
            format!(
                "tool|{name}|{}|dulcet-mongoose|{}",
                root.path().display(),
                input
            ),
            "{name} crosses the front door with the caller's identity"
        );
    }
    assert!(
        deposit::pending(root.path()).is_empty(),
        "an engine act queues nothing at any machine's mailbox"
    );
}
