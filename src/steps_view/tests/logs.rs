//! bl-83d6: the drill-in's **capture-log seats** — the step's own `stderr.log`
//! and the agent's `driver.log`, browsable in full where the §7.3 banners quote
//! three lines of them.
//!
//! Two claims, both driven from the on-disk shape: the row set is **derived** —
//! a log with nothing in it is offered no seat, so a healthy step's picker is
//! unchanged — and a log that *is* there reaches the paint output whole, at the
//! bounded-file cap every other reading surface uses rather than a tail.

use std::collections::HashSet;

use super::{AGENT, write_file};
use crate::files_view::{PREVIEW_CAP, Preview};
use crate::steps_view::render::{StepTab, render};
use crate::steps_view::{StepDetail, StepsView, UNPARSED, detail, seats};

/// The observed shape a wound leaves: a request, an opened-and-empty response,
/// and the adapter's words on stderr.
const ADAPTER: &str = "bz: no workspace in this environment — providers, sign-ins and the model \
cache belong to a workspace.";

/// What a driver that declined at the boundary writes to `driver.log`.
const DRIVER: &str = "lernie: provider refused the turn — unpaired tool_use in the tail";

fn painted(detail: &StepDetail, tab: StepTab, raw: bool) -> String {
    let mut collapsed = HashSet::new();
    crate::paint_probe::paint(|ui| {
        render(
            ui,
            &StepsView::default(),
            None,
            Some(detail),
            tab,
            &mut collapsed,
            raw,
        );
    })
}

/// `driver.log` sits **beside** the step dirs, one per agent (ARCH §2.11), so it
/// is written a level above every step's own records.
fn write_driver_log(ws: &std::path::Path, bytes: &[u8]) {
    let dir = ws.join("steps").join(AGENT);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("driver.log"), bytes).unwrap();
}

/// A step that failed the way bl-55d8's did, with a driver that also left words.
fn wounded() -> (tempfile::TempDir, StepDetail) {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "request.json", br#"{"model":"opus"}"#);
    write_file(ws, "001", "response.json", b"");
    write_file(ws, "001", "stderr.log", ADAPTER.as_bytes());
    write_driver_log(ws, DRIVER.as_bytes());
    let d = detail(ws, AGENT, "001");
    (dir, d)
}

#[test]
fn a_step_with_captured_words_carries_both_logs_whole() {
    let (_dir, d) = wounded();
    assert_eq!(d.stderr, Some(Preview::Text(ADAPTER.to_owned())));
    assert_eq!(d.driver, Some(Preview::Text(DRIVER.to_owned())));
    // The picker's row set grows by exactly the logs that have bytes, and they
    // come last — the five contract records never move under an operator.
    let offered: Vec<&str> = seats(Some(&d)).iter().map(|(_, label, _)| *label).collect();
    assert_eq!(
        offered,
        [
            "meta",
            "request",
            "staging",
            "response",
            "tools",
            "stderr.log",
            "driver.log"
        ],
        "the derived row set"
    );
    // Every seat says what it is, logs included (§11 discoverability).
    for (_, label, hint) in seats(Some(&d)) {
        assert!(!hint.is_empty(), "unexplained seat {label}");
    }
}

#[test]
fn an_ordinary_step_is_offered_no_log_seat() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    // The ordinary run: lernie opens `stderr.log` every attempt and the adapter
    // says nothing in it (lernie ARCH §2.3), and this agent's drivers have
    // written no log at all.
    write_file(ws, "001", "meta.json", br#"{"commit":"c0ffee"}"#);
    write_file(ws, "001", "stderr.log", b"");
    let d = detail(ws, AGENT, "001");
    assert_eq!(d.stderr, None, "an empty log is nothing to read");
    assert_eq!(d.driver, None, "an absent log is the same fact");
    assert_eq!(seats(Some(&d)).len(), 5, "no dead rows on a healthy step");
    // And with no answer in hand at all — a step picked one round trip ago —
    // the strip is the five contract records, which need no answer to be named.
    assert_eq!(seats(None).len(), 5);
    let text = painted(&d, StepTab::Meta, false);
    assert!(!text.contains("stderr.log"), "seatless:\n{text}");
    assert!(!text.contains("driver.log"), "seatless:\n{text}");
}

#[test]
fn each_log_seat_paints_its_own_file_in_full() {
    let (_dir, d) = wounded();
    let stderr = painted(&d, StepTab::Stderr, false);
    assert!(
        stderr.contains("no workspace in this environment"),
        "{stderr}"
    );
    // The picker offers both seats by their file names, and names the open one.
    assert!(stderr.contains("stderr.log"), "unseated:\n{stderr}");
    assert!(stderr.contains("driver.log"), "unseated:\n{stderr}");
    let driver = painted(&d, StepTab::Driver, false);
    assert!(driver.contains("unpaired tool_use in the tail"), "{driver}");
    // A log was never parsed, so the malformed-JSON row must not frame it and
    // Raw has nothing to escape from: the same bytes, either way (§11).
    assert!(
        !stderr.contains(UNPARSED),
        "a log is not broken JSON:\n{stderr}"
    );
    assert_eq!(painted(&d, StepTab::Stderr, true), stderr);
    assert_eq!(painted(&d, StepTab::Driver, true), driver);
}

/// The seat the ball is about: a driver error too long to read in a banner. It
/// is shown at the bounded-file cap — with the cap said outright — rather than
/// clipped to a tail that leaves the operator guessing there is more.
#[test]
fn a_long_log_is_shown_to_the_bounded_file_cap_and_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "response.json", b"");
    let long = "a driver error line that runs on and on\n".repeat(2000);
    assert!(long.len() > PREVIEW_CAP, "the fixture must exceed the cap");
    write_driver_log(ws, long.as_bytes());
    let mut d = detail(ws, AGENT, "001");
    let Some(Preview::Truncated { text, size }) = &d.driver else {
        panic!(
            "a log past the cap is truncated, not silently clipped: {:?}",
            d.driver
        );
    };
    assert_eq!(text.len(), PREVIEW_CAP);
    assert_eq!(*size, long.len() as u64);
    assert!(
        painted(&d, StepTab::Driver, false).contains("a driver error line"),
        "the bytes reach the seat"
    );
    // The cap is *said*, at this seat as at the Files tab's — asserted on a
    // short truncation because 64 KiB of monospace pushes the sentence off the
    // probe's viewport (it is one scroll down in the window, inside the tab's
    // own `tail::scroll`), and a paint assertion must witness what it claims.
    d.driver = Some(Preview::Truncated {
        text: "leading".into(),
        size: 99_999,
    });
    let said = painted(&d, StepTab::Driver, false);
    assert!(said.contains("leading"), "got:\n{said}");
    assert!(said.contains("preview truncated at 64 KiB"), "got:\n{said}");
    assert!(said.contains("99999"), "the whole file's size:\n{said}");
}

/// A seat held across a step that has the log to one that does not: no row is
/// offered, and the body says the same word every other empty record says.
#[test]
fn a_log_seat_held_onto_an_empty_step_says_absent() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "meta.json", b"{}");
    let d = detail(ws, AGENT, "001");
    for tab in [StepTab::Stderr, StepTab::Driver] {
        let text = painted(&d, tab, false);
        assert!(text.contains("(absent)"), "got:\n{text}");
        assert!(!text.contains(UNPARSED), "absent ≠ unparseable:\n{text}");
    }
}
