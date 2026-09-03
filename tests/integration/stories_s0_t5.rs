//! STORIES **S0-T5** login-stream: bz's one interactive surface run as the
//! streamed-piped class (§8's third class, §8.3 as amended). A fake `bz --login`
//! prints its sign-in lines **on stderr**, where the real bz writes them → the
//! streamed runner's [`LoginView`] carries them **verbatim**; ONE outcome row
//! lands in `ops.jsonl` at exit (§4.2, the stream never logged line-by-line); a
//! non-zero exit carries the exact run-by-hand command as the fallback (§8.3).
//! The spawn is the **browser** flow (`--browser`, bl-b4e5): the one flow every
//! oauth row can serve and the only one a GUI can drive. Credentials stay bz's —
//! yog renders.
//!
//! One `#[test]` runs both scenarios sequentially — a shape this file kept
//! from the era when a recorder-script write could race a peer's fork into
//! ETXTBSY. It cannot since bl-fd28: every fixture here is written by a child.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use std::path::Path;
use tempfile::tempdir;
use yog::cli_outbound::Cli;
use yog::login::{self, LoginRun, LoginView};

/// Poll the run to settlement (bounded — the fake exits promptly), returning the
/// terminal view-model the shell would paint.
fn drain(run: &mut LoginRun) -> LoginView {
    for _ in 0..400 {
        if !run.poll() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    run.view()
}

#[test]
fn s0_t5_login_streams_device_code_lines_and_converges_to_one_outcome_row() {
    let dir = tempdir().unwrap();

    // --- Clean device flow: the code/URL lines stream, exit 0, no fallback. ---
    let ok_state = tempdir().unwrap();
    let bz_ok = Recorder::new(dir.path(), "bz").on_err(
        "--login",
        "",
        "To authorize, visit https://example.test/auth\nWaiting for the redirect…\n",
        0,
    );
    let mut run = login::start(
        &Cli::new(bz_ok.path()),
        "openai",
        ok_state.path(),
        "T0",
        Some(Path::new("/ws")),
    )
    .unwrap();
    let view = drain(&mut run);

    // The sign-in lines are carried verbatim, in order (§8.3) — off stderr,
    // which is the only stream bz's login flow writes to (bl-b4e5 defect 3).
    assert_eq!(
        view.lines
            .iter()
            .map(|l| l.text.clone())
            .collect::<Vec<_>>(),
        [
            "To authorize, visit https://example.test/auth".to_owned(),
            "Waiting for the redirect…".to_owned(),
        ]
    );
    assert!(view.lines.iter().all(|l| l.err), "both came from stderr");
    assert_eq!(view.outcome, Some(0));
    assert_eq!(view.fallback, None, "a clean exit needs no fallback");

    // The spawn was the BROWSER flow — not bz's device-flow default, which
    // 78s on any row whose `oauth` block omits the optional `device_url`.
    let inv = bz_ok.invocations();
    assert_eq!(inv.len(), 1);
    assert_eq!(
        inv[0].argv,
        ["--login", "--provider", "openai", "--browser"]
    );

    // Exactly one outcome row at exit; the live stream is never re-logged.
    let bin = bz_ok.path().display().to_string();
    let ops = yog::opslog::tail(ok_state.path(), 8);
    assert_eq!(ops.len(), 1, "the stream converges to ONE outcome row");
    assert_eq!(ops[0].exit, 0);
    assert_eq!(
        ops[0].argv,
        [bin.as_str(), "--login", "--provider", "openai", "--browser"]
    );
    assert_eq!(ops[0].stdout, "", "the live stream is not re-logged (§4.2)");

    // --- Non-zero exit: the exact command rides back as the fallback (§8.3). ---
    let bad_state = tempdir().unwrap();
    // The live failure shape: bz's 78 with its remedy on stderr.
    let bz_bad = Recorder::new(dir.path(), "bz-bad").on_err(
        "--login",
        "",
        "this provider has no device endpoint; use `--browser`\n",
        78,
    );
    let mut run = login::start(
        &Cli::new(bz_bad.path()),
        "anthropic",
        bad_state.path(),
        "T0",
        Some(Path::new("/ws")),
    )
    .unwrap();
    let view = drain(&mut run);

    assert_eq!(view.outcome, Some(78));
    // The terminal reason reaches the PANE, not just the log (bl-b4e5 defect 3).
    assert!(
        view.lines
            .iter()
            .any(|l| l.text.contains("no device endpoint")),
        "bz's terminal reason line is rendered: {:?}",
        view.lines
    );
    let fallback = view
        .fallback
        .expect("a non-zero exit must carry the fallback command");
    assert!(
        fallback.ends_with("--login --provider anthropic --browser"),
        "the fallback is a command that would actually succeed, got: {fallback}"
    );

    let ops = yog::opslog::tail(bad_state.path(), 8);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].exit, 78);
    assert!(ops[0].stderr.contains("no device endpoint"));
}
