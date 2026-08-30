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
        litany: Cli::new("/no/such/litany"),
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

/// **A channel it cannot open is said once, and its neighbours are served**
/// (REMOTE §8.2). The box holds one good channel and one half-provisioned
/// entry: the entry's refusal goes to stderr on its own, and what the mode
/// answers with is the good channel's own sentence — never the entry's, which
/// would make one bad directory look like the whole host failing.
#[test]
fn a_refused_entry_does_not_refuse_the_host() {
    let tmp = TempDir::new().expect("tmp");
    let world = provision(&tmp);
    let root = material::dir(&world);
    fs::write(
        root.join(material::ADDRESS),
        crate::test_support::wire::NO_LISTENER,
    )
    .expect("address");
    let bad = root.join(crate::wire::entries::ENTRIES).join("bad");
    fs::create_dir_all(&bad).expect("mkdir");
    fs::write(bad.join(material::ANCHORS), "-----PEM-----\n").expect("write");

    let stopped = serve(&world);
    assert!(
        stopped.starts_with("connect ") && !stopped.contains("half-provisioned"),
        "the served channel's own sentence, unlabelled: {stopped}"
    );
}

/// **What an engine can answer that is not this machine's work** — its own
/// corpus at §12's cap, on the seam this file's own doc draws: above is the
/// span and what this box holds, and there is what comes back down it.
mod engine;
