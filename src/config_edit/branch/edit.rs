//! Config-branch **edit half** (DESIGN §9.3 Y21): the scripted-`$EDITOR`
//! drive of `litany config`. This is the only lawful writer of `config/*`
//! (ARCH §2.2), so yog never writes inside a litany workspace itself — it
//! stages its drafted files, then drives `litany config`, whose `$EDITOR`
//! callback re-enters the yog binary in [`apply`](crate::config_edit::apply)
//! shim mode to copy those files over the checkout. litany performs the
//! commit.
//!
//! The flow (§9.3):
//! 1. [`stage_files`] writes the UI's drafted files into
//!    `$XDG_STATE_HOME/yog/stage/<nonce>/`.
//! 2. [`EditPlan::compose`] builds the `litany config <ws> <name> [flags]`
//!    argv plus the `EDITOR` + `YOG_EDIT_SRC` environment (pure).
//! 3. [`drive`] spawns it through the injected [`Cli`] and streams the
//!    outcome to `ops.jsonl`.
//! 4. litany execs `$EDITOR` = `<yog> --editor-apply` against the checkout;
//!    the shim copies only the staged files (see [`apply`] for the exact
//!    `sh -c` invocation shape this depends on).
//! 5. Stale `<nonce>/` dirs are swept at startup ([`sweep_staging`], §5.2).
//!
//! Steps 1 and 5 — the staging dir a draft is written into and the sweep that
//! collects the ones a crash left behind — are [`staging`], split off at §12's
//! pre-split band: what yog *stages* is a scratch-dir lifecycle, and what it
//! *drives* is a subprocess with an environment.
//!
//! [`apply`]: crate::config_edit::apply

use crate::cli_outbound::{Chunk, Cli, ExitInfo};
use crate::config_edit::apply::EDITOR_APPLY_FLAG;
use crate::opslog::{self, OpEntry, Origin};
use std::path::Path;

const SUBCOMMAND_CONFIG: &str = "config";
const ENV_EDITOR: &str = "EDITOR";
const ENV_EDIT_SRC: &str = "YOG_EDIT_SRC";

/// Step 1 and step 5: the drafted files' staging dir and the §5.2 sweep that
/// collects the ones a crash left behind.
mod staging;
pub use staging::{DraftFile, next_nonce, stage_files, stale_staging, sweep_staging};

/// Which config lineage an edit targets (mirrors litany
/// `template::authoring::Origin`, §2.2/§2.3): advance the existing branch,
/// fork a new one off a source head, or start a fresh orphan lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOrigin {
    Advance,
    Fork { source: String },
    Orphan,
}

impl EditOrigin {
    /// The `litany config` flags this origin adds after `<ws> <name>` — the
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
/// litany's `sh -c 'exec {EDITOR} "$1"'` word-splitting (see [`apply`]).
///
/// [`apply`]: crate::config_edit::apply
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The `$EDITOR` value that re-enters this binary in shim mode: the yog
/// binary path (sh-quoted) plus [`EDITOR_APPLY_FLAG`]. litany word-splits
/// it, so the quoting keeps a spaced binary path a single argv element.
pub fn editor_env_value(yog_binary: &Path) -> String {
    format!(
        "{} {EDITOR_APPLY_FLAG}",
        sh_quote(&yog_binary.display().to_string())
    )
}

/// The fully-composed spawn plan for one config-branch edit (pure): the
/// `litany config …` argv plus the two environment variables the shim needs.
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

    /// The `litany config …` argv (no binary). Test-only reader.
    #[cfg(test)]
    pub(crate) fn argv(&self) -> &[String] {
        &self.argv
    }

    /// The `EDITOR` + `YOG_EDIT_SRC` pair for the child environment (§9.3).
    pub fn env(&self) -> [(&str, &str); 2] {
        [(ENV_EDITOR, &self.editor), (ENV_EDIT_SRC, &self.staging)]
    }
}

/// Spawn `litany config …` through `cli` with the plan's environment, drain
/// the stream, and stream the outcome to `<state_root>/ops.jsonl` (§4.2).
/// `ts` is the caller's wall-clock stamp (opslog reads no clock); `cwd` is
/// the workspace, recorded for context. Logging is best-effort — the
/// operation's real effect is litany's commit — so the [`OpEntry`] is
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

#[cfg(test)]
mod tests;
