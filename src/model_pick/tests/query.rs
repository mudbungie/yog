//! The roster query (§9.4 / §5.1 #26): the JSON projection, every settle arm
//! driven deterministically through [`Streamed::from_rx`], and the two real
//! spawn arms of [`start`].

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::mpsc;

use tempfile::tempdir;

use crate::cli_outbound::{Chunk, Cli, ExitInfo, Streamed};
use crate::model_pick::query::{EMPTY_ROSTER, Roster, model_ids, start};

/// The exact payload `bz --list-models --provider codex --json` returns against
/// a live codex credential (§9.4, measured): ids and a default flag and nothing
/// else, because OpenAI's list GET serves no metadata — which is why yog
/// declares a default window for a row like this one.
const LIVE_PAYLOAD: &str = r#"{"models":[{"default":false,"id":"gpt-5.6-sol"},{"default":false,"id":"gpt-5.6-terra"},{"default":false,"id":"gpt-5.4"}]}"#;

/// The other shape, from a provider that DOES serve metadata (Google): the same
/// two keys plus brazen's three option-shaped ones. Both are lawful `Model`
/// rows — the keys are additive and per-provider, so the candidate list must be
/// read off `id` alone and never assume a shape.
const SERVED_PAYLOAD: &str = r#"{"models":[{"default":false,"id":"gemini-3-pro","context_window":1048576,"max_output_tokens":65536,"display_name":"Gemini 3 Pro"},{"default":false,"id":"gemini-3-flash"}]}"#;

/// Drive a roster to settlement over a hand-fed channel.
fn settled(chunks: Vec<Chunk>) -> Roster {
    let (tx, rx) = mpsc::channel();
    for chunk in chunks {
        tx.send(chunk).unwrap();
    }
    drop(tx);
    let mut roster = Roster::from_streamed(Streamed::from_rx(rx), "codex");
    while roster.poll() {}
    roster
}

fn stdout(text: &str) -> Chunk {
    Chunk::Stdout(text.as_bytes().to_vec())
}

#[test]
fn model_ids_projects_the_live_payload_in_order() {
    assert_eq!(
        model_ids(LIVE_PAYLOAD),
        ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.4"]
    );
    // The one key the payload carries besides `id` is not mistaken for one.
    assert!(!model_ids(LIVE_PAYLOAD).contains(&"false".to_string()));
}

/// A shapeless payload folds to no ids, never an error — the empty case is
/// already named by `EMPTY_ROSTER`.
#[test]
fn model_ids_folds_every_unusable_payload_to_nothing() {
    for payload in [
        "",
        "not json",
        "[]",
        r#"{"providers":[]}"#,
        r#"{"models":"nope"}"#,
        r#"{"models":[{"slug":"x"}]}"#,
    ] {
        assert!(model_ids(payload).is_empty(), "{payload}");
    }
}

/// A metadata-carrying roster reads back as ordinary ids — the extra keys are
/// additive, so the picker's candidate list is the same list either way.
#[test]
fn model_ids_reads_a_metadata_carrying_roster_the_same_way() {
    assert_eq!(
        model_ids(SERVED_PAYLOAD),
        ["gemini-3-pro", "gemini-3-flash"]
    );
}

