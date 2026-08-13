//! The two world escape hatches (DESIGN §8.4, §16.4) — the human counterpart to
//! the embedded-crate agent tools. Both are multi-call subcommands of the yog
//! binary, beside `--editor-apply` (§9.3), and both are **pure argv → plan**
//! here; `main.rs` does the thin dispatch (print, or spawn-and-exit).
//!
//! - `yog env` prints the world's `export` lines ([`env_script`]) so
//!   `eval "$(yog env)"` drops the caller's shell *into* the world (§16.2),
//!   where a bare `bl`/`lernie`/`bz` is the world's own shim — each an exec of
//!   yog's embedded substrate against the nested state (§16.4; the phase-1
//!   "ambient binaries on nested state" reading died with the batteries).
//!   Values are shell-quoted ([`shell_quote`]) so a path with a space or a
//!   quote survives `eval` intact.
//! - `yog exec <cmd…>` runs one command inside the world: [`parse_exec`] turns
//!   `[--cwd DIR] <cmd> [args…]` into an [`ExecPlan`], which `main.rs` runs
//!   through [`Cli::exec_in_world`](crate::cli_outbound::Cli::exec_in_world)
//!   (world overrides layered over the inherited env, stdio inherited, the
//!   child's exit faithfully yog's).
//!
//! Both hatches **converge the world's tool shims first** (bl-44a5,
//! [`tools::ensure_tools`](crate::world::tools::ensure_tools) from `main.rs`):
//! the `PATH` they hand out fronts `world/tools/` unconditionally, so the dir
//! is materialized wherever the world is handed out — never only by a Start.

use std::path::PathBuf;

/// The `yog env` subcommand token (argv\[1\]).
pub const ENV_SUBCMD: &str = "env";
/// The `yog exec` subcommand token (argv\[1\]).
pub const EXEC_SUBCMD: &str = "exec";
/// The optional leading `--cwd <dir>` flag of `yog exec` (§8.4).
pub const CWD_FLAG: &str = "--cwd";
/// The optional leading `--ws <workspace>` flag of **both** hatches (§8.4 as
/// amended by bl-b589) — the workspace whose **wall** the hatch stands in.
///
/// It is spelled and read exactly as `yog gesture --ws` is: the workspace's
/// **path**, whose §3.1 leaf keys the wall. One flag, one meaning, wherever a
/// seat has to name a sphere it is not already inside.
pub const WS_FLAG: &str = "--ws";

/// POSIX single-quote escaping: wrap `s` in `'…'` and rewrite each embedded
/// `'` as `'\''` (close-quote, an escaped literal quote, reopen-quote). The
/// result is one shell word that `eval` reproduces byte-for-byte — empty
/// strings, spaces, and quotes included — so `export VAR=<shell_quote(v)>` is
/// safe for any value.
pub fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// The `yog env` product: one `export <VAR>='<value>'` line per world override
/// (§16.2 — `LERNIE_HOME`, `XDG_STATE_HOME`, then the §16.7 W9 `PATH` prepend,
/// in that order), each value [`shell_quote`]d, newline-terminated. Fed the same
/// [`world::overrides`](crate::world::overrides) every spawn layers, so the
/// dir a human's `eval`'d shell writes is the dir yog watches.
pub fn env_script(overrides: &[(String, String)]) -> String {
    let mut out = String::new();
    for (key, value) in overrides {
        out.push_str("export ");
        out.push_str(key);
        out.push('=');
        out.push_str(&shell_quote(value));
        out.push('\n');
    }
    out
}

/// A parsed `yog exec` invocation: the command, its arguments, and the optional
/// working directory the child runs in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecPlan {
    /// `--cwd <dir>` when given; the child otherwise inherits yog's cwd.
    pub cwd: Option<PathBuf>,
    /// `--ws <workspace>` when given: the child runs inside that workspace's
    /// wall, so a `bz` it spawns reaches that sphere's providers, sign-ins and
    /// model cache. Absent, the child has the world and no wall.
    pub workspace: Option<PathBuf>,
    /// The command to run — an arbitrary binary, taken verbatim.
    pub cmd: String,
    /// The command's own arguments (everything after `<cmd>`).
    pub args: Vec<String>,
}

