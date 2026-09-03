//! bl-e058 — **a self-provisioned engine says the port it bound.**
//!
//! `address` holds `127.0.0.1:0` on a box that provisioned itself (REMOTE §8,
//! bl-dc14: an implicit mint must not take a process-global number), so the
//! port is the kernel's answer and lives only in the listener. The one
//! in-process consumer that used to be handed it was the window, and the
//! window left with bl-7942 — after which the bound port was knowable only by
//! asking the kernel about the process, which is not an interface. The ruling
//! is **say it**: one line on stderr, the success arm of the refusal the
//! failure arm already had.
//!
//! Driven against the real binary under a private `XDG_DATA_HOME`
//! (`stories_s8_t3`'s idiom), because the claim is about what a *boot* prints
//! and an in-process test can only observe the listener it built itself.

#![allow(clippy::unwrap_used)]

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Stdio;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// What the boot says, and the whole of what a seat needs off it.
const SAID: &str = "yog: wire: listening on ";

/// The ball's own repro, turned around: boot bare into a scratch world, read
/// the bound address off stderr, and check the request file is untouched —
/// there is no second address file and the `:0` default is unchanged.
#[test]
fn a_self_provisioned_boot_says_the_address_it_bound() {
    let anchor = tempdir().unwrap();
    let mut child = yog::git_env::command(Path::new(env!("CARGO_BIN_EXE_yog")))
        .env("XDG_DATA_HOME", anchor.path())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Read on a thread so the wait below has a deadline: a parked engine never
    // closes its stderr, so a blocking scan for a line that is not coming would
    // hang the suite instead of failing it.
    let err = child.stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(err).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                return;
            }
        }
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut said = None;
    while said.is_none() {
        let Some(left) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        match rx.recv_timeout(left) {
            Ok(line) => said = line.strip_prefix(SAID).map(str::to_owned),
            Err(_) => break,
        }
    }

    // Stop the engine before asserting: a failed assertion must not leave a
    // parked yog behind, and `Drop` on a `Child` does not kill it.
    let _ = yog::git_env::command(Path::new("kill"))
        .args(["-TERM", &child.id().to_string()])
        .status();
    let _ = child.wait();

    let said = said.expect("the boot named the address it bound");
    let port = said.rsplit_once(':').expect("host:port").1;
    assert!(said.starts_with("127.0.0.1:"), "loopback: {said}");
    assert!(
        port.parse::<u16>().is_ok_and(|p| p != 0),
        "the kernel's answer, never the `:0` request: {said}"
    );

    // The request stays the operator's, in its one home (bl-dc14): the boot
    // says what it became and writes nothing.
    let request = std::fs::read_to_string(anchor.path().join("yog/wire/address")).unwrap();
    assert_eq!(
        request.trim(),
        "127.0.0.1:0",
        "`address` is the request and the boot does not rewrite it"
    );
}
