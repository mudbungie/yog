//! bl-44d8 — the two terminal start steps, composed across two processes.
//!
//! `/prepare` and `/prompt` are the §8.1 start family's two gestures, and the
//! help contract presents them as consecutive steps. Every `yog gesture` is its
//! own process, so the [`Prepared`](crate::start::Prepared) the first returns
//! had no way into the second: `line::Context::prepared` was invocation-local
//! and argv could not state it, which made the only advertised next step
//! impossible from a terminal. `--prepared` is that statement — the seat says
//! what it elides, exactly as `--ws`/`--agent`/`--project`/`--as` do — and it is
//! read by the same codec that wrote the reply, so no second start
//! implementation exists.

use super::snapshot;
use crate::boundary::dispatch::Deps;
use crate::boundary::{consume::consume, deposit, sugar};
use crate::cli_outbound::Cli;
use crate::test_support::world::no_world;
use crate::test_support::{authoring_new_arm, spawn_guard};
use crate::ui_state::UiState;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A fake `lernie` that materializes what the real one does — the seed marker
/// and the workspace's config branch — and, on `prompt`, reports the goal it
/// was fired with through a fifo, so the detached child is *observed* rather
/// than assumed. It reads the goal as the **last** argument rather than a fixed
/// position, which is the §4.2 invariant `clip_goal` rests on and which the
/// bl-6654 `--cwd` binding rides in front of.
fn fake_lernie(dir: &Path, fifo: &Path) -> PathBuf {
    let body = format!(
        "#!/bin/sh\ncase \"$1\" in\nprime) [ -n \"$LERNIE_HOME\" ] && mkdir -p \"$LERNIE_HOME\" \
         && : > \"$LERNIE_HOME/models.yaml\";;\n{arm}prompt) for a; do last=\"$a\"; done; printf '%s' \"$last\" > '{fifo}';;\
         \nesac\nexit 0\n",
        arm = authoring_new_arm(),
        fifo = fifo.display(),
    );
    let path = dir.join("lernie");
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

fn make_fifo(path: &Path) {
    let status = crate::git_env::command(Path::new("mkfifo"))
        .args(["-m", "600"])
        .arg(path)
        .status()
        .expect("spawn mkfifo");
    assert!(status.success(), "mkfifo");
}

/// The two invocations, run for real over one world: `/prepare dir …` then
/// `/prompt …`, the second carrying nothing but the first's own reply.
#[test]
fn a_prepared_reply_fires_the_next_invocations_prompt() {
    let _g = spawn_guard();
    let bin = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let yog = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let balls = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let fifo = bin.path().join("report");
    make_fifo(&fifo);
    let ws = crate::binding::workspace_path(yog.path(), "alba");
    let lernie_home = crate::world::layout_under(yog.path()).lernie;
    let deps = Deps {
        lernie: Cli::new(fake_lernie(bin.path(), &fifo)).with_env(vec![(
            "LERNIE_HOME".to_owned(),
            lernie_home.to_string_lossy().into_owned(),
        )]),
        bl: Cli::new("/no/such/bl"),
        state_root: state.path().to_path_buf(),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: no_world(),
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        snapshot: Arc::new(snapshot(&ws, "alba", vec![], vec![])),
        mint_seed: 7,
    };
    let mut ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));

    // Invocation one: the mutating half. Its reply is the whole start.
    let prepare = sugar::run(
        state.path(),
        &[
            "--ws".to_owned(),
            ws.to_string_lossy().into_owned(),
            format!("/prepare dir {}", repo.path().display()),
        ],
        "p",
        2,
        &mut || {
            consume(&deps, &mut ui, "T1", 100);
        },
    );
    assert_eq!(prepare, 0, "the seed/workspace steps ran");
    let reply = deposit::read_reply(state.path(), "p-0").unwrap();
    assert_eq!(reply["kind"], "prepared");
    assert_eq!(
        reply["prepared"]["binding"].as_str(),
        Some(repo.path().to_string_lossy().as_ref()),
        "the typed work target survives the process boundary (§3.3, bl-6654)"
    );

    // Invocation two: a different process, holding nothing but those bytes.
    let prompt = sugar::run(
        state.path(),
        &[
            "--prepared".to_owned(),
            reply["prepared"].to_string(),
            "/prompt build the greeting script and tests".to_owned(),
        ],
        "q",
        2,
        &mut || {
            consume(&deps, &mut ui, "T2", 101);
        },
    );
    assert_eq!(prompt, 0, "the deferred fire landed");
    let started = deposit::read_reply(state.path(), "q-0").unwrap();
    assert_eq!(started["kind"], "started");
    assert!(started["conversation"].is_string(), "one drone was born");
    assert_eq!(
        fs::read_to_string(&fifo).unwrap(),
        "build the greeting script and tests",
        "exactly one detached `lernie prompt`, with the goal verbatim"
    );
}

/// The flag states a `Prepared`, not a wish: anything else refuses at the
/// depositor, naming why, and the inbox stays clean (§8.5's strict edge).
#[test]
fn a_prepared_flag_that_states_no_prepared_never_deposits() {
    let root = tempfile::tempdir().unwrap();
    for stated in ["{", r#"{"name": "alba"}"#] {
        assert_eq!(
            sugar::run(
                root.path(),
                &[
                    "--prepared".to_owned(),
                    stated.to_owned(),
                    "/prompt go".to_owned(),
                ],
                "g",
                1,
                &mut || panic!("must not wait"),
            ),
            sugar::USAGE_EXIT,
            "{stated}"
        );
    }
    assert!(deposit::pending(root.path()).is_empty());
}
