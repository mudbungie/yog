//! The writable root: what it is derived from, and what it deliberately is not.

use super::*;
use crate::opslog::Origin;
use tempfile::tempdir;

/// A root over `writable`, with `cwd` and a fixed home.
fn root(writable: &[&str], cwd: &str) -> Root {
    Root {
        writable: writable.iter().map(PathBuf::from).collect(),
        cwd: PathBuf::from(cwd),
        home: PathBuf::from("/home/op"),
    }
}

/// A `bl claim` ops row as the start flow writes it.
fn claim_row(project: &str, id: &str, claimant: &str) -> OpEntry {
    OpEntry {
        ts: "TS".to_owned(),
        argv: ["bl", "claim", id, "--as", claimant]
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
        cwd: project.to_owned(),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::Balls,
    }
}

#[test]
fn an_operand_resolves_against_the_cwd_with_home_and_dots_folded() {
    let r = root(&["/w/agent"], "/w/agent/src");
    assert_eq!(r.resolve("mod.rs"), PathBuf::from("/w/agent/src/mod.rs"));
    assert_eq!(
        r.resolve("../Cargo.toml"),
        PathBuf::from("/w/agent/Cargo.toml")
    );
    assert_eq!(r.resolve("./x/./y"), PathBuf::from("/w/agent/src/x/y"));
    assert_eq!(r.resolve("/etc/hosts"), PathBuf::from("/etc/hosts"));
    assert_eq!(r.resolve("~"), PathBuf::from("/home/op"));
    assert_eq!(r.resolve("~/.ssh"), PathBuf::from("/home/op/.ssh"));
    // `~other` names another account — not ours to expand, and outside anyway.
    assert_eq!(r.resolve("~root/x"), PathBuf::from("/w/agent/src/~root/x"));
    // A `..` above the filesystem root drops, exactly as the kernel treats it.
    assert_eq!(r.resolve("/../../x"), PathBuf::from("/x"));
}

#[test]
fn containment_covers_the_root_itself_and_every_writable_dir() {
    let r = root(&["/w/agent", "/state/work/bl-1a2b"], "/w/agent");
    assert!(r.holds(Path::new("/w/agent")));
    assert!(r.holds(Path::new("/w/agent/src/mod.rs")));
    assert!(r.holds(Path::new("/state/work/bl-1a2b/README.md")));
    assert!(!r.holds(Path::new("/w/other/x")));
    assert!(r.holds_all(&[]), "no operands is inside the root");
    assert!(r.holds_all(&["src/a".to_owned(), "/w/agent/b".to_owned()]));
    assert!(!r.holds_all(&["src/a".to_owned(), "/etc/b".to_owned()]));
}

#[test]
fn the_agent_worktree_is_the_workspace_s_agents_dir() {
    assert_eq!(
        agent_worktree(Path::new("/w/ws"), "amber-1"),
        PathBuf::from("/w/ws/agents/amber-1")
    );
}

#[test]
fn the_bound_worktree_is_the_last_claim_this_workspace_made() {
    let balls = Path::new("/state/balls");
    let rows = [
        claim_row("/dev/proj", "bl-0000", "someone-else"),
        claim_row("/dev/proj", "bl-1111", "cobalt-gecko"),
        claim_row("/dev/other", "bl-2222", "cobalt-gecko"),
    ];
    // The last matching row wins — a re-claim supersedes — and both leaf
    // spellings are candidates, since which one balls minted is a disk fact
    // containment need not ask about.
    assert_eq!(
        bound_worktrees(&rows, balls, "cobalt-gecko"),
        [
            crate::binding::work_worktree_path(balls, Path::new("/dev/other"), "bl-2222", None),
            crate::binding::work_worktree_path(
                balls,
                Path::new("/dev/other"),
                "bl-2222",
                Some("cobalt-gecko")
            ),
        ]
    );
    // A workspace that never claimed through yog gets no bound worktree — the
    // stated limit of the §3.2 join, not an error.
    assert!(bound_worktrees(&rows, balls, "nobody").is_empty());
    assert!(bound_worktrees(&[], balls, "cobalt-gecko").is_empty());
}

