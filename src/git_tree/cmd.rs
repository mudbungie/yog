//! Git CLI wrapper and log/diff parsing for the git-tree view-model.
//!
//! The UI reads git state exclusively through the CLI (no libgit2), so
//! all subprocess invocations route through [`git`] here — and every one
//! of them is built by [`crate::git_env::command`], which scrubs the
//! inherited `GIT_DIR` and friends that would otherwise redirect a child
//! `git` back to the outer repo when the UI is launched from a git-hook
//! context.

use super::{GitTreeError, StepCommit};
use std::path::Path;
use std::process::Command;

/// A `git -C <repo> <args>` command over the scrubbed base, shared by every
/// runner below.
fn base_cmd(repo: &Path, args: &[&str]) -> Command {
    let mut cmd = crate::git_env::git();
    cmd.arg("-C").arg(repo).args(args);
    cmd
}

pub(super) fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>, GitTreeError> {
    let output = spawn_output(base_cmd(repo, args))?;
    if !output.status.success() {
        return Err(GitTreeError::Git {
            command: args.join(" "),
            repo: repo.to_path_buf(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(output.stdout)
}

/// Run `git`, returning its stdout on exit 0 or `None` on any clean non-zero
/// exit — the "no result" signal of the ancestry probes (`merge-base` with no
/// shared ancestor; `merge-base --is-ancestor` reporting "not an ancestor").
/// Only a spawn failure surfaces as `Err`. Mirrors lernie's own governing-
/// config fold (`src/workspace.rs`), which treats a failed `merge-base` as
/// "this config lineage contributes no candidate — skip it".
pub(super) fn git_optional(repo: &Path, args: &[&str]) -> Result<Option<Vec<u8>>, GitTreeError> {
    let output = spawn_output(base_cmd(repo, args))?;
    Ok(output.status.success().then_some(output.stdout))
}

/// Fork + exec `git`, capturing stdout/stderr. In production this is a plain
/// `Command::output`; under `cargo test` it routes through the binary-wide
/// `SPAWN_LOCK` (via `crate::test_support::spawn_locked`) so this `git` fork never
/// lands while a recorder-script test holds a write fd it is about to exec
/// (ETXTBSY — see `crate::test_support`). One fork discipline for every site.
#[cfg(not(test))]
fn spawn_output(mut cmd: Command) -> std::io::Result<std::process::Output> {
    cmd.output()
}

#[cfg(test)]
fn spawn_output(mut cmd: Command) -> std::io::Result<std::process::Output> {
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    crate::test_support::spawn_locked(&mut cmd)?.wait_with_output()
}

/// Raw `git log --format='%H %ct%x00%s'` row, parsed. The trunk is
/// the config lineage (§2.2), so `subject` labels a config commit.
#[derive(Debug)]
pub(super) struct LogEntry {
    pub(super) oid: String,
    pub(super) timestamp: i64,
    pub(super) subject: String,
}

pub(super) fn git_log_first_parent(repo: &Path) -> Result<Vec<LogEntry>, GitTreeError> {
    // `--first-parent` keeps conversation branches off the trunk log;
    // step commits are rendered nested under their merge node instead.
    // `\x00` separates the metadata from the subject so a subject
    // containing spaces parses unambiguously.
    let out = git(
        repo,
        &[
            "log",
            "--first-parent",
            "--format=%H %ct%x00%s",
            "--reverse",
            "HEAD",
        ],
    )?;
    parse_log(&out)
}

pub(super) fn parse_log(stdout: &[u8]) -> Result<Vec<LogEntry>, GitTreeError> {
    let text = String::from_utf8_lossy(stdout);
    let mut result = Vec::new();
    for line in text.lines() {
        let (head, subject) = line
            .split_once('\x00')
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let mut parts = head.splitn(2, ' ');
        let oid = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?
            .to_string();
        let ts_str = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let ts: i64 = ts_str
            .parse()
            .map_err(|_| GitTreeError::LogFormat(line.to_string()))?;
        result.push(LogEntry {
            oid,
            timestamp: ts,
            subject: subject.to_string(),
        });
    }
    Ok(result)
}

pub(super) fn walk_branch_steps(
    repo: &Path,
    branch: &str,
) -> Result<Vec<StepCommit>, GitTreeError> {
    // Commits on the agent branch past every config lineage (§2.2 —
    // there is no `main`; the fork point is a config commit). `\x00`
    // separates the timestamp from the subject so a subject containing
    // spaces parses unambiguously — the subject surfaces delivery and
    // work-product-transfer commits (§2.11, §2.6, §7.1).
    let out = git(
        repo,
        &[
            "log",
            "--reverse",
            "--first-parent",
            "--format=%H %ct%x00%s",
            branch,
            "--not",
            "--branches=config/*",
        ],
    )?;
    parse_step_commits(&out)
}

pub(super) fn parse_step_commits(stdout: &[u8]) -> Result<Vec<StepCommit>, GitTreeError> {
    let text = String::from_utf8_lossy(stdout);
    let mut result = Vec::new();
    for line in text.lines() {
        let (head, subject) = line
            .split_once('\x00')
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let (oid, ts) = head
            .split_once(' ')
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let ts: i64 = ts
            .parse()
            .map_err(|_| GitTreeError::LogFormat(line.to_string()))?;
        let short_oid = oid.get(..8).unwrap_or(oid).to_string();
        result.push(StepCommit {
            oid: oid.to_string(),
            short_oid,
            timestamp_unix: ts,
            subject: subject.to_string(),
        });
    }
    Ok(result)
}

/// Agent branches: every ref under `refs/heads/agents/` (ARCH §2.3 —
/// the prefix is the kind; agents never merge anywhere, §2.6).
pub(super) fn for_each_ref_agents(repo: &Path) -> Result<Vec<u8>, GitTreeError> {
    git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short) %(objectname) %(committerdate:unix)",
            "refs/heads/agents/",
        ],
    )
}

/// The lernie-stored **name fact** on an agent branch (lernie ARCH §2.3, DESIGN
/// §3.3): the `name` blob committed beside `goal.md` on the dispatch commit,
/// read `git show <branch>:name` — the ref namespace is the registry, so this
/// is a query against the bare repo, exactly lernie's own outside read. `None`
/// for a pre-0.0.4 branch with no blob (a clean non-zero `show`) and for the
/// empty blob lernie writes for an unnamed agent — absence and emptiness are
/// one fact: no name.
pub(super) fn ref_name(repo: &Path, branch: &str) -> Result<Option<String>, GitTreeError> {
    Ok(git_optional(repo, &["show", &format!("{branch}:name")])?
        .map(|out| String::from_utf8_lossy(&out).trim().to_string())
        .filter(|name| !name.is_empty()))
}

/// Every ref under a `refs/lernie/<kind>/` namespace as `<refname> <oid>`
/// lines (ARCH §8 — all four mark namespaces). The caller strips `prefix`
/// to recover the agent id and keeps the oid as §6 watermark evidence
/// ([`super::marks`]).
pub(super) fn for_each_ref_under(repo: &Path, prefix: &str) -> Result<Vec<u8>, GitTreeError> {
    git(
        repo,
        &["for-each-ref", "--format=%(refname) %(objectname)", prefix],
    )
}

/// One blob's bytes by oid — the **value** half of a valued mark (lernie ARCH
/// §3.3: `held` and `cwd` name blobs, the other four name commits).
/// `for-each-ref`'s `%(contents)` is empty for a blob, so the value is a second
/// call; it is only ever made for a ref that exists, which is why holds cost
/// nothing on the ordinary tick where none does.
pub(super) fn blob(repo: &Path, oid: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["cat-file", "blob", oid])
}

