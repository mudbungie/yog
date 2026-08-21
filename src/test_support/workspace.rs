//! A real lernie workspace on disk (lernie ARCH §2.2), reduced to what yog's
//! own policy authoring reads: a bare `repo.git` whose orphan `config/default`
//! root carries the control files — `workflow.yaml` for §8.6's capability
//! control, `manifest.yaml` for §3.7's instruction glob, `instructions.yaml`
//! for §3.7's filename override. Shared by the authoring tests and the
//! start-flow tests that prove a workspace which cannot be converged aborts the
//! start.

use std::path::Path;

/// The single-file case: a `config/default` carrying only `workflow.yaml` —
/// what §8.6's tests read, and a workspace whose manifest yog leaves alone.
pub(crate) fn seed_workspace_workflow(workspace: &Path, workflow: &str) {
    seed_workspace_config(workspace, &[("workflow.yaml", workflow)]);
}

/// Author a workspace's `config/default` carrying `files` — the shape `lernie
/// new` leaves behind, reduced to the control files yog reads and rewrites.
pub(crate) fn seed_workspace_config(workspace: &Path, files: &[(&str, &str)]) {
    let repo = workspace.join("repo.git");
    let author = workspace.join(".author");
    std::fs::create_dir_all(&repo).unwrap();
    let run = |args: &[&str]| {
        let status = crate::git_env::status(crate::git_env::git().args(args)).unwrap();
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
    for (name, body) in files {
        std::fs::write(author.join(name), body).unwrap();
    }
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
