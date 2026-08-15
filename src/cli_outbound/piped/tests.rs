//! The fourth spawn shape: a child handed a document on stdin, and the EOF
//! that says the document is whole.

use super::*;
use crate::cli_outbound::Chunk;
use crate::test_support::spawn_guard;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

/// Everything a stream said, in order, and the exit it ended on.
fn drained(stream: Stream) -> (String, i32) {
    let mut out = Vec::new();
    let mut exit = -1;
    for chunk in stream {
        match chunk {
            Chunk::Stdout(bytes) | Chunk::Stderr(bytes) => out.extend(bytes),
            Chunk::Exited(info) => exit = info.shell_code(),
        }
    }
    (String::from_utf8_lossy(&out).into_owned(), exit)
}

/// The input arrives whole and the pipe is **closed**, so a child that reads to
/// EOF finishes — the one thing that distinguishes this shape from the three
/// that close stdin outright.
#[test]
fn the_input_arrives_and_the_pipe_closes() {
    let _guard = spawn_guard();
    let dir = tempdir().expect("tmp");
    let tool = dir.path().join("cat-tool");
    fs::write(&tool, "#!/bin/sh\ncat\nexit 3\n").expect("script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("chmod");

    let stream = Cli::new(&tool)
        .run_input(Some(dir.path()), b"{\"command\":\"ls\"}", &[])
        .expect("spawned");
    assert_eq!(
        drained(stream),
        ("{\"command\":\"ls\"}".to_owned(), 3),
        "read to EOF, and the exit is the child's"
    );
}

/// A child that never reads its stdin is not held up by it — the write is
/// best-effort, and its own capture is what says what happened.
#[test]
fn a_child_that_ignores_its_input_still_answers() {
    let _guard = spawn_guard();
    let dir = tempdir().expect("tmp");
    let tool = dir.path().join("deaf-tool");
    fs::write(&tool, "#!/bin/sh\necho 'never read it'\n").expect("script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("chmod");

    let stream = Cli::new(&tool)
        .run_input(None, &vec![b'x'; 4096], &[])
        .expect("spawned");
    assert_eq!(drained(stream), ("never read it\n".to_owned(), 0));
}

/// A fork that never happened is the one error — the same refusal every other
/// spawn shape gives, through the same mapping.
#[test]
fn an_unspawnable_command_is_the_one_error() {
    let _guard = spawn_guard();
    let dir = tempdir().expect("tmp");
    assert!(
        Cli::new(dir.path().join("no-such-binary"))
            .run_input(None, b"", &[])
            .is_err()
    );
}
