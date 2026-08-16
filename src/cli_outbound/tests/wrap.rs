//! The confinement wrapper seam (§8.6): a wrapped spawn execs the wrapper
//! first with the whole unwrapped spawn as its tail, a spawn *failure* names
//! the wrapper (the file the OS could not find), and the logical surfaces —
//! [`Cli::binary`], [`Cli::exec_words`] — stay wrapper-blind.

use super::*;

/// A recorder wrapper: writes the argv it received (one word per line) to its
/// first argument, then execs the rest — proving both the words and the
/// pass-through.
const RECORDER: &str =
    "#!/bin/sh\nlog=\"$1\"; shift\nprintf '%s\\n' \"$@\" > \"$log\"\nexec \"$@\"\n";

#[test]
fn a_wrapped_spawn_execs_the_wrapper_around_the_whole_unwrapped_spawn() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("argv.log");
    let (recorder, guard) = write_script(dir.path(), "recorder", RECORDER);
    let cli = Cli::new("/bin/echo").and_wrapper(vec![
        recorder.to_string_lossy().into_owned(),
        log.to_string_lossy().into_owned(),
    ]);
    // The logical surfaces are wrapper-blind: the trail logs the act, the W9
    // shim re-execs the tool — neither may name the envelope.
    assert_eq!(cli.binary(), Path::new("/bin/echo"));
    assert_eq!(cli.exec_words(), vec!["/bin/echo".to_owned()]);
    let stream = cli.run(&["held"]).expect("wrapper spawns");
    let (out, _, exit) = collect(stream);
    drop(guard);
    assert_eq!(exit, ExitInfo::Code(0));
    assert_eq!(String::from_utf8(out).unwrap(), "held\n");
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "/bin/echo\nheld\n",
        "the wrapper received the unwrapped spawn verbatim"
    );
}

#[test]
fn a_missing_wrapper_fails_the_spawn_naming_the_wrapper_not_the_tool() {
    let cli = Cli::new("/bin/echo").and_wrapper(vec!["/no/such/wrapper".to_owned()]);
    let guard = spawn_guard();
    let Err(err) = cli.run(&["held"]) else {
        panic!("no wrapper to exec")
    };
    drop(guard);
    assert!(err.to_string().contains("/no/such/wrapper"), "{err}");
}

#[test]
fn an_empty_wrapper_is_the_unwrapped_spawn() {
    let cli = Cli::new("/bin/echo").and_wrapper(Vec::new());
    assert_eq!(cli, Cli::new("/bin/echo"));
}
