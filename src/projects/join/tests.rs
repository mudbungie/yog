use super::*;
use crate::projects::balls::{Ball, Blocker};

fn ball(id: &str) -> Ball {
    Ball {
        id: id.to_owned(),
        title: format!("t-{id}"),
        body: String::new(),
        claimant: None,
        blockers: Vec::new(),
        parent: None,
        priority: 0,
        tags: Vec::new(),
        created: None,
        updated: None,
        root_commit: None,
    }
}

/// A ball claimed by `by`.
fn claimed(id: &str, by: &str) -> Ball {
    Ball {
        claimant: Some(by.to_owned()),
        ..ball(id)
    }
}

/// A named workspace at `path` whose leaf-name is `name`.
fn named(name: &str, path: &str) -> Workspace {
    Workspace {
        path: PathBuf::from(path),
        kind: WorkspaceKind::Named {
            name: name.to_owned(),
        },
    }
}

#[test]
fn classify_is_the_total_ball_driven_table() {
    use JoinState as S;
    use JoinStatus::{Blocked, Claimed, Closed, Ready};
    // (status, bound) => the §3.5 ball-driven row. All 4×2 combos.
    let table = [
        (Ready, false, S::ReadyStartable),
        (Ready, true, S::ReadyStartable),
        (Blocked, false, S::Blocked),
        (Blocked, true, S::Blocked),
        (Claimed, false, S::ClaimedElsewhere),
        (Claimed, true, S::Bound),
        (Closed, false, S::Delivered),
        (Closed, true, S::Delivered),
    ];
    for (status, bound, want) in table {
        assert_eq!(classify(status, bound), want, "({status:?},{bound})");
    }
}

#[test]
fn badge_labels_each_join_state() {
    assert_eq!(badge(JoinState::ReadyStartable, None), None);
    assert_eq!(badge(JoinState::Bound, None), None);
    assert_eq!(badge(JoinState::UnassignedWorkspace, None), None);
    assert_eq!(badge(JoinState::Blocked, None).as_deref(), Some("blocked"));
    assert_eq!(
        badge(JoinState::ClaimedElsewhere, Some("boss")).as_deref(),
        Some("claimed by boss")
    );
    assert_eq!(
        badge(JoinState::ClaimedElsewhere, None).as_deref(),
        Some("claimed by ?")
    );
    assert_eq!(
        badge(JoinState::Delivered, None).as_deref(),
        Some("delivered")
    );
    assert_eq!(
        badge(JoinState::OrphanedProject, None).as_deref(),
        Some("project missing")
    );
}

/// Find the single row for `(project, ball)` in a join result (cloned, so the
/// caller reads its fields without holding the `rows` borrow).
fn find(rows: &[JoinRow], project: &str, ball: &str) -> JoinRow {
    rows.iter()
        .find(|r| r.project == crate::naming::leaf(Path::new(project)) && r.ball_id == ball)
        .cloned()
        .unwrap_or_else(|| panic!("no row for {project}/{ball}"))
}

#[test]
fn named_map_keeps_only_named_workspaces() {
    let ws = vec![
        named("cobalt", "/w/cobalt"),
        Workspace {
            path: PathBuf::from("/w/foreign"),
            kind: WorkspaceKind::Foreign,
        },
        Workspace {
            path: PathBuf::from("/w/replay"),
            kind: WorkspaceKind::Replay,
        },
    ];
    let m = named_map(&ws);
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("cobalt"), Some(&Path::new("/w/cobalt")));
}

