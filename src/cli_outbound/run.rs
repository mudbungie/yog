//! **The streamed spawn** (§8): stdout and stderr piped and pumped by a reader
//! thread each, terminal exit reporting, and SIGTERM-then-SIGKILL cleanup when
//! the [`Stream`] drops. Split from [`Cli`] itself at §12's budget, on the seam
//! the module already cuts for every other spawn shape — `exec`, `piped`,
//! `detach` — leaving [`super`] to say only what a `Cli` *is*.
//!
//! `stdin` is the shape's one parameter (bl-024b): `None` closes the child's,
//! `Some(bytes)` pipes them in and closes it, which is what `piped` rides.

use std::path::Path;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

use super::chunk::pump;
use super::{Chunk, Cli, CliError, Stream};

impl Cli {
    /// Spawn `<binary> <args...>` and return a streaming handle: stdout and
    /// stderr piped, stdin closed. Dropping the `Stream` terminates the
    /// child (SIGTERM, then SIGKILL after a short grace).
    pub fn run(&self, args: &[&str]) -> Result<Stream, CliError> {
        self.run_streaming(None, &[], args, None)
    }

    /// Like [`run`](Self::run) but with the child's working directory set
    /// to `dir` — bl verbs run cwd = project (§8.2).
    pub fn run_in(&self, dir: &Path, args: &[&str]) -> Result<Stream, CliError> {
        self.run_streaming(Some(dir), &[], args, None)
    }

    /// Like [`run`](Self::run) but layering explicit env vars over the
    /// inherited environment — the config-edit drive's `EDITOR` (the
    /// `--editor-apply` shim re-entry) and `YOG_EDIT_SRC` (the staging dir),
    /// §9.3. Nothing is scrubbed; the child otherwise inherits yog's env.
    pub fn run_env(&self, env: &[(&str, &str)], args: &[&str]) -> Result<Stream, CliError> {
        self.run_streaming(None, env, args, None)
    }

    /// `stdin` is the fourth spawn shape's one difference (bl-024b): `None`
    /// closes the child's stdin, `Some(bytes)` pipes them in and closes it —
    /// the tool contract litany's executor already speaks (its ARCH §3.3), and
    /// therefore the one a tool host's own child speaks too.
    pub(super) fn run_streaming(
        &self,
        cwd: Option<&Path>,
        env: &[(&str, &str)],
        args: &[&str],
        stdin: Option<&[u8]>,
    ) -> Result<Stream, CliError> {
        // Physical spawn (§16.7 W12): the wrapper when one stands (§8.6), then
        // `program` + the namespace `prefix` (empty in host mode), then the
        // caller's args — [`wrap`]'s spawn base, built through `git_env` like
        // every child so the ambient git env is scrubbed for the whole
        // descendant tree (bl-916a).
        let mut cmd = self.spawn_base();
        cmd.args(args)
            .envs(self.standing_env())
            .envs(env.iter().copied())
            .stdin(match stdin {
                Some(_) => Stdio::piped(),
                None => Stdio::null(),
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        // The crate's one fork, which under `cargo test` holds the spawn lock
        // so no fork lands while a peer holds a not-yet-closed write fd
        // (ETXTBSY; `crate::git_env`).
        let mut child = crate::git_env::spawn(&mut cmd)
            .map_err(|e| CliError::spawn(self.exec_target(), cwd, e))?;
        // Written and closed at once: the child reads its whole input and sees
        // EOF, which is what "the input on stdin" means. A write that fails is
        // a child that closed the pipe or died — not a spawn failure, and its
        // own capture is what says so.
        if let (Some(bytes), Some(mut pipe)) = (stdin, child.stdin.take()) {
            let _ = std::io::Write::write_all(&mut pipe, bytes);
        }
        let stdout = child.stdout.take().ok_or(CliError::Stdio("stdout"))?;
        let stderr = child.stderr.take().ok_or(CliError::Stdio("stderr"))?;
        let (tx, rx) = mpsc::channel();
        let tx_err = tx.clone();
        thread::spawn(move || pump(stdout, tx, Chunk::Stdout));
        thread::spawn(move || pump(stderr, tx_err, Chunk::Stderr));
        Ok(Stream::new(child, rx))
    }

    /// The standing world env (§16.6 W2) as `(&str, &str)` pairs for
    /// [`Command::envs`] — layered **first**, so an explicit per-call `env` (the
    /// `run_env` shim vars) wins on any key while the world overrides still beat
    /// the inherited environment. Empty for a non-world `Cli`.
    pub(super) fn standing_env(&self) -> impl Iterator<Item = (&str, &str)> {
        self.env.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
