//! **The whole span, over real loopback mTLS** (REMOTE §5, §9 step 7): an
//! agent's tool call is queued at the engine, crosses to a tool host on this
//! machine's own certificate, is run there, and the capture comes back to the
//! caller — through the one codec, the one `dispatch`/`answer`, and no verb the
//! wire added.
//!
//! The engine end is a real listener over freshly minted material, because a
//! certificate is the operator's out-of-channel act (REMOTE §1.4) and none is
//! ever committed. The driver end is the deposit inbox's own
//! [`ConsumerCtx::answer`], which is what an in-world caller is.

use super::*;
use crate::boundary::consumer::ConsumerCtx;
use crate::cli_outbound::Cli;
use crate::registry::mailbox::Mailbox;
use crate::registry::presence::Presence;
use crate::test_support::{wire::mint, world_under};
use crate::ui_state::SystemClock;
use crate::wire::intake::Intake;
use crate::wire::server::Listener;
use crate::wire::{material, seat};
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// The engine, as small as one can be and still be the real thing: the deposit
/// consumer's own context, which is what both intakes answer through.
fn ctx(state_root: &Path, mailbox: &Mailbox) -> Arc<ConsumerCtx> {
    Arc::new(ConsumerCtx {
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        lernie: Cli::new("/no/such/lernie"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        ui_path: state_root.join("ui.json"),
        cell: crate::state::new_snapshot_cell(Arc::new(crate::app::Snapshot::empty(0))),
        presence: Presence::default(),
        mailbox: mailbox.clone(),
        clock: Arc::new(SystemClock),
    })
}

/// A tool this machine can run: it echoes the invocation's own JSON back.
fn tool_config(dir: &Path) -> PathBuf {
    let tool = dir.join("echo-tool");
    fs::write(&tool, "#!/bin/sh\ncat\n").expect("script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("chmod");
    tool
}

/// A provisioned box: certificates minted the way an operator mints them, and
/// the one document that says what this machine can run.
fn provision(tmp: &TempDir) -> crate::xdg::Env {
    let world = world_under(tmp.path());
    mint(&material::dir(&world));
    let dir = world.yog_data_root();
    fs::write(
        dir.join(config::TOOLS),
        json!([{
            "name": "Bash",
            "description": "run a command",
            "input_schema": {"type": "object"},
            "command": [tool_config(&dir).to_string_lossy()],
        }])
        .to_string(),
    )
    .expect("config");
    world
}

/// **The span** — advertise, invoke, execute, capture — with a real socket in
/// the middle of it.
#[test]
fn an_invocation_crosses_to_a_tool_host_and_the_capture_comes_back() {
    let tmp = TempDir::new().expect("tmp");
    let state = tempdir().expect("state");
    let world = provision(&tmp);
    let mailbox = Mailbox::holding(60, Duration::from_millis(25));
    let engine = ctx(state.path(), &mailbox);

    let m = material::read(&world, material::Role::Server)
        .expect("material")
        .expect("provisioned");
    let listener = Listener::bind(
        &m,
        Arc::new(Intake::new(Arc::clone(&engine))),
        Presence::default(),
    )
    .expect("bound");
    fs::write(
        material::dir(&world).join(material::ADDRESS),
        listener.address(),
    )
    .expect("address");

    // The tool host, running as its own process would.
    let hosting = std::thread::spawn(move || serve(&world));

    // Its advertisement lands under the identity its certificate names, which
    // is what the driver's `invoke` checks against.
    let client = crate::registry::Client::parse("yog-client").expect("identity");
    let advertised = std::thread::scope(|_| {
        for _ in 0..400 {
            let set = crate::registry::tools::read(state.path(), &client);
            if !set.is_empty() {
                return set;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        Vec::new()
    });
    assert_eq!(
        advertised.first().map(|t| t.name.clone()),
        Some("Bash".to_owned()),
        "the advertisement is the config with the local half dropped"
    );

    // The driver's two gestures, through the door in-world callers use.
    let queued = engine.answer(&json!({"op": "invoke", "client": "yog-client",
                                       "tool": "Bash", "input": {"command": "ls"}}));
    assert_eq!(queued["ok"], true, "{queued}");
    let handle = queued["invocation"].as_str().unwrap_or_default().to_owned();
    assert!(!handle.is_empty(), "{queued}");

    let mut answered = json!(null);
    for _ in 0..400 {
        answered = engine.answer(&json!({"op": "capture", "invocation": handle}));
        if answered.get("capture").is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(
        answered["capture"],
        json!({"stdout": "{\"command\":\"ls\"}", "stderr": "", "exit_code": 0}),
        "the far machine's own three facts: {answered}"
    );

    // Dropping the engine's listener is how the host's next gesture fails, and
    // a failed gesture is how the mode exits — naming what stopped it.
    drop(listener);
    assert!(
        !hosting.join().expect("the host").is_empty(),
        "the loop's only exit is a gesture that failed, and it says which"
    );
}

/// A machine with no config is refused before it dials anything, and one with a
/// config and no certificates is refused at the seat — the same two refusals
/// `yog seat` gives, in the order the acts happen.
#[test]
fn an_unprovisioned_machine_is_refused_and_says_which_half_is_missing() {
    let tmp = TempDir::new().expect("tmp");
    let world = world_under(tmp.path());
    fs::create_dir_all(world.yog_data_root()).expect("data root");
    assert_eq!(run(&world, &[]), 1, "no config");

    fs::write(
        world.yog_data_root().join(config::TOOLS),
        json!([]).to_string(),
    )
    .expect("config");
    let e = serve(&world);
    assert!(e.contains(material::REMEDY), "{e}");
    assert_eq!(run(&world, &[]), 1);
}

/// The mode takes no arguments, and says so rather than ignoring them.
#[test]
fn arguments_are_a_usage_refusal() {
    let tmp = TempDir::new().expect("tmp");
    assert_eq!(
        run(&world_under(tmp.path()), &["--follow".to_owned()]),
        seat::USAGE_EXIT
    );
}

/// Every way an engine can answer something that is not this host's work, each
/// named rather than guessed at. The stand-in answerer is the point: a real
/// intake cannot produce these, and a client that could not tell them apart
/// would loop forever against an engine that had stopped making sense.
#[test]
fn an_answer_that_is_not_this_machines_work_names_itself() {
    struct Says(Vec<Value>);
    impl crate::wire::server::Answerer for Says {
        fn answer(
            &self,
            _client: &crate::registry::Client,
            _request: Value,
        ) -> Box<dyn Iterator<Item = Value>> {
            Box::new(self.0.clone().into_iter())
        }
    }

    for (said, needle) in [
        (Vec::new(), "closed the stream"),
        (
            vec![json!({"ok": true, "kind": "teleported"})],
            "undecodable",
        ),
        (vec![json!({"ok": false, "error": "no"})], "no"),
        (
            vec![json!({"ok": true, "kind": "acked"})],
            "not this machine's work",
        ),
    ] {
        let tmp = TempDir::new().expect("tmp");
        let world = provision(&tmp);
        let m = material::read(&world, material::Role::Server)
            .expect("material")
            .expect("provisioned");
        let listener =
            Listener::bind(&m, Arc::new(Says(said)), Presence::default()).expect("bound");
        fs::write(
            material::dir(&world).join(material::ADDRESS),
            listener.address(),
        )
        .expect("address");
        let e = serve(&world);
        assert!(e.contains(needle), "{e}");
    }
}

/// **A refused completion stops the host**, rather than being dropped on the
/// floor: an engine that will not take this machine's answers — an expired
/// handle, a slot addressed elsewhere — is one there is no point running
/// against. The stand-in answers each request in turn, so the failure lands on
/// the `complete` and nowhere earlier.
#[test]
fn a_completion_the_engine_refuses_stops_the_host() {
    struct InTurn {
        said: Vec<Value>,
        at: std::sync::atomic::AtomicUsize,
    }
    impl crate::wire::server::Answerer for InTurn {
        fn answer(
            &self,
            _client: &crate::registry::Client,
            _request: Value,
        ) -> Box<dyn Iterator<Item = Value>> {
            let at = self.at.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Box::new(std::iter::once(
                self.said
                    .get(at)
                    .cloned()
                    .unwrap_or(json!({"ok": false, "error": "nothing left to say"})),
            ))
        }
    }

    let tmp = TempDir::new().expect("tmp");
    let world = provision(&tmp);
    let engine = InTurn {
        said: vec![
            json!({"ok": true, "kind": "advertised"}),
            json!({"ok": true, "kind": "invocations",
                   "rows": [{"invocation": "inv-1", "tool": "Bash",
                             "input": {"command": "ls"}}]}),
            json!({"ok": false, "error": "no invocation \"inv-1\" is in flight"}),
        ],
        at: std::sync::atomic::AtomicUsize::new(0),
    };
    let m = material::read(&world, material::Role::Server)
        .expect("material")
        .expect("provisioned");
    let listener = Listener::bind(&m, Arc::new(engine), Presence::default()).expect("bound");
    fs::write(
        material::dir(&world).join(material::ADDRESS),
        listener.address(),
    )
    .expect("address");

    let stopped = serve(&world);
    assert!(
        stopped.contains("inv-1"),
        "the refusal rides back: {stopped}"
    );
}
