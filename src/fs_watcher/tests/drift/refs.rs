//! Ref delivery: the git ref writes that must reach a live watcher —
//! `packed-refs` rewrites that touch nothing under `refs/`, and the
//! write-then-rename every ref update performs (§7.2).

use super::*;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn scenario_deleting_a_packed_ref_touches_nothing_under_refs() {
    // `git gc` runs `git pack-refs`, which empties `repo.git/refs/` into
    // `repo.git/packed-refs`. Deleting a ref that is only packed then rewrites
    // packed-refs ALONE — the loose tree never moves. yog reads refs through
    // `git for-each-ref` (loose + packed), so the agent disappears from the
    // derived tree while the watcher, allowlisting only `repo.git/refs`, sees
    // absolutely nothing. This asserts the ground truth the allowlist hole
    // rested on.
    let (_dir, root) = workspace_with_refs();
    let repo = root.join("repo.git");
    git(&repo, &["pack-refs", "--all"]);
    assert!(
        !repo.join("refs/heads/agents/aa-bb").exists(),
        "pack-refs emptied the loose tree"
    );
    git(&repo, &["update-ref", "-d", "refs/heads/agents/aa-bb"]);
    let refs = git(&repo, &["for-each-ref", "--format=%(refname)"]);
    assert_eq!(refs, "refs/heads/agents/cc-dd", "the ref is really gone");
    assert!(
        !repo.join("refs/heads/agents").exists(),
        "and nothing under repo.git/refs/ changed to say so"
    );
}

#[test]
fn a_packed_ref_deletion_now_reaches_the_watcher() {
    // The fix: `repo.git/packed-refs` is allowlisted, so the rewrite above is a
    // watched change and the workspace re-derives on the event instead of on
    // the sweep.
    let (_dir, root) = workspace_with_refs();
    let repo = root.join("repo.git");
    git(&repo, &["pack-refs", "--all"]);
    let watcher = Watcher::new(&root).unwrap();
    sleep(Duration::from_millis(200));
    let _ = watcher.tick();
    git(&repo, &["update-ref", "-d", "refs/heads/agents/aa-bb"]);
    let packed = repo.join("packed-refs");
    let changes = wait_for(&watcher, |c| c.path == packed);
    assert!(
        changes.iter().any(|c| c.path == packed),
        "packed-refs is watched: {changes:?}"
    );
}

#[test]
fn scenario_a_git_ref_update_coalesces_to_one_change_at_the_destination() {
    // git never writes a ref in place: it writes `<ref>.lock` and renames it
    // over the target — the atomic-rename sequence §7.2 claims to coalesce.
    // Every agent commit does this, so a leaked `.lock` path would be a
    // re-derivation storm and a missed destination would be a dropped event.
    let (_dir, root) = workspace_with_refs();
    let repo = root.join("repo.git");
    let watcher = Watcher::new(&root).unwrap();
    sleep(Duration::from_millis(200));
    let _ = watcher.tick();
    let tree = git(&repo, &["hash-object", "-w", "-t", "tree", "/dev/null"]);
    let head = git(&repo, &["rev-parse", "refs/heads/agents/aa-bb"]);
    let next = git(&repo, &["commit-tree", &tree, "-m", "y", "-p", &head]);
    git(&repo, &["update-ref", "refs/heads/agents/aa-bb", &next]);
    let target = repo.join("refs/heads/agents/aa-bb");
    let changes = wait_for(&watcher, |c| c.path == target);
    let hits: Vec<&Change> = changes.iter().filter(|c| c.path == target).collect();
    assert_eq!(hits.len(), 1, "one change at the destination: {changes:?}");
    assert_eq!(hits[0].kind, ChangeKind::Touched);
    assert!(
        !changes.iter().any(|c| c.path.ends_with("aa-bb.lock")),
        "the .lock rename source never surfaces: {changes:?}"
    );
}
