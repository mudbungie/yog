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
/// Only a spawn failure surfaces as `Err`. Mirrors litany's own governing-
/// config fold (`src/workspace.rs`), which treats a failed `merge-base` as
/// "this config lineage contributes no candidate — skip it".
pub(super) fn git_optional(repo: &Path, args: &[&str]) -> Result<Option<Vec<u8>>, GitTreeError> {
    let output = spawn_output(base_cmd(repo, args))?;
    Ok(output.status.success().then_some(output.stdout))
}

/// Fork + exec `git`, capturing stdout/stderr — the crate's one fork
/// (`crate::git_env::output`), which under `cargo test` takes the binary-wide
/// spawn lock so this `git` never forks while a recorder-script test holds a
/// write fd it is about to exec (ETXTBSY — see `crate::git_env`).
fn spawn_output(mut cmd: Command) -> std::io::Result<std::process::Output> {
    crate::git_env::output(&mut cmd)
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

/// The litany-stored **name fact** on an agent branch (litany ARCH §2.3, DESIGN
/// §3.3): the `name` blob committed beside `goal.md` on the dispatch commit,
/// read `git show <branch>:name` — the ref namespace is the registry, so this
/// is a query against the bare repo, exactly litany's own outside read. `None`
/// for a pre-0.0.4 branch with no blob (a clean non-zero `show`) and for the
/// empty blob litany writes for an unnamed agent — absence and emptiness are
/// one fact: no name.
pub(super) fn ref_name(repo: &Path, branch: &str) -> Result<Option<String>, GitTreeError> {
    Ok(git_optional(repo, &["show", &format!("{branch}:name")])?
        .map(|out| String::from_utf8_lossy(&out).trim().to_string())
        .filter(|name| !name.is_empty()))
}

/// Every ref under a `refs/litany/<kind>/` namespace as `<refname> <oid>`
/// lines (ARCH §8 — all four mark namespaces). The caller strips `prefix`
/// to recover the agent id and keeps the oid as §6 watermark evidence
/// ([`super::marks`]).
pub(super) fn for_each_ref_under(repo: &Path, prefix: &str) -> Result<Vec<u8>, GitTreeError> {
    git(
        repo,
        &["for-each-ref", "--format=%(refname) %(objectname)", prefix],
    )
}

/// One blob's bytes by oid — the **value** half of a valued mark (litany ARCH
/// §3.3: `held` and `cwd` name blobs, the other four name commits).
/// `for-each-ref`'s `%(contents)` is empty for a blob, so the value is a second
/// call; it is only ever made for a ref that exists, which is why holds cost
/// nothing on the ordinary tick where none does.
pub(super) fn blob(repo: &Path, oid: &str) -> Result<Vec<u8>, GitTreeError> {
    git(repo, &["cat-file", "blob", oid])
}

/// The §9.3 config-branch browse reads (§5.1 #17–#18) and the two ancestry
/// probes beside them — every one of them a call built by [`base_cmd`] here, so
/// the env scrub is never bypassed by a second doorway.
pub(super) mod browse;