/// Why a hatch argv could not be turned into a plan.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ExecError {
    /// `--cwd` was given with no directory following it.
    #[error("--cwd requires a directory argument")]
    MissingCwdValue,
    /// `--ws` was given with no workspace following it.
    #[error("--ws requires a workspace path")]
    MissingWsValue,
    /// No command was given (an empty argv, or only the leading flags).
    #[error("no command given (usage: yog exec [--cwd DIR] [--ws WORKSPACE] <cmd> [args...])")]
    MissingCommand,
    /// `yog env` was given a word it has no use for.
    #[error("yog env takes no arguments but [--ws WORKSPACE]; got {0:?}")]
    UnexpectedEnvArg(String),
}

/// The leading flags a hatch reads before the command begins — `--cwd` and
/// `--ws`, in either order (§8.4). Only a *leading* one is yog's: once the
/// command word is reached, everything after it belongs to the command, so
/// `yog exec --ws A env --ws B` passes `--ws B` through untouched.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Leading {
    /// `--cwd <dir>`: the child's working directory.
    pub cwd: Option<PathBuf>,
    /// `--ws <workspace>`: the workspace whose wall stands over the world.
    pub workspace: Option<PathBuf>,
}

/// Read the leading flags off `args`, returning them and the remaining argv.
/// Pure and total: an unrecognized word simply ends the flags and begins the
/// command, which is what keeps the command's own argv verbatim.
fn leading(args: &[String]) -> Result<(Leading, &[String]), ExecError> {
    let mut read = Leading::default();
    let mut rest = args;
    while let Some((flag, tail)) = rest.split_first() {
        let slot = match flag.as_str() {
            CWD_FLAG => &mut read.cwd,
            WS_FLAG => &mut read.workspace,
            _ => break,
        };
        let missing = if flag == CWD_FLAG {
            ExecError::MissingCwdValue
        } else {
            ExecError::MissingWsValue
        };
        let (value, more) = tail.split_first().ok_or(missing)?;
        *slot = Some(PathBuf::from(value));
        rest = more;
    }
    Ok((read, rest))
}

/// Parse the argv **after** `env` (§8.4, bl-b589): nothing, or `--ws
/// <workspace>`. Refuses anything else by name rather than ignoring it — a
/// hatch that silently dropped a word would hand out the wrong environment,
/// which is the one thing it must not do.
pub fn parse_env(args: &[String]) -> Result<Leading, ExecError> {
    let (read, rest) = leading(args)?;
    match (rest.first(), read.cwd.is_some()) {
        (Some(extra), _) => Err(ExecError::UnexpectedEnvArg(extra.clone())),
        (None, true) => Err(ExecError::UnexpectedEnvArg(CWD_FLAG.to_owned())),
        (None, false) => Ok(read),
    }
}

/// The environment a hatch hands out (§16.2, bl-b589): the world's own
/// overrides, and — when the seat named a workspace — that workspace's **wall**
/// layered on top, exactly as [`wall::pairs`](crate::world::wall::pairs) layers
/// it on every workspace-bound spawn the window makes.
///
/// This is the whole of the headless workspace binding. Naming no workspace
/// layers nothing, so a bare `yog env` still hands out the world and only the
/// world: brazen then finds no wall and refuses, and credentials never fall
/// back to the machine's own (§16.2's rule, kept).
pub fn overrides_for(
    ambient: &crate::xdg::Env,
    workspace: Option<&std::path::Path>,
) -> Vec<(String, String)> {
    let mut overrides = super::overrides(ambient);
    if let Some(ws) = workspace {
        overrides.extend(super::wall::pairs(ambient, ws));
    }
    overrides
}

/// Parse the argv **after** `exec` into an [`ExecPlan`]: the optional leading
/// flags ([`leading`]), then `<cmd> [args…]`. Pure; `main.rs` runs the plan.
pub fn parse_exec(args: &[String]) -> Result<ExecPlan, ExecError> {
    let (read, rest) = leading(args)?;
    let (cmd, cmd_args) = rest.split_first().ok_or(ExecError::MissingCommand)?;
    Ok(ExecPlan {
        cwd: read.cwd,
        workspace: read.workspace,
        cmd: cmd.clone(),
        args: cmd_args.to_vec(),
    })
}

#[cfg(test)]
mod tests;