#[test]
fn a_clean_run_settles_into_the_roster() {
    let roster = settled(vec![
        stdout(&format!("{LIVE_PAYLOAD}\n")),
        Chunk::Exited(ExitInfo::Code(0)),
    ]);
    let view = roster.view();
    assert!(!view.in_flight);
    assert_eq!(view.models, ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.4"]);
    assert_eq!(view.error, None);
    assert_eq!(view.fallback, None);
    assert_eq!(roster.provider(), "codex");
}

/// A payload arriving split across reads still parses — the picker reads whole
/// lines, and the JSON is one line.
#[test]
fn a_torn_payload_reassembles_before_it_is_parsed() {
    let (head, tail) = LIVE_PAYLOAD.split_at(20);
    let roster = settled(vec![
        stdout(head),
        stdout(tail),
        Chunk::Exited(ExitInfo::Code(0)),
    ]);
    assert_eq!(roster.view().models.len(), 3);
}

/// An exit-0 run offering nothing is named as itself, never rendered as a
/// picker with no rows (§7.3).
#[test]
fn an_empty_roster_is_named_not_shown_as_an_empty_list() {
    let roster = settled(vec![
        stdout("{\"models\":[]}\n"),
        Chunk::Exited(ExitInfo::Code(0)),
    ]);
    let view = roster.view();
    assert!(view.models.is_empty());
    assert_eq!(view.error.as_deref(), Some(EMPTY_ROSTER));
    assert!(
        view.fallback.is_some(),
        "the run-by-hand command is offered"
    );
}

#[test]
fn a_failing_run_surfaces_its_stderr_and_the_run_by_hand_command() {
    let roster = settled(vec![
        Chunk::Stderr(b"401 Unauthorized\n".to_vec()),
        Chunk::Exited(ExitInfo::Code(69)),
    ]);
    let view = roster.view();
    assert_eq!(view.error.as_deref(), Some("401 Unauthorized"));
    assert_eq!(view.fallback.as_deref(), Some("bz --list-models"));
    assert!(view.models.is_empty());
}

/// A failure that said nothing still renders as itself — the bare exit.
#[test]
fn a_silent_failure_renders_its_exit_code() {
    let roster = settled(vec![Chunk::Exited(ExitInfo::Signal(15))]);
    assert_eq!(
        roster.view().error.as_deref(),
        Some("bz --list-models exited 143")
    );
}

/// Settling is idempotent: a stray extra poll after the child exits never
/// re-settles or double-reports.
#[test]
fn polling_a_settled_roster_is_a_no_op() {
    let mut roster = settled(vec![
        stdout(&format!("{LIVE_PAYLOAD}\n")),
        Chunk::Exited(ExitInfo::Code(0)),
    ]);
    let before = roster.view();
    assert!(!roster.poll());
    assert_eq!(roster.view(), before);
}

/// A roster with nothing ready yet stays in flight — the frame the pulse is
/// painted on.
#[test]
fn a_pending_child_stays_in_flight() {
    let (tx, rx) = mpsc::channel();
    let mut roster = Roster::from_streamed(Streamed::from_rx(rx), "codex");
    assert!(roster.poll(), "nothing ready yet");
    assert!(roster.view().in_flight);
    // A whole line lands while the child is still running: accreted, not
    // settled — the roster only reads its payload at exit.
    tx.send(stdout("{\"models\":[\n")).unwrap();
    assert!(roster.poll(), "a line arrived, still live");
    assert!(roster.view().in_flight);
    assert!(roster.view().models.is_empty(), "nothing is offered yet");
    // A partial line stays buffered rather than surfacing torn.
    tx.send(stdout("partial")).unwrap();
    assert!(roster.poll(), "still live");
    drop(tx);
}

/// The real spawn path: `start` execs the argv §9.4 specifies and settles into
/// the roster the child printed.
#[test]
fn start_spawns_the_documented_argv_and_reads_its_roster() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fake-bz");
    let seen = dir.path().join("argv");
    fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > {}\nprintf '%s\\n' '{LIVE_PAYLOAD}'\n",
            seen.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let mut roster = start(&Cli::new(&path), "codex");
    while roster.poll() {}
    let view = roster.view();
    assert_eq!(view.error, None, "the child exited clean");
    assert_eq!(view.models, ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.4"]);
    assert_eq!(
        fs::read_to_string(&seen).unwrap().trim(),
        "--list-models --provider codex --json"
    );
}

/// A spawn that cannot happen is still a rendered fact (INV-2): `start` is
/// infallible by construction and hands back a settled failure view.
#[test]
fn start_folds_a_spawn_failure_into_the_error_view() {
    let roster = start(&Cli::new("/definitely/not/a/real/bz-xyz"), "codex");
    let view = roster.view();
    assert!(!view.in_flight);
    assert!(view.error.is_some(), "the spawn failure is rendered");
    assert_eq!(
        view.fallback.as_deref(),
        Some("/definitely/not/a/real/bz-xyz --list-models --provider codex --json")
    );
}
