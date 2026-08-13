//! CLI outbound: the frontend's sole command surface to the harness. Every user
//! action is an `exec(<binary>, args)` and nothing else (ARCH §3.4/§3.5;
//! "resume" is no longer user-facing per the §2.9 amendment, bl-abf3).
//!
//! Three spawn shapes over one binary abstraction (DESIGN §8):
//! - [`Cli::run`] / [`Cli::run_in`] stream stdout/stderr with terminal exit
//!   reporting and aggressive SIGTERM-then-SIGKILL cleanup on [`Stream`] drop;
//!   `run_in` sets the child's `current_dir` (bl verbs run cwd = project, §8.2).
//! - [`Cli::spawn_detached`] fires a child in its own process group
//!   (`process_group(0)`, safe std — a new group, not a session; enough, since
//!   terminal signals hit the foreground group), stdin/stdout null, stderr to a
//!   caller-named per-spawn sink file, no pipe and no signal — long-lived drivers
//!   (§8.1) that yog's exit can't kill. Its one retained thread only *reaps* the
//!   child (bl-3016): detachment never made yog stop being the parent, so
//!   somebody has to take the status the kernel is holding.
//! - [`Streamed`] consumes a [`Cli::run`] child non-blocking and line-buffered,
//!   live (§8's streamed-piped class: `bz --login`, [`crate::login`]).
//!
//! The crate's one remaining `unsafe` is the SIGTERM in [`sys`]. Which binary a
//! [`Cli`] execs — and under what leading argv — is [`resolve`]'s concern: a
//! host PATH name / `*_BINARY` override, or (§16.7 W12) yog's own `current_exe`
//! under a namespace prefix. A [`Cli`] carries the *physical* `program` +
//! `prefix` it execs and derives the *logical* [`binary`](Cli::binary) name
//! from them, so the ops-log argv (§8.2) is invariant across that switch. Pure
//! Rust — no egui — so a future `lernie-ui-web` crate reuses it unchanged; the
//! caller supplies argv.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

/// The values a running child hands back — [`Chunk`], [`ExitInfo`], [`CliError`]
/// — and the reader-thread pump behind them.
mod chunk;
use chunk::pump;
pub use chunk::{Chunk, CliError, ExitInfo, work_dir_fault};

/// Binary resolution — the host/self-multiplex switch (§16.7 W12) + [`Binary`].
pub(crate) mod resolve;
pub use resolve::Binary;

mod stream;
pub use stream::{Stream, StreamPoll};

mod streamed;
pub use streamed::{
    Streamed, StreamedLine, StreamedOutcome, StreamedPoll, stderr_text, stdout_text,
};

/// The `yog exec` world escape hatch spawn (§8.4): [`Cli::exec_in_world`].
mod exec;

/// The fire-and-forget detached spawn (§8.1): [`Cli::spawn_detached`] and its
/// per-spawn stderr sink. Split out to hold [`self`] under the 300-line cap.
mod detach;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    /// The **physical** executable this `Cli` execs. Host mode: the resolved
    /// tool (PATH name or `*_BINARY` override). Self-multiplex mode (§16.7 W12):
    /// yog's own `current_exe`. See [`resolve`].
    program: PathBuf,
    /// The **physical** argv prepended before the caller's args. Empty in host
    /// mode; `[<namespace>]` in self-multiplex mode, so the spawn is `yog
    /// <namespace> <args…>`. Also the *logical*-name source — [`binary`](Self::binary)
    /// (the ops-log argv[0], §8.2) reads `prefix[0]` when present, else
    /// `program` — so ops rows log the logical `["lernie", …]` whatever the
    /// physical target.
    prefix: Vec<String>,
    /// The **standing** env overrides layered over the inherited environment on
    /// every spawn (§16.6 W2): the composed world's nesting set (`LERNIE_HOME`/
    /// `XDG_STATE_HOME`, §16.2) when built through
    /// [`resolve_in_world`](Self::resolve_in_world), empty otherwise. Carrying it
    /// at construction makes nesting impossible to forget at a new call site — a
    /// world `Cli` already nests every `run`/`run_in`/`run_env`/`spawn_detached`.
    /// This crate stays generic: it layers whatever pairs it is handed, knowing
    /// nothing of *which* vars nest (that fact lives in [`crate::world`]).
    env: Vec<(String, String)>,
}

