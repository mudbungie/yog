//! A real lernie workspace on disk (lernie ARCH §2.2), reduced to what §8.6's
//! control authoring reads: a bare `repo.git` whose orphan `config/default`
//! root carries `workflow.yaml`. Shared by the authoring tests and the
//! start-flow test that proves a workspace which cannot be controlled aborts
//! the start.

use std::path::Path;

/// Author a workspace's `config/default` carrying `workflow.yaml` — the shape
/// `lernie new` leaves behind, reduced to the one file §8.6's control authoring
/// reads and rewrites. Shared by the authoring tests and the start-flow test
/// that proves a workspace which cannot be controlled aborts the start.
pub(crate) fn seed_workspace_workflow(workspace: &Path, workflow: &str) {
    let repo = workspace.join("repo.git");
    let author = workspace.join(".author");
    std::fs::create_dir_all(&repo).unwrap();
    let run = |args: &[&str]| {
        let status = crate::git_env::git().args(args).status().unwrap();
        assert!(status.success(), "git {args:?}");
    };
    let (repo_s, author_s) = (repo.display().to_string(), author.display().to_string());
    run(&["init", "-q", "--bare", "-b", "config/default", &repo_s]);
    run(&[
        "-C",
        &repo_s,
        "worktree",
        "add",
        "-q",
        "--orphan",
        "-b",
        "config/default",
        &author_s,
    ]);
    std::fs::write(author.join("workflow.yaml"), workflow).unwrap();
    run(&["-C", &author_s, "add", "-A"]);
    run(&[
        "-C",
        &author_s,
        "-c",
        "user.email=t@t.local",
        "-c",
        "user.name=T",
        "-c",
        "commit.gpgsign=false",
        "commit",
        "-q",
        "-m",
        "config: init [config/default]",
    ]);
    run(&["-C", &repo_s, "worktree", "remove", &author_s]);
}
