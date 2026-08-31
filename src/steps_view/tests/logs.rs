//! bl-83d6: the drill-in's **capture-log reads** — the step's own `stderr.log`
//! and the agent's `driver.log`, answered in full where the §7.3 banners quote
//! three lines of them.
//!
//! One claim, driven from the on-disk shape: a log with nothing in it is not a
//! reading, and a log that *is* there crosses the boundary whole, at the
//! bounded-file cap every other reading surface uses rather than a tail. What
//! a seat then does with the two (its picker's derived row set, and where the
//! seat sits) went with the window (bl-7942); the READS are the engine's.

use super::{AGENT, write_file};
use crate::files_view::Preview;
use crate::steps_view::{StepDetail, detail};

/// The observed shape a wound leaves: a request, an opened-and-empty response,
/// and the adapter's words on stderr.
const ADAPTER: &str = "bz: no workspace in this environment — providers, sign-ins and the model \
cache belong to a workspace.";

/// What a driver that declined at the boundary writes to `driver.log`.
const DRIVER: &str = "litany: provider refused the turn — unpaired tool_use in the tail";

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
    // And both cross the §8.5 boundary as the reading they are, so a seat
    // never has to go to disk for them.
    let said = crate::steps_view::wire::detail(&d);
    assert!(
        said["stderr"]["text"].as_str().unwrap_or_default() == ADAPTER,
        "got:\n{said:#}"
    );
    assert!(
        said["driver"]["text"].as_str().unwrap_or_default() == DRIVER,
        "got:\n{said:#}"
    );
}

#[test]
fn an_empty_or_absent_log_is_no_reading_at_all() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    // The ordinary run: litany opens `stderr.log` every attempt and the adapter
    // says nothing in it (litany ARCH §2.3), and this agent's drivers have
    // written no log at all. The two are one fact — nothing to read — and a
    // seat offered a row for either would be offering a dead one.
    write_file(ws, "001", "meta.json", br#"{"commit":"c0ffee"}"#);
    write_file(ws, "001", "stderr.log", b"");
    let d = detail(ws, AGENT, "001");
    assert_eq!(d.stderr, None, "an empty log is nothing to read");
    assert_eq!(d.driver, None, "an absent log is the same fact");
    let said = crate::steps_view::wire::detail(&d);
    assert!(said["stderr"].is_null(), "got:\n{said:#}");
    assert!(said["driver"].is_null(), "got:\n{said:#}");
}
