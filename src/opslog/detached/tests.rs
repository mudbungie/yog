//! The detached-stderr sink: its computed name, and the read-time fold that
//! turns a silently-dead driver into a rendered failure (§8.1, §13.3, §7.3).

use super::super::{DETACHED_EXIT, OpEntry, OpRow, Origin, PIPED_UNOBSERVED};
use super::{TAIL, fold, sink};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// A detached `lernie prompt` line as [`crate::start::execute_prompt`] writes it
/// (bl-08f2 shape: `--name` rides ahead, the workspace is second-to-last, the
/// goal last), `stderr` empty (a launch, not an outcome).
fn prompt(ws: &Path) -> OpEntry {
    OpEntry {
        ts: "1784783961".into(),
        argv: vec![
            "lernie".into(),
            "prompt".into(),
            "--name".into(),
            "gecko".into(),
            ws.to_string_lossy().into_owned(),
            "goal".into(),
        ],
        cwd: "/proj".into(),
        exit: DETACHED_EXIT,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::default(),
    }
}

#[test]
fn sink_is_named_from_the_ts_and_the_workspace_leaf() {
    let root = Path::new("/state");
    assert_eq!(
        sink(root, "1784783961", Path::new("/ws/cobalt-gecko")),
        root.join("detached").join("1784783961-cobalt-gecko.err"),
    );
    // Two fires into different workspaces in the same second stay distinct.
    assert_ne!(
        sink(root, "17", Path::new("/ws/a")),
        sink(root, "17", Path::new("/ws/b")),
    );
    // A workspace path with no file name still yields a usable leaf.
    assert_eq!(
        sink(root, "17", Path::new("/")),
        root.join("detached").join("17-workspace.err"),
    );
}

#[test]
fn fold_surfaces_a_driver_that_died_after_launching() {
    let dir = tempdir().unwrap();
    let (state, ws) = (dir.path(), dir.path().join("cobalt-gecko"));
    let entry = prompt(&ws);
    // Before the child says anything: the row is a clean launch, exactly as the
    // append-time line recorded it — no sink file exists yet.
    let clean = fold(state, &entry);
    assert!(clean.stderr.is_empty());
    assert!(
        !OpRow::from(&clean).failed(),
        "a silent launch is no failure"
    );

    // The child refuses and dies; its stderr landed in the sink.
    let path = sink(state, &entry.ts, &ws);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "lernie: brazen 0.0.2 != 0.0.3\n").unwrap();
    let dead = fold(state, &entry);
    assert_eq!(dead.stderr, "lernie: brazen 0.0.2 != 0.0.3\n");
    assert!(
        OpRow::from(&dead).failed(),
        "captured stderr makes the -2 row a rendered failure (§7.3)"
    );
    // The append-only log's pre-bl-08f2 lines have no `--name` — the workspace
    // still reads off the tail (second-to-last), so old rows fold identically.
    let old_shape = OpEntry {
        argv: vec![
            "lernie".into(),
            "prompt".into(),
            ws.to_string_lossy().into_owned(),
            "goal".into(),
        ],
        ..entry.clone()
    };
    assert_eq!(
        fold(state, &old_shape).stderr,
        "lernie: brazen 0.0.2 != 0.0.3\n"
    );
}

/// **bl-b95e**: the fold is a transport and reads nothing. Whatever the sink
/// says — a benign lernie notice, a death — the tail rides into the row and the
/// row is a failure, because the caller folds only over a launch whose product
/// the derivation already found missing
/// ([`crate::opslog::launch::stillborn`]). bl-1296 put a phrase table here
/// instead; the words it read are inert now, which is what this pins.
#[test]
fn the_fold_carries_the_tail_and_reads_none_of_it() {
    let dir = tempdir().unwrap();
    let (state, ws) = (dir.path(), dir.path().join("cobalt-gecko"));
    let entry = prompt(&ws);
    let path = sink(state, &entry.ts, &ws);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let notice = "lernie: compaction landing [c-2] superseded — a compaction landed \
                  since its fork point (ARCH §2.6); the branch continues\n";
    fs::write(&path, notice).unwrap();

    let folded = fold(state, &entry);
    assert_eq!(folded.stderr, notice, "the tail folds in verbatim");
    let row = OpRow::from(&folded);
    assert!(
        row.failed(),
        "and a folded tail is the verdict, not the prose"
    );
    assert!(row.has_output(), "the pane still offers the expansion");

    // A death in the same append-only sink reads identically here: this layer
    // has no second answer to give.
    fs::write(&path, "lernie: brazen 0.0.2 != 0.0.3\n").unwrap();
    assert!(OpRow::from(&fold(state, &entry)).failed());
}

#[test]
fn fold_leaves_every_row_it_is_not_the_authority_for() {
    let dir = tempdir().unwrap();
    let (state, ws) = (dir.path(), dir.path().join("cobalt-gecko"));
    let entry = prompt(&ws);
    let path = sink(state, &entry.ts, &ws);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "from the sink").unwrap();

    // A piped verb's own captured stderr is the authority for its row.
    let piped = OpEntry {
        exit: PIPED_UNOBSERVED,
        ..entry.clone()
    };
    assert!(fold(state, &piped).stderr.is_empty());
    // A `-2` line that already carries text is a stored fact and the sink never
    // clobbers it. Nothing yog writes today takes this arm — a spawn failure is
    // a `-3` line since bl-afa9 — but `ops.jsonl` is append-only and pre-bl-afa9
    // lines on disk still say what they said.
    let legacy = OpEntry {
        stderr: "failed to spawn lernie".into(),
        ..entry.clone()
    };
    assert_eq!(fold(state, &legacy).stderr, "failed to spawn lernie");
    // A detached line too short to carry a workspace argument names no sink.
    let truncated_argv = OpEntry {
        argv: vec!["lernie".into()],
        ..entry
    };
    assert!(fold(state, &truncated_argv).stderr.is_empty());
}

#[test]
fn a_long_sink_folds_in_only_its_tail_from_a_line_boundary() {
    let dir = tempdir().unwrap();
    let (state, ws) = (dir.path(), dir.path().join("noisy"));
    let entry = prompt(&ws);
    let path = sink(state, &entry.ts, &ws);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    // A chattering driver: far more than TAIL bytes, then the cause, last.
    let noise = "chatter\n".repeat(2 * TAIL as usize / 8);
    fs::write(&path, format!("{noise}the cause\n")).unwrap();

    let folded = fold(state, &entry).stderr;
    assert!(folded.len() as u64 <= TAIL, "the fold is bounded");
    assert!(folded.ends_with("the cause\n"), "the tail is what is kept");
    assert!(
        folded.starts_with("chatter\n"),
        "the clipped head's partial line is dropped: {:?}",
        folded.get(..16),
    );
}