// --- Config-branch browse plumbing (DESIGN §9.3 / §5.1 #17–#18) ------------
// The read-only config surface reads config branches, their trees, and their
// file contents, and derives an agent's governing config by folding
// `merge-base` over the config refs. Every one of those calls lands here so
// the env scrub (`base_cmd`) is never bypassed.

/// Config branches: every `refs/heads/config/*` ref as
/// `<short-name> <oid> <committer-unix>` lines (§5.1 #18). `%(refname:short)`
/// yields `config/<name>`; the caller strips the `config/` prefix.
pub(crate) fn for_each_ref_config(repo: &Path) -> Result<Vec<u8>, GitTreeError> {
    git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short) %(objectname) %(committerdate:unix)",
            "refs/heads/config/",
        ],
    )
}

/// Every file path in a commit's tree (`git ls-tree -r --name-only`, §5.1 #18)
/// — the config commit's control files (`souls/**`, `workflow.yaml`, …).
pub(crate) fn ls_tree(repo: &Path, refspec: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["ls-tree", "-r", "--name-only", refspec])
}

/// Every **blob** in a commit's tree with its byte size
/// (`git ls-tree -r -l <refspec>`, VISION V1.2's pinned tree): lines of
/// `<mode> blob <oid> <size>\t<path>`. The long form exists because a listing
/// that shows a size must not show a guessed one — `--name-only` cannot say
/// how big a file was at that commit, and a zero would be a lie.
pub(crate) fn ls_tree_long(repo: &Path, refspec: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["ls-tree", "-r", "-l", refspec])
}

/// One file's raw bytes from a commit's tree (`git show <refspec>:<path>`,
/// §9.3). YAML is returned as text — yog adds no YAML dependency.
pub(crate) fn show_file(repo: &Path, refspec: &str, path: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["show", &format!("{refspec}:{path}")])
}

/// The paths added or modified between two commits under `prefix`
/// (`git diff --name-only --diff-filter=AM`, VISION §4.9): the transcript delta
/// a monitor check reads, derived from the sha its ops row names rather than
/// remembered. Renames and deletions are excluded because the transcript is
/// append-only — a file that vanished between two shas is not new work.
pub(crate) fn diff_names(
    repo: &Path,
    from: &str,
    to: &str,
    prefix: &str,
) -> Result<Vec<String>, GitTreeError> {
    let out = git(
        repo,
        &[
            "diff",
            "--name-only",
            "--diff-filter=AM",
            from,
            to,
            "--",
            prefix,
        ],
    )?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .map(str::to_owned)
        .filter(|line| !line.is_empty())
        .collect())
}

/// The best common ancestor of two commits (`git merge-base`), or `None` when
/// they share no history (an unrelated orphan lineage — exit 1, §9.3 fold).
pub(crate) fn merge_base(repo: &Path, a: &str, b: &str) -> Result<Option<String>, GitTreeError> {
    Ok(git_optional(repo, &["merge-base", a, b])?
        .map(|out| String::from_utf8_lossy(&out).trim().to_string()))
}

/// Is `a` an ancestor of `b`? (`git merge-base --is-ancestor`, §9.3 fold
/// tie-break — exit 0 = yes, any non-zero = no.)
pub(crate) fn is_ancestor(repo: &Path, a: &str, b: &str) -> Result<bool, GitTreeError> {
    Ok(git_optional(repo, &["merge-base", "--is-ancestor", a, b])?.is_some())
}
