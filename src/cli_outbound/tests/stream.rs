//! Streaming and chunk plumbing: iterator termination after `Exited`,
//! `exit_info` classification (signal / unknown / stopped), and the
//! `pump_step` reader→channel forwarding primitive.

use super::super::chunk::pump_step;
use super::*;
use crate::cli_outbound::stream::exit_info;
use std::io::Read;
use tempfile::tempdir;

#[test]
fn exit_info_reports_signal_when_child_killed_mid_flight() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nprintf 'hi\\n'\nkill -USR1 $$\nsleep 5\n",
    );
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let (_out, _err, exit) = collect(stream);
    assert!(
        matches!(exit, ExitInfo::Signal(_)),
        "expected Signal, got {exit:?}"
    );
}

#[test]
fn iterator_returns_none_after_exited() {
    let dir = tempdir().unwrap();
    let bin = write_script(dir.path(), "fake_lernie", "#!/bin/sh\nexit 0\n");
    let cli = Cli::new(bin);
    let mut stream = cli.run(&[]).unwrap();
    for chunk in stream.by_ref() {
        if matches!(chunk, Chunk::Exited(_)) {
            break;
        }
    }
    assert!(stream.next().is_none());
    assert!(stream.next().is_none());
}

#[test]
fn exit_info_unknown_when_status_missing() {
    assert_eq!(exit_info(None), ExitInfo::Unknown);
}

#[test]
fn shell_code_follows_the_shell_convention() {
    // A plain code passes through, a signal is 128 + signum, an unobservable
    // status is -1 — the single mapping `yog exec` and `ops.jsonl` share.
    assert_eq!(ExitInfo::Code(3).shell_code(), 3);
    assert_eq!(ExitInfo::Signal(9).shell_code(), 137);
    assert_eq!(ExitInfo::Unknown.shell_code(), -1);
}

#[test]
fn exit_info_unknown_for_stopped_status() {
    use std::os::unix::process::ExitStatusExt;
    // Raw wait status 0x7f = WIFSTOPPED. On Linux this produces
    // code() == None && signal() == None (stopped_signal is separate),
    // which is our Unknown branch.
    let stopped = std::process::ExitStatus::from_raw(0x7f);
    assert_eq!(exit_info(Some(stopped)), ExitInfo::Unknown);
}

#[test]
fn pump_step_returns_false_on_eof() {
    let (tx, _rx) = mpsc::channel::<Chunk>();
    let mut reader: &[u8] = &[];
    let mut buf = [0u8; 16];
    assert!(!pump_step(&mut reader, &tx, &mut buf, Chunk::Stdout));
}

#[test]
fn pump_step_returns_false_when_receiver_dropped() {
    let (tx, rx) = mpsc::channel::<Chunk>();
    drop(rx);
    let mut reader: &[u8] = b"data";
    let mut buf = [0u8; 16];
    assert!(!pump_step(&mut reader, &tx, &mut buf, Chunk::Stdout));
}

#[test]
fn pump_step_returns_true_and_forwards_chunk() {
    let (tx, rx) = mpsc::channel::<Chunk>();
    let mut reader: &[u8] = b"abc";
    let mut buf = [0u8; 16];
    assert!(pump_step(&mut reader, &tx, &mut buf, Chunk::Stderr));
    assert_eq!(rx.try_recv().unwrap(), Chunk::Stderr(b"abc".to_vec()));
}

#[test]
fn pump_step_returns_false_on_read_error() {
    struct Failing;
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("boom"))
        }
    }
    let (tx, _rx) = mpsc::channel::<Chunk>();
    let mut reader = Failing;
    let mut buf = [0u8; 16];
    assert!(!pump_step(&mut reader, &tx, &mut buf, Chunk::Stdout));
}