#[test]
fn only_a_bl_claim_row_stamped_with_this_claimant_joins() {
    let balls = Path::new("/state/balls");
    let mut close = claim_row("/dev/proj", "bl-1111", "cobalt-gecko");
    close.argv[1] = "close".to_owned();
    let mut foreign = claim_row("/dev/proj", "bl-1111", "cobalt-gecko");
    foreign.argv[0] = "lernie".to_owned();
    let mut unstamped = claim_row("/dev/proj", "bl-1111", "cobalt-gecko");
    unstamped.argv.truncate(3);
    // And a row too short to be a claim at all — the trail carries every op.
    let mut short = claim_row("/dev/proj", "bl-1111", "cobalt-gecko");
    short.argv.truncate(2);
    for row in [close, foreign, unstamped, short] {
        assert!(
            bound_worktrees(std::slice::from_ref(&row), balls, "cobalt-gecko").is_empty(),
            "{:?} is not a claim this workspace made",
            row.argv
        );
    }
}

/// The §4.10 fan's candidates are in the writable root too, and by the same
/// two yog-owned facts: the claim row for the project, the fire rows for the
/// bindings. Without them a fanned drone could not write in the only directory
/// it has.
#[test]
fn a_fanned_candidates_own_worktree_is_writable_and_a_strangers_is_not() {
    let xdg = balls::layout::Xdg::with(Path::new("/home/u"), None, Some("/home/u/.local/state"));
    let ws = Path::new("/w/workspaces/cobalt-gecko");
    let mine = balls::delivery_path::attempt_path(&xdg, "/dev/proj", "at-0badcafe");
    let fire = |binding: &Path, cwd: &str| OpEntry {
        ts: "TS".to_owned(),
        argv: [
            "lernie",
            "prompt",
            "--name",
            "amber-1",
            "--cwd",
            &binding.to_string_lossy(),
            "/w/workspaces/cobalt-gecko",
            "goal",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect(),
        cwd: cwd.to_owned(),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::Balls,
    };
    let trail = vec![
        claim_row("/dev/proj", "bl-1111", "cobalt-gecko"),
        fire(&mine, "/w/workspaces/cobalt-gecko"),
        // Another workspace's fan is another seat's root, never ours.
        fire(
            &balls::delivery_path::attempt_path(&xdg, "/dev/proj", "at-99999999"),
            "/w/workspaces/other",
        ),
    ];
    assert_eq!(
        candidate_worktrees(&trail, &xdg, ws, "cobalt-gecko"),
        vec![mine],
    );
    // A workspace that never claimed anything has no obligation, so it has no
    // candidates either — the join starts at the claim, as the work one does.
    assert!(candidate_worktrees(&trail, &xdg, ws, "somebody-else").is_empty());
}

#[test]
fn the_cwd_mark_is_read_from_lernie_s_own_ref() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    let repo = ws.join("repo.git");
    std::fs::create_dir_all(&repo).unwrap();
    // No repo, no mark: the caller's default (the agent worktree) applies.
    assert_eq!(agent_cwd(&ws, "amber-1"), None);
    let git = |args: &[&str]| {
        crate::git_env::output(crate::git_env::git().arg("--git-dir").arg(&repo).args(args))
            .unwrap()
    };
    assert!(
        git(&["init", "--bare", "-b", "config/default"])
            .status
            .success()
    );
    // An initialized repo with no mark still reads as absent.
    assert_eq!(agent_cwd(&ws, "amber-1"), None);
    let blob = dir.path().join("cwd");
    std::fs::write(&blob, "/w/ws/agents/amber-1/sub\n").unwrap();
    let out = git(&["hash-object", "-w", "--", &blob.display().to_string()]);
    let oid = String::from_utf8(out.stdout).unwrap().trim().to_owned();
    assert!(
        git(&["update-ref", "refs/lernie/cwd/amber-1", &oid])
            .status
            .success()
    );
    assert_eq!(
        agent_cwd(&ws, "amber-1"),
        Some(PathBuf::from("/w/ws/agents/amber-1/sub")),
    );
    // An empty mark is no mark.
    std::fs::write(&blob, "").unwrap();
    let out = git(&["hash-object", "-w", "--", &blob.display().to_string()]);
    let oid = String::from_utf8(out.stdout).unwrap().trim().to_owned();
    assert!(
        git(&["update-ref", "refs/lernie/cwd/amber-1", &oid])
            .status
            .success()
    );
    assert_eq!(agent_cwd(&ws, "amber-1"), None);
}