#[test]
fn join_enumerates_every_row_state() {
    let p1 = "/p1";
    // Live balls: bound (claimant = a local name), ready, claimed-elsewhere
    // (claimant = a non-local name), blocked.
    let b_bound = claimed("bl-bound", "cobalt");
    let b_ready = ball("bl-ready");
    let b_boss = claimed("bl-boss", "boss"); // no workspace named "boss"
    let mut b_blocked = ball("bl-blocked");
    b_blocked.blockers = vec![Blocker {
        id: "bl-bound".into(),
        on: "claim".into(),
    }];
    let mut live = HashMap::new();
    live.insert(PathBuf::from(p1), vec![b_bound, b_ready, b_boss, b_blocked]);

    // On-demand closed listing: a delivered ball under "cobalt", and a closed
    // ball claimed by a non-local name (dropped — raw-listing only).
    let mut closed = HashMap::new();
    closed.insert(
        PathBuf::from(p1),
        vec![claimed("bl-done", "cobalt"), claimed("bl-gone", "boss")],
    );

    // Workspaces: "cobalt" (engaged by bl-bound), "spare" (no ball → unassigned).
    let workspaces = vec![named("cobalt", "/w/cobalt"), named("spare", "/w/spare")];
    // p1 is cloned + listable; "/gone" is cloned but absent from `live` → orphan.
    let cloned = vec![PathBuf::from(p1), PathBuf::from("/gone")];
    let rows = join(&cloned, &cloned, &live, &closed, &workspaces);

    // Bound: grouped under its workspace, live detail carried.
    let bound = find(&rows, p1, "bl-bound");
    assert_eq!(bound.state, JoinState::Bound);
    assert_eq!(bound.workspace.as_deref(), Some("cobalt"));
    assert_eq!(bound.claimant.as_deref(), Some("cobalt"));
    assert_eq!(bound.title.as_deref(), Some("t-bl-bound"));
    // Ready / blocked: unclaimed, no workspace.
    assert_eq!(find(&rows, p1, "bl-ready").state, JoinState::ReadyStartable);
    assert_eq!(find(&rows, p1, "bl-ready").workspace, None);
    assert_eq!(find(&rows, p1, "bl-blocked").state, JoinState::Blocked);
    // Claimed elsewhere: claimant verbatim, no local workspace.
    let boss = find(&rows, p1, "bl-boss");
    assert_eq!(boss.state, JoinState::ClaimedElsewhere);
    assert_eq!(boss.claimant.as_deref(), Some("boss"));
    assert_eq!(boss.workspace, None);
    // Delivered: the closed ball under "cobalt", no live detail.
    let done = find(&rows, p1, "bl-done");
    assert_eq!(done.state, JoinState::Delivered);
    assert_eq!(done.workspace.as_deref(), Some("cobalt"));
    assert_eq!(done.title, None);
    // The closed ball claimed elsewhere contributes no row.
    assert!(!rows.iter().any(|r| r.ball_id == "bl-gone"));
    // Unassigned workspace: "spare", no ball, no project.
    let spare = rows
        .iter()
        .find(|r| r.workspace.as_deref() == Some("spare"))
        .unwrap();
    assert_eq!(spare.state, JoinState::UnassignedWorkspace);
    assert_eq!(spare.ball_id, "");
    assert_eq!(spare.project, "");
    // Orphaned project: cloned but unlistable.
    let orphan = find(&rows, "/gone", "");
    assert_eq!(orphan.state, JoinState::OrphanedProject);
    assert_eq!(orphan.workspace, None);
}

#[test]
fn a_reopened_ball_keeps_only_its_live_row_not_a_stale_delivered() {
    let p1 = "/p1";
    // The stale closed cache still lists bl-back under "cobalt", but it is live
    // again — the live Bound row is authoritative, the closed entry is skipped.
    let mut live = HashMap::new();
    live.insert(PathBuf::from(p1), vec![claimed("bl-back", "cobalt")]);
    let mut closed = HashMap::new();
    closed.insert(PathBuf::from(p1), vec![claimed("bl-back", "cobalt")]);
    let workspaces = vec![named("cobalt", "/w/cobalt")];
    let rows = join(
        &[PathBuf::from(p1)],
        &[PathBuf::from(p1)],
        &live,
        &closed,
        &workspaces,
    );

    let back: Vec<JoinState> = rows
        .iter()
        .filter(|r| r.ball_id == "bl-back")
        .map(|r| r.state)
        .collect();
    assert_eq!(
        back,
        vec![JoinState::Bound],
        "one live row, no delivered dup"
    );
}

#[test]
fn owner_name_is_the_bound_ball_claimant() {
    // The name close/release/move-from stamps `--as` (§8.2 rider): the claimant,
    // which for a Bound row is the local workspace's own name.
    let row = JoinRow {
        project: "p".into(),
        ball_id: "bl-1".to_owned(),
        state: JoinState::Bound,
        workspace: Some("yog/workspaces/cobalt".into()),
        claimant: Some("cobalt".to_owned()),
        title: None,
    };
    assert_eq!(owner_name(&row), "cobalt");
    // A claimant-less row (never a Bound one) → empty, never a panic.
    let bare = JoinRow {
        claimant: None,
        ..row
    };
    assert_eq!(owner_name(&bare), "");
}
