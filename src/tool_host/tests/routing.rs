//! A loaded remote name, routed: the capture that comes back verbatim, and the
//! transport failure that is a sentence (REMOTE §9 step 7).

use super::*;

/// **The leg, end to end through the injection** (REMOTE §9 step 7, bl-024b):
/// a loaded name is owned, queued at the engine, polled for, and the far
/// machine's own capture comes back **verbatim** — a non-zero verdict and text
/// on stderr included, because a routed tool must be indistinguishable from a
/// local one.
#[test]
fn a_loaded_remote_name_is_routed_and_its_capture_comes_back() {
    let root = TempDir::new().expect("tmp");
    loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[loaded::Entry {
            client: "laptop".to_owned(),
            tool: tool("Bash"),
        }],
    )
    .expect("loaded");

    let (handle, seen) = scripted(
        root.path(),
        &[
            json!({"ok": true, "kind": "routed", "invocation": "inv-1"}),
            json!({"ok": true, "kind": "routed", "invocation": "inv-1",
                   "capture": {"stdout": "a\nb\n", "stderr": "warned\n", "exit_code": 3}}),
        ],
    );
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture =
        at(root.path(), PathBuf::new(), budget()).route(call!("laptop_Bash", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 3, "the far machine's verdict, verbatim");
    assert_eq!(capture.stdout, b"a\nb\n");
    assert_eq!(capture.stderr, b"warned\n");
    assert_eq!(
        seen.recv().expect("the queueing act"),
        json!({"op": "invoke", "client": "laptop", "tool": "Bash",
               "input": {"command": "ls"}}),
        "the advertised name crosses, never the prefixed one"
    );
    assert_eq!(
        seen.recv().expect("the poll"),
        json!({"op": "capture", "invocation": "inv-1"})
    );
}

/// A transport failure is yog's own sentence, and it is in band and non-zero —
/// the shape a vanished endpoint already had to produce. Nothing hangs.
#[test]
fn an_engine_that_never_answers_refuses_in_band() {
    let root = TempDir::new().expect("tmp");
    loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[loaded::Entry {
            client: "laptop".to_owned(),
            tool: tool("Bash"),
        }],
    )
    .expect("loaded");

    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("laptop_Bash", &input, &stop));
    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("laptop_Bash: "), "{said}");
    assert!(said.contains("no engine answered"), "{said}");
}
