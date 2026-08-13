//! Config-branch **edit half** (DESIGN §9.3 Y21): the scripted-`$EDITOR`
//! drive of `lernie config`. This is the only lawful writer of `config/*`
//! (ARCH §2.2), so yog never writes inside a lernie workspace itself — it
//! stages its drafted files, then drives `lernie config`, whose `$EDITOR`
//! callback re-enters the yog binary in [`apply`](crate::config_edit::apply)
//! shim mode to copy those files over the checkout. lernie performs the
//! commit.
//!
//! The flow (§9.3):
//! 1. [`stage_files`] writes the UI's drafted files into
//!    `$XDG_STATE_HOME/yog/stage/<nonce>/`.
//! 2. [`EditPlan::compose`] builds the `lernie config <ws> <name> [flags]`
//!    argv plus the `EDITOR` + `YOG_EDIT_SRC` environment (pure).
//! 3. [`drive`] spawns it through the injected [`Cli`] and streams the
//!    outcome to `ops.jsonl`.
//! 4. lernie execs `$EDITOR` = `<yog> --editor-apply` against the checkout;
//!    the shim copies only the staged files (see [`apply`] for the exact
//!    `sh -c` invocation shape this depends on).
//! 5. Stale `<nonce>/` dirs are swept at startup ([`sweep_staging`], §5.2).
//!
//! [`apply`]: crate::config_edit::apply

use crate::cli_outbound::{Chunk, Cli, ExitInfo};
use crate::config_edit::apply::EDITOR_APPLY_FLAG;
use crate::opslog::{self, OpEntry, Origin};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const SUBCOMMAND_CONFIG: &str = "config";
const ENV_EDITOR: &str = "EDITOR";
const ENV_EDIT_SRC: &str = "YOG_EDIT_SRC";
/// Staging dirs untouched for longer than this are swept at startup (§5.2).
const STALE_SECS: i64 = 24 * 60 * 60;

/// Which config lineage an edit targets (mirrors lernie
/// `template::authoring::Origin`, §2.2/§2.3): advance the existing branch,
/// fork a new one off a source head, or start a fresh orphan lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOrigin {
    Advance,
    Fork { source: String },
    Orphan,
}

impl EditOrigin {
    /// The `lernie config` flags this origin adds after `<ws> <name>` — the
    /// exact surface of `Command::Config` (`--from`/`--orphan`).
    fn flags(&self) -> Vec<String> {
        match self {
            EditOrigin::Advance => Vec::new(),
            EditOrigin::Fork { source } => vec!["--from".to_string(), source.clone()],
            EditOrigin::Orphan => vec!["--orphan".to_string()],
        }
    }
}

/// Single-quote `s` as one POSIX `sh` word so a path with spaces survives
/// lernie's `sh -c 'exec {EDITOR} "$1"'` word-splitting (see [`apply`]).
///
/// [`apply`]: crate::config_edit::apply
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The `$EDITOR` value that re-enters this binary in shim mode: the yog
/// binary path (sh-quoted) plus [`EDITOR_APPLY_FLAG`]. lernie word-splits
/// it, so the quoting keeps a spaced binary path a single argv element.
pub fn editor_env_value(yog_binary: &Path) -> String {
    format!(
        "{} {EDITOR_APPLY_FLAG}",
        sh_quote(&yog_binary.display().to_string())
    )
}

/// A drafted config file: a checkout-relative path and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftFile {
    pub rel_path: String,
    pub bytes: Vec<u8>,
}

/// A collision-safe staging nonce: `<pid>-<counter>`. No clock / randomness
/// (per the task) — the pid scopes it to this process and the monotonic
/// counter to this call, so concurrent edits never share a dir.
pub fn next_nonce() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{n}", std::process::id())
}

/// Write the drafted `files` into `<staging_root>/<nonce>/` (creating parent
/// dirs) and return that staging dir. An empty `files` still creates the
/// dir — the shim then copies nothing and lernie declines the empty commit.
pub fn stage_files(
    staging_root: &Path,
    nonce: &str,
    files: &[DraftFile],
) -> std::io::Result<PathBuf> {
    let dir = staging_root.join(nonce);
    std::fs::create_dir_all(&dir)?;
    for f in files {
        let dest = dir.join(&f.rel_path);
        // A path joined under `dir` always has a parent (at worst `dir`
        // itself); the fallback keeps the staging write panic-free.
        std::fs::create_dir_all(dest.parent().unwrap_or(&dir))?;
        std::fs::write(&dest, &f.bytes)?;
    }
    Ok(dir)
}

/// The fully-composed spawn plan for one config-branch edit (pure): the
/// `lernie config …` argv plus the two environment variables the shim needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditPlan {
    argv: Vec<String>,
    editor: String,
    staging: String,
}