impl Cli {
    /// Stand `env` on every spawn (builder) — the world-`Cli` seam for
    /// [`resolve_in_world`](Self::resolve_in_world), and the only way to put a
    /// **recorder** in the world without PATH resolution. `pub` for that second
    /// reason: STORIES S8-T2 asserts that every dispatched verb nests by
    /// construction, which is a claim about what a *child* observes, and the
    /// `tests/` crate cannot mutate process env to arrange it any other way
    /// (`set_var` is `unsafe` under the parallel runner). Owned in, owned out.
    #[must_use]
    pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
        self.env = env;
        self
    }

    /// A clone with `extra` appended to the standing env — the workspace-scoped
    /// `YOG_NAME=<name>` layer (§8, §3.3). `pub(crate)`, an internal spawn seam.
    pub(crate) fn and_env(&self, extra: Vec<(String, String)>) -> Self {
        let mut cli = self.clone();
        cli.env.extend(extra);
        cli
    }

    /// The **logical** binary name — the ops-log argv[0] (§8.2) and every argv
    /// projection ([`crate::login`], [`crate::config_edit`], [`crate::start`]).
    /// Derived from the physical target: the namespace `prefix[0]` in
    /// self-multiplex mode, else the `program` itself — so a spawn retargeted to
    /// `yog <namespace>` (§16.7 W12) still logs `["lernie", …]`.
    pub(crate) fn binary(&self) -> &Path {
        self.prefix
            .first()
            .map_or(self.program.as_path(), |ns| Path::new(ns))
    }

    /// The **physical** argv words that exec this tool: `program` followed by
    /// the namespace `prefix` (empty in host mode). One spawn of this `Cli` is
    /// exactly these words plus the caller's args — which is why the world's
    /// tool shim (§16.7 W9) is written from them: the shim an agent runs can
    /// never name a different target than yog's own spawns. Owned (rule 2).
    pub(crate) fn exec_words(&self) -> Vec<String> {
        let mut words = vec![self.program.to_string_lossy().into_owned()];
        words.extend(self.prefix.iter().cloned());
        words
    }

    /// The **physical** executable this `Cli` execs — `program` (the fields
    /// themselves drive every spawn; this accessor is the resolution tests'
    /// window onto the host/self-multiplex split, §16.7 W12).
    #[cfg(test)]
    pub(crate) fn program(&self) -> &Path {
        &self.program
    }

    /// The **physical** argv prefix prepended before the caller's args (empty in
    /// host mode; `[<namespace>]` in self-multiplex mode) — a test window.
    #[cfg(test)]
    pub(crate) fn prefix(&self) -> &[String] {
        &self.prefix
    }

    /// Spawn `<binary> <args...>` and return a streaming handle: stdout and
    /// stderr piped, stdin closed. Dropping the `Stream` terminates the
    /// child (SIGTERM, then SIGKILL after a short grace).
    pub fn run(&self, args: &[&str]) -> Result<Stream, CliError> {
        self.run_streaming(None, &[], args)
    }

    /// Like [`run`](Self::run) but with the child's working directory set
    /// to `dir` — bl verbs run cwd = project (§8.2).
    pub fn run_in(&self, dir: &Path, args: &[&str]) -> Result<Stream, CliError> {
        self.run_streaming(Some(dir), &[], args)
    }

    /// Like [`run`](Self::run) but layering explicit env vars over the
    /// inherited environment — the config-edit drive's `EDITOR` (the
    /// `--editor-apply` shim re-entry) and `YOG_EDIT_SRC` (the staging dir),
    /// §9.3. Nothing is scrubbed; the child otherwise inherits yog's env.
    pub fn run_env(&self, env: &[(&str, &str)], args: &[&str]) -> Result<Stream, CliError> {
        self.run_streaming(None, env, args)
    }

    fn run_streaming(
        &self,
        cwd: Option<&Path>,
        env: &[(&str, &str)],
        args: &[&str],
    ) -> Result<Stream, CliError> {
        // Physical spawn (§16.7 W12): `program` + the namespace `prefix` (empty
        // in host mode), then the caller's args. Built through `git_env` like
        // every child: the ambient git env is scrubbed for the whole descendant
        // tree, so a `bl`/`lernie` that forks git of its own accord cannot be
        // re-aimed at a hook's repo (bl-916a).
        let mut cmd = crate::git_env::command(&self.program);
        cmd.args(&self.prefix)
            .args(args)
            .envs(self.standing_env())
            .envs(env.iter().copied())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        // Callers hold `SPAWN_LOCK` across the spawn so no fork lands while a peer holds a not-yet-closed write fd (ETXTBSY; test_support).
        let mut child = cmd
            .spawn()
            .map_err(|e| CliError::spawn(&self.program, cwd, e))?;
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
    fn standing_env(&self) -> impl Iterator<Item = (&str, &str)> {
        self.env.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
/// The crate's one confined `unsafe`: raw `SIGTERM` in [`Stream`]'s drop.
mod sys;

#[cfg(test)]
mod tests;
