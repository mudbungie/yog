//! The **confinement wrapper** seam (DESIGN §8.6): a physical argv prefix
//! standing *in front of* the program, the way `prefix` stands behind it.
//!
//! A wrapped spawn is `wrapper[0] wrapper[1..] <program> <prefix> <args…>` —
//! the sandbox program first, its flags, then everything the unwrapped spawn
//! would have been. Like the standing env, this crate stays generic: it
//! prepends whatever words it is handed and knows nothing of *which* backend
//! they name (that fact lives in [`crate::control::confine`]). The logical
//! [`binary`](Cli::binary) is untouched, so the ops-log argv (§8.2) and the
//! world's tool shim (§16.7 W9, written from [`exec_words`](Cli::exec_words))
//! both stay wrapper-blind — the shim runs *inside* the sandbox, where
//! re-wrapping would nest a second one, and the trail records the act, not
//! the envelope it ran in.

use std::path::Path;

use super::Cli;

impl Cli {
    /// A clone with `wrapper` standing in front of every spawn — empty is the
    /// unwrapped spawn, byte-identical (the general path with empty inputs).
    /// `pub(crate)`: an internal spawn seam, like [`and_env`](Cli::and_env).
    pub(crate) fn and_wrapper(&self, wrapper: Vec<String>) -> Self {
        let mut cli = self.clone();
        cli.wrapper = wrapper;
        cli
    }

    /// The command every spawn shape starts from: the wrapper words when one
    /// stands, then `program` + the namespace `prefix` — built through
    /// [`git_env`](crate::git_env) like every child, so the ambient git env is
    /// scrubbed for the whole descendant tree (bl-916a).
    pub(super) fn spawn_base(&self) -> std::process::Command {
        let mut cmd = crate::git_env::command(self.exec_target());
        if let Some((_, flags)) = self.wrapper.split_first() {
            cmd.args(flags).arg(&self.program);
        }
        cmd.args(&self.prefix);
        cmd
    }

    /// What actually execs: the wrapper program when one stands, else
    /// `program` — the honest name for a spawn *failure*, where the file the
    /// OS could not find is the wrapper, not the tool it would have run.
    pub(super) fn exec_target(&self) -> &Path {
        self.wrapper
            .first()
            .map_or(self.program.as_path(), Path::new)
    }
}
