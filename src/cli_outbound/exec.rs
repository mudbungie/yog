//! The `yog exec` world escape hatch's spawn shape (DESIGN §8.4, §16.4): run a
//! **foreign** command inside yog's composed world and block until it exits.
//! Split from [`super`] so this shape — a blocking wait with **inherited** stdio
//! (the child owns yog's terminal), distinct from the piped
//! [`run`](super::Cli::run)/[`run_in`](super::Cli::run_in) family — keeps
//! [`super`] under the 300-line cap. It is `Cli::new + with_env`
//! construction (§16.6 W2): the world overrides stand over the inherited env
//! exactly as they do for every other world spawn, so the command sees the
//! nested `$XDG_STATE_HOME`/`$LERNIE_HOME` a human joining the
//! world by hand would.

use std::path::Path;

use super::{Cli, CliError, ExitInfo, stream};

impl Cli {
    /// Run `binary args…` inside the world, blocking until it exits and
    /// returning its [`ExitInfo`] (the child's fate faithfully — a plain exit
    /// code or the terminating signal, classified the same way as every piped
    /// stream). `binary` is an arbitrary command taken verbatim (unlike
    /// [`resolve_in_world`](Cli::resolve_in_world), which PATH-resolves a known
    /// [`Binary`](super::Binary)); `overrides` are the world's nesting set
    /// (§16.2), layered over the inherited environment; `cwd` sets the working
    /// directory when given. Stdio is **inherited** — `yog exec <cmd>` behaves
    /// exactly as running `<cmd>` in a world shell. The one error is a spawn
    /// failure (e.g. command not found) — nothing ran.
    pub fn exec_in_world(
        binary: &str,
        overrides: &[(String, String)],
        cwd: Option<&Path>,
        args: &[&str],
    ) -> Result<ExitInfo, CliError> {
        let cli = Self::new(binary).with_env(overrides.to_vec());
        let mut cmd = crate::git_env::command(cli.binary());
        // Standing world env first, stdio inherited by default (`status`):
        // the child owns the terminal. Callers hold `SPAWN_LOCK` across the
        // spawn (test_support) so no fork races a peer's open write fd.
        cmd.args(args).envs(cli.standing_env());
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let status = cmd
            .status()
            .map_err(|source| CliError::spawn(cli.binary(), cwd, source))?;
        Ok(stream::exit_info(Some(status)))
    }
}
