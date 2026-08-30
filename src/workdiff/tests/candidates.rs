//! The fan's candidate rows on the work-diff surface (VISION V3.2–V3.3,
//! bl-c2bd): membership off the workspace's own trail, the ruled range
//! `work/<id>..attempt/<handle>` per candidate, the derived acceptance mark,
//! and the patch drill-in addressed one level deeper than a ball.

use std::path::Path;

use crate::files_view::Preview;
use crate::opslog::OpEntry;
use crate::workdiff::{Change, WorkFile, patch, read};

use super::{ball, snap, xdg};

const WS: &str = "/data/workspaces/storeroom";
const NAME: &str = "storeroom";
const BALL: &str = "bl-1";

/// The claim row the obligation is read from (`control::root::claimed`) and
/// one fire row per candidate binding (`fan::cohort::members`) — the same two
/// yog-owned facts the §8.6 writable root reads.
fn trail(project: &Path, bindings: &[&Path]) -> Vec<OpEntry> {
    let mut entries = vec![OpEntry {
        argv: ["bl", "claim", BALL, "--as", NAME]
            .map(str::to_owned)
            .to_vec(),
        cwd: project.to_string_lossy().into_owned(),
        ..OpEntry::default()
    }];
    for (i, binding) in bindings.iter().enumerate() {
        entries.push(OpEntry {
            argv: [
                "litany",
                "prompt",
                "--name",
                &format!("conv-{i}"),
                "--cwd",
                &binding.to_string_lossy(),
            ]
            .map(str::to_owned)
            .to_vec(),
            cwd: WS.to_owned(),
            ..OpEntry::default()
        });
    }
    entries
}

/// One commit of one file in a candidate's own worktree.
fn work(worktree: &Path, file: &str) {
    use crate::git_tree::tests::git::run_git;
    std::fs::write(worktree.join(file), "fn f() {}\n").unwrap();
    run_git(worktree, &["add", file]);
    run_git(worktree, &["config", "user.email", "t@t.local"]);
    run_git(worktree, &["config", "user.name", "Tester"]);
    run_git(worktree, &["config", "commit.gpgsign", "false"]);
    run_git(worktree, &["commit", "-q", "-m", "candidate work"]);
}

#[test]
fn candidates_read_at_the_ruled_range_and_wear_the_derived_mark() {
    let project = super::Project::new();
    let dir = tempfile::tempdir().unwrap();
    let xdg = xdg(dir.path());
    let obligation = crate::fan::Obligation {
        project: "proj".to_owned(),
        ball: Some(BALL.to_owned()),
    };
    let candidates = crate::fan::open(&obligation, &project.path, &xdg, 2).unwrap();
    work(&candidates[0].worktree, "won.rs");
    let entries = trail(
        &project.path,
        &[&candidates[0].worktree, &candidates[1].worktree],
    );
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![ball(BALL, Some(NAME), None)],
    );

    let attempts = read(&snap, Path::new(WS), &entries, &xdg);
    // The claim row first (its WorkFile address has no handle), then one row
    // per candidate, oldest fire first.
    assert_eq!(attempts.len(), 3, "{attempts:?}");
    assert_eq!(attempts[0].handle, None);
    let (worked, empty) = (&attempts[1], &attempts[2]);
    assert_eq!(
        worked.handle.as_deref(),
        Some(candidates[0].handle.as_str())
    );
    assert_eq!(worked.ball_id, BALL);
    assert_eq!(worked.project, snap.project_name(&project.path));
    assert_eq!(
        worked.range(),
        Some(format!("work/{BALL}..attempt/{}", candidates[0].handle)),
    );
    let Change::Diff { files, .. } = &worked.change else {
        panic!("a worked candidate diffs: {worked:?}");
    };
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "won.rs");
    // Nothing is delivered yet: no mark anywhere, and rejection would look
    // exactly like this — the absence of a delivery.
    assert_eq!(worked.delivered, None);
    assert_eq!(empty.delivered, None);

    // The patch drill-in is addressed by ball AND handle — a fan's candidates
    // all wear the obligation's ball.
    let picked = WorkFile {
        ball: BALL.to_owned(),
        handle: Some(candidates[0].handle.clone()),
        path: "won.rs".to_owned(),
    };
    let Some(Preview::Text(text)) = patch(&snap, &attempts, &picked) else {
        panic!("a changed candidate file has a patch");
    };
    assert!(text.contains("+fn f() {}"), "{text}");

    // Accept the worked candidate: the mark appears on it — read back off the
    // target's history, stored nowhere — and stays absent on its sibling.
    let delivery = crate::fan::deliver(
        &obligation,
        &project.path,
        &xdg,
        &candidates[0].handle,
        "take it",
    )
    .unwrap();
    let attempts = read(&snap, Path::new(WS), &entries, &xdg);
    assert_eq!(attempts[1].delivered, delivery.commit);
    assert_eq!(attempts[2].delivered, None);
}

/// A trail whose fires bound only the ordinary claim worktree contributes no
/// candidate rows: balls' attempt formula does not reproduce `work/<id>`
/// paths, so the fold states "no fan here" rather than guessing one.
#[test]
fn an_ordinary_claim_fire_is_no_candidate() {
    let project = super::Project::new();
    let dir = tempfile::tempdir().unwrap();
    let xdg = xdg(dir.path());
    let claim_worktree = dir.path().join("claim");
    let entries = trail(&project.path, &[&claim_worktree]);
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![ball(BALL, Some(NAME), None)],
    );
    let attempts = read(&snap, Path::new(WS), &entries, &xdg);
    assert_eq!(attempts.len(), 1, "the claim row alone: {attempts:?}");
    assert_eq!(attempts[0].handle, None);
}