impl EditPlan {
    /// Compose the plan. `argv` is `config <ws> <name> [--from src|--orphan]`;
    /// `EDITOR` re-enters `yog_binary` in shim mode; `YOG_EDIT_SRC` is
    /// `staging_dir`.
    pub fn compose(
        yog_binary: &Path,
        workspace: &Path,
        name: &str,
        origin: &EditOrigin,
        staging_dir: &Path,
    ) -> Self {
        let mut argv = vec![
            SUBCOMMAND_CONFIG.to_string(),
            workspace.display().to_string(),
            name.to_string(),
        ];
        argv.extend(origin.flags());
        Self {
            argv,
            editor: editor_env_value(yog_binary),
            staging: staging_dir.display().to_string(),
        }
    }

    /// The `lernie config …` argv (no binary). Test-only reader.
    #[cfg(test)]
    pub(crate) fn argv(&self) -> &[String] {
        &self.argv
    }

    /// The `EDITOR` + `YOG_EDIT_SRC` pair for the child environment (§9.3).
    pub fn env(&self) -> [(&str, &str); 2] {
        [(ENV_EDITOR, &self.editor), (ENV_EDIT_SRC, &self.staging)]
    }
}

/// Spawn `lernie config …` through `cli` with the plan's environment, drain
/// the stream, and stream the outcome to `<state_root>/ops.jsonl` (§4.2).
/// `ts` is the caller's wall-clock stamp (opslog reads no clock); `cwd` is
/// the workspace, recorded for context. Logging is best-effort — the
/// operation's real effect is lernie's commit — so the [`OpEntry`] is
/// returned regardless for the UI to surface. A spawn failure is itself a
/// non-zero (-1) outcome carrying the error.
///
/// `origin` is the [`opslog::Origin`] the row is attributed to (§7.3, bl-48f8)
/// — the *banner surface*, unrelated to this module's [`EditOrigin`], which is
/// a branching mode. An operator's own config edit is
/// [`Origin::World`]: this function hands the entry straight back and the §9
/// pane states its outcome in place, so a banner elsewhere would be the same
/// error twice. A non-operator caller can pass a different origin — its
/// failure then banners on that caller's own surface instead.
pub fn drive(
    cli: &Cli,
    workspace: &Path,
    plan: &EditPlan,
    ts: &str,
    state_root: &Path,
    origin: Origin,
) -> OpEntry {
    let args: Vec<&str> = plan.argv.iter().map(String::as_str).collect();
    let (stdout, stderr, exit) = match cli.run_env(&plan.env(), &args) {
        Ok(stream) => collect(stream),
        Err(e) => (String::new(), e.to_string(), -1),
    };
    let mut argv = vec![cli.binary().display().to_string()];
    argv.extend(plan.argv.iter().cloned());
    let entry = OpEntry {
        ts: ts.to_string(),
        argv,
        cwd: workspace.display().to_string(),
        exit,
        stdout,
        stderr,
        origin,
    };
    let _ = opslog::append(state_root, &entry);
    entry
}

/// Drain a stream to `(stdout, stderr, exit-code)`; a signal/unknown exit is
/// recorded as -1.
fn collect(stream: crate::cli_outbound::Stream) -> (String, String, i32) {
    let (mut out, mut err, mut code) = (Vec::new(), Vec::new(), -1);
    for chunk in stream {
        match chunk {
            Chunk::Stdout(b) => out.extend(b),
            Chunk::Stderr(b) => err.extend(b),
            Chunk::Exited(ExitInfo::Code(c)) => code = c,
            Chunk::Exited(_) => code = -1,
        }
    }
    (
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
        code,
    )
}

/// Pure decision (clock-injected): the staging dirs whose mtime is more than
/// 24 h before `now_secs`. `now_secs` and each mtime are unix seconds, so
/// every arm is deterministic under test.
pub fn stale_staging(now_secs: i64, dirs: &[(PathBuf, i64)]) -> Vec<PathBuf> {
    dirs.iter()
        .filter(|(_, mtime)| now_secs - mtime > STALE_SECS)
        .map(|(p, _)| p.clone())
        .collect()
}

/// Sweep `<stage_root>/*`: best-effort delete every `<nonce>/` dir untouched
/// for over 24 h (§5.2 startup sweep). A missing root is a no-op; the wall
/// clock is the caller's (main.rs), keeping the decision ([`stale_staging`])
/// pure. Returns the dirs decided stale.
pub fn sweep_staging(stage_root: &Path, now_secs: i64) -> Vec<PathBuf> {
    let stale = stale_staging(now_secs, &staging_dirs(stage_root));
    for dir in &stale {
        let _ = std::fs::remove_dir_all(dir);
    }
    stale
}

/// Enumerate `<stage_root>/*` sub-dirs paired with their mtime (unix secs).
/// A missing root, or any entry that cannot be stat'd (a dangling symlink, a
/// racing unlink), contributes nothing — enumeration is best-effort.
fn staging_dirs(stage_root: &Path) -> Vec<(PathBuf, i64)> {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir(stage_root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata()
            && meta.is_dir()
        {
            out.push((entry.path(), meta.mtime()));
        }
    }
    out
}

#[cfg(test)]
mod tests;
