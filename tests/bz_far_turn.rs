//! **A real turn whose model call runs on another machine** (REMOTE §5.7,
//! bl-e0c9), driven through the shipped binary: `yog litany prompt` on a
//! workspace whose roles name `fix@foot`, litany spawning the world's `bz`
//! shim, the arm reading the qualification and asking through the real
//! `gestures/` inbox, and a fixture foot running a slow child and posting what
//! it writes while it writes it.
//!
//! **What stands in, and what does not.** litany, the `bz` arm, the inbox, the
//! mailbox and the reply codec are the real ones. The server's intake is this
//! file's: a thread that answers the arm's two gestures out of a real
//! [`Mailbox`] — the dispatch arms themselves are `boundary::routing`'s unit
//! tests' — and the foot reaches that mailbox directly rather than over TLS,
//! which `wire::listener`'s suite proves and thrall owns the client of.
//!
//! The child holds half its stream back until this test has **seen
//! `response.json` grow**, so "the stream arrives before the child exits" is
//! an ordering the test enforces rather than a race it hopes to win.
#![allow(clippy::unwrap_used)]

#[path = "bz_far_turn/fixture.rs"]
mod fixture;

use std::io::Write as _;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use fixture::{events, far_bz, intake, responses, seat_foot, serve, workspace, yog};
use serde_json::json;
use yog::registry::mailbox::Mailbox;

#[test]
fn a_turn_streams_from_another_machine_and_commits_like_a_local_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    let home = tmp.path();
    let ws = workspace(home);
    let ws_s = ws.display().to_string();

    // The foot: registered in this workspace, advertising `bz`. Its entry
    // writes four events, waits to be released, then writes the rest.
    let world = home.join("xdg_data_home/yog/world");
    let state_root = world.join("state/yog");
    seat_foot(&state_root);
    let (lines, release) = (events(), home.join("release"));
    let script = far_bz(home, &lines, &release);

    let (mail, done) = (
        Mailbox::holding(5, Duration::from_millis(20)),
        Arc::new(AtomicBool::new(false)),
    );
    let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
    let intake = intake(&mail, &state_root, &done, &Arc::new(AtomicUsize::new(0)));
    let host = {
        let (mail, done, asked) = (mail.clone(), Arc::clone(&done), Arc::clone(&asked));
        std::thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                serve(&mail, &script, &asked);
            }
        })
    };

    // The turn. The wall is what the server stands on every spawn (§16.2).
    let wall = world.join("walls/ws");
    let turn = yog(home, &[("YOG_WALL", &wall)])
        .args(["litany", "prompt", &ws_s, "say where you are"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // `response.json` holds part of the stream while the far child is still
    // alive — it cannot exit before the release this observation writes.
    let deadline = Instant::now() + Duration::from_mins(2);
    let partial = loop {
        if let Some(seen) = responses(&ws).into_iter().find(|(_, size)| *size > 0) {
            break seen;
        }
        assert!(Instant::now() < deadline, "no response.json grew");
        std::thread::sleep(Duration::from_millis(20));
    };
    let early = std::fs::read_to_string(&partial.0).unwrap();
    assert!(
        early.contains("message_start") && !early.contains("\"end\""),
        "{early}"
    );
    std::fs::File::create(&release).unwrap().flush().unwrap();

    let out = turn.wait_with_output().unwrap();
    done.store(true, Ordering::Relaxed);
    intake.join().unwrap();
    host.join().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // What crossed is the command line with the row unqualified, and the
    // request litany assembled as its standard input.
    let asked = asked.lock().unwrap();
    let input = asked.first().expect("the foot was handed a call");
    assert_eq!(input["args"], json!(["--json", "--provider", "fix"]));
    assert!(
        input["stdin"]
            .as_str()
            .unwrap()
            .contains("say where you are")
    );

    // The step is litany's own: the whole stream, the usage, and a commit.
    let whole = std::fs::read_to_string(&partial.0).unwrap();
    assert_eq!(
        whole.lines().collect::<Vec<_>>(),
        lines,
        "relayed byte for byte"
    );
    let agent = String::from_utf8(out.stdout).unwrap().trim().to_owned();
    let tree = yog::git_env::command(Path::new("git"))
        .args(["-C", &ws.join("repo.git").display().to_string()])
        .args(["log", "-p", &format!("refs/heads/agents/{agent}")])
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&tree.stdout);
    assert!(
        log.contains("from the far machine"),
        "the reply is committed: {log}"
    );
    let meta =
        std::fs::read_to_string(ws.join("steps").join(&agent).join("001/meta.json")).unwrap();
    assert!(
        meta.contains("fix@foot"),
        "the step records the qualified row: {meta}"
    );
    assert!(meta.contains("\"commit\": \""), "the step commits: {meta}");
    assert!(
        log.contains(r#""usage":{"input_tokens":11,"output_tokens":7}"#),
        "the far machine's usage is the step's: {log}"
    );
}

/// **A stopped turn stops the polls** (REMOTE §5.7, §3.1). A stop signals the
/// driver's process group, and the arm installs no handler: it dies where it
/// stands, and the far run — whose asker no longer listens — is refused its
/// next post. Here: an arm polling for a call nobody serves is sent `SIGTERM`,
/// and the inbox sees no poll after it.
#[test]
fn a_stopped_arm_asks_nothing_more() {
    let tmp = tempfile::TempDir::new().unwrap();
    let state_root = tmp.path().join("state/yog");
    seat_foot(&state_root);
    let (mail, done) = (Mailbox::default(), Arc::new(AtomicBool::new(false)));
    let polls = Arc::new(AtomicUsize::new(0));
    let intake = intake(&mail, &state_root, &done, &polls);
    let mut arm = yog(
        tmp.path(),
        &[
            ("XDG_STATE_HOME", &tmp.path().join("state")),
            ("YOG_WALL", Path::new("/nonexistent/walls/ws")),
        ],
    )
    .args(["bz", "--json", "--provider", "fix@foot"])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
    let deadline = Instant::now() + Duration::from_mins(1);
    while polls.load(Ordering::Relaxed) < 2 {
        assert!(Instant::now() < deadline, "the arm never polled");
        std::thread::sleep(Duration::from_millis(20));
    }
    let term = yog::git_env::command(Path::new("kill"))
        .args(["-TERM", &arm.id().to_string()])
        .status()
        .unwrap();
    assert!(term.success());
    assert!(
        !arm.wait().unwrap().success(),
        "a stopped arm does not exit clean"
    );
    let stood = polls.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(
        polls.load(Ordering::Relaxed),
        stood,
        "no poll after the stop"
    );
    done.store(true, Ordering::Relaxed);
    intake.join().unwrap();
}
