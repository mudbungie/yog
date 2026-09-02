//! The consumer thread's tables (§8.5): the pass over the latest published
//! snapshot, and the one thing only a real thread can prove — spawn, consume,
//! stop. The fixtures the REMOTE §4 half builds its worlds from live here too.

/// **Birth is a barrier** (bl-6c9e): the two-call composition every documented
/// start flow is, at both intakes. Its own file beside [`scope`] for that
/// file's reason — a real seam, and the one the drive reproduced on.
mod birth;
/// The REMOTE §4.2 half (bl-7ff3): the certificate grade raising at this same
/// chokepoint — what a foot may say, and the sentence everything else earns.
mod grade;
/// The scoped intake (REMOTE §4, bl-8bbc): what a connection enumerates, what
/// an unregistered name earns, and the create that seats its own client. Its
/// own file at §12's cap — a real seam, because everything above is the
/// in-world intake and everything there is the wire's.
mod scope;
/// The REMOTE §5 half (bl-4e08): the intake threading its identity to the ACT
/// side, and the roster read that joins the three facts back.
mod tools;

use super::*;
use crate::boundary::deposit;
use crate::cli_outbound::Cli;
use crate::ui_state::SystemClock;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn ctx(state_root: &std::path::Path) -> ConsumerCtx {
    over(
        state_root,
        crate::app::Snapshot::empty(0),
        PathBuf::from("/data"),
        Cli::new("/no/such/litany"),
    )
}

/// A context over a stated snapshot, world root and `litany` — everything the
/// REMOTE §4 tests below vary.
fn over(
    state_root: &std::path::Path,
    snap: crate::app::Snapshot,
    yog_data_root: PathBuf,
    litany: Cli,
) -> ConsumerCtx {
    over_world(
        state_root,
        snap,
        yog_data_root,
        litany,
        crate::test_support::no_world(),
    )
}

/// The same, over a **stated world** — what the §5.1 #1 project enumeration is
/// read out of (bl-3377). Nearly every test here wants the hermetic
/// `no_world()`, whose clones dir does not exist and therefore enumerates
/// nothing; the project-birth barrier is the one that needs a real one.
fn over_world(
    state_root: &std::path::Path,
    snap: crate::app::Snapshot,
    yog_data_root: PathBuf,
    litany: Cli,
    world: crate::xdg::Env,
) -> ConsumerCtx {
    ConsumerCtx {
        yog_binary: PathBuf::from("/no/such/yog"),
        world,
        litany,
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root,
        balls_state_root: PathBuf::from("/balls"),
        ui_path: state_root.join("ui.json"),
        cell: crate::state::new_snapshot_cell(std::sync::Arc::new(snap)),
        presence: crate::registry::presence::Presence::default(),
        mailbox: crate::registry::mailbox::Mailbox::default(),
        clock: Arc::new(SystemClock),
    }
}

/// A world holding one named workspace per element of `names` under `root` —
/// **on disk and in the snapshot both**, because since bl-6c9e the intake
/// resolves over the §3.1 enumeration rather than over the cached copy. A
/// fixture that claimed a workspace no root holds was claiming something no
/// gesture could ever have addressed.
fn world_of(root: &std::path::Path, names: &[&str]) -> crate::app::Snapshot {
    let mut snap = crate::app::Snapshot::empty(0);
    snap.workspaces = names
        .iter()
        .map(|name| {
            let path = crate::binding::workspace_path(root, name);
            std::fs::create_dir_all(path.join("repo.git")).expect("a workspace on disk");
            crate::binding::Workspace {
                path,
                kind: crate::binding::WorkspaceKind::Named {
                    name: (*name).to_owned(),
                },
            }
        })
        .collect();
    snap
}

/// A `litany` that materializes what the real one does for a start's substrate
/// steps — the world's seed marker and the workspace's config branch — and
/// nothing else. `prime` is short-circuited by [`seed`] ahead of it.
fn fake_litany(dir: &std::path::Path) -> Cli {
    use std::os::unix::fs::PermissionsExt;
    let body = format!(
        "#!/bin/sh\ncase \"$1\" in\n{arm}esac\nexit 0\n",
        arm = crate::test_support::authoring_new_arm()
    );
    let path = dir.join("litany");
    std::fs::write(&path, body).unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// Lay the world's seed marker so `prime` short-circuits (§16.6 W3).
fn seed(yog_data_root: &std::path::Path) {
    let litany = crate::world::layout_under(yog_data_root).litany;
    std::fs::create_dir_all(&litany).unwrap();
    std::fs::write(litany.join("models.yaml"), b"models: {}\n").unwrap();
}

/// The workspace names a `workspaces` reply lists.
fn listed(reply: &serde_json::Value) -> Vec<String> {
    reply["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|r| r["workspace"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn client(name: &str) -> crate::registry::Client {
    crate::registry::Client::parse(name).expect("a usable identity")
}

/// A peer of operator grade (REMOTE §4.2) — every seat, and every certificate
/// minted before the grade existed. The default the scope tests are written
/// against; the foot's own raise is [`grade`]'s.
fn operator(client: crate::registry::Client) -> crate::registry::Peer {
    crate::registry::Peer {
        client,
        grade: crate::registry::Grade::Operator,
    }
}

/// [`operator`] over a named identity — the everyday spelling.
fn seat(name: &str) -> crate::registry::Peer {
    operator(client(name))
}

#[test]
fn an_empty_inbox_costs_one_listing_and_nothing_else() {
    let root = tempdir().unwrap();
    assert_eq!(ctx(root.path()).pass(), 0);
}

#[test]
fn a_pass_answers_from_the_latest_published_snapshot() {
    let root = tempdir().unwrap();
    deposit::deposit(root.path(), "q-1", &json!({"op": "workspaces"})).unwrap();
    assert_eq!(ctx(root.path()).pass(), 1);
    let reply = deposit::read_reply(root.path(), "q-1").unwrap();
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["kind"], "workspaces");
    assert_eq!(
        reply["rows"].as_array().unwrap().len(),
        0,
        "the empty snapshot"
    );
}

/// The wire's intake is this same context (REMOTE §3, bl-b6fa): an envelope
/// handed straight in is answered exactly as a deposited one is, which is what
/// makes the listener a second intake rather than a second implementation.
#[test]
fn one_envelope_is_answered_where_a_deposit_is() {
    let root = tempdir().unwrap();
    let reply = ctx(root.path()).answer(&json!({"op": "workspaces"}));
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["kind"], "workspaces");
    // And a torn envelope refuses in-band rather than wedging the caller.
    let refusal = ctx(root.path()).answer(&json!({"op": "enhance"}));
    assert_eq!(refusal["ok"], false);
}

#[test]
fn the_thread_consumes_a_deposit_and_stops_on_drop() {
    let root = tempdir().unwrap();
    deposit::deposit(root.path(), "q-t", &json!({"op": "balls"})).unwrap();
    let consumer = Consumer::spawn(Arc::new(ctx(root.path())));
    let deadline = Instant::now() + Duration::from_secs(10);
    while deposit::read_reply(root.path(), "q-t").is_none() {
        assert!(Instant::now() < deadline, "the consumer never answered");
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(consumer); // joins cleanly — the Drop is the shutdown
}
