//! **The honest absences** a projected row wears when nothing bound it — a
//! project that is no repo, two ends sharing no ancestor, a workspace claiming
//! nothing, a name the snapshot cannot resolve back to a repository. Split from
//! the bound columns at §12's budget on the seam the parent's own doc draws:
//! above is every column read off a real fan, here is what each column says
//! when there is nothing to read.

use super::super::{AGENT, BALL, CONV, NAME, named_agent, snap, trail};
use super::Lab;
use crate::science::Outcome;
use crate::workdiff::{Change, tests::Project};

/// A project that is no git repo states so — the diff column reads unreadable —
/// and the outcome is pending rather than a guess: with no target ref named,
/// there is no history to record a delivery and no sibling to lose to.
#[test]
fn an_unreadable_project_is_pending_with_nothing_named() {
    let lab = Lab::new();
    let plain = lab.ws.join("not-a-repo");
    std::fs::create_dir_all(&plain).unwrap();
    let snap = snap(&lab.ws, &plain, vec![], vec![]);
    let rows = lab.project_at(&snap, &[]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].diff.change, Change::Unreadable);
    assert_eq!(rows[0].diff.range(), None);
    assert_eq!(rows[0].base, None, "no pair, so no commit to name");
    assert_eq!(rows[0].outcome, Outcome::Pending);
}

/// Two ends that share no ancestor departed from **nothing this repo can name**:
/// the diff still reads — git compares unrelated trees happily — and the base
/// column says so rather than naming one of the two roots.
#[test]
fn ends_with_no_shared_ancestor_have_no_base() {
    let project = Project::new();
    let source = balls::delivery_path::work_branch(BALL);
    crate::git_tree::tests::git::run_git(&project.path, &["checkout", "-q", "--orphan", &source]);
    project.commit("src/b.rs", "fn b() {}\n");
    project.checkout(crate::workdiff::tests::MAIN);
    let lab = Lab::over(project);
    let snap = snap(&lab.ws, &lab.project.path, vec![], vec![]);
    let rows = lab.project_at(&snap, &[]);
    assert!(
        matches!(rows[0].diff.change, Change::Diff { .. }),
        "{rows:?}"
    );
    assert_eq!(rows[0].base, None);
}

/// A workspace that claims nothing projects nothing — the general path with no
/// inputs, not an arm of its own.
#[test]
fn no_obligation_projects_no_rows() {
    let lab = Lab::new();
    let mut snap = snap(&lab.ws, &lab.project.path, vec![], vec![]);
    snap.balls_by_project.clear();
    assert!(lab.project_at(&snap, &[]).is_empty());
}

/// A project whose name the snapshot cannot resolve has no repo to reproduce a
/// worktree path in, so the binding join declines rather than attributing
/// another attempt's conversation to the row.
#[test]
fn an_unresolvable_project_binds_nothing() {
    let lab = Lab::new();
    let claim = lab.claim(None);
    let entries = trail(&lab.ws, &lab.project.path, &[(CONV, &claim)]);
    let mut snap = snap(&lab.ws, &lab.project.path, vec![named_agent()], vec![]);
    // The enumeration is what makes a NAME resolve back to a repository; empty
    // it, and the row still reads (its diff was already taken) while nothing can
    // be located to bind against.
    snap.projects.clear();
    let rows = lab.project_at(&snap, &entries);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].conversation, None);
}

/// A claim worktree balls disambiguated with the claimant leaf binds too —
/// both spellings are asked, because which one exists is a disk question.
#[test]
fn the_claimant_disambiguated_leaf_binds() {
    let lab = Lab::new();
    let claim = lab.claim(Some(NAME));
    let entries = trail(&lab.ws, &lab.project.path, &[(CONV, &claim)]);
    let snap = snap(&lab.ws, &lab.project.path, vec![named_agent()], vec![]);
    super::super::worktree(&lab.ws, AGENT, "either leaf", &[]);
    let rows = lab.project_at(&snap, &entries);
    assert_eq!(rows[0].conversation.as_deref(), Some(AGENT));
}
