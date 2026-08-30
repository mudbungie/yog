//! The dispatch + `ops.jsonl` logging core beneath the short verbs (§8.2, §4.2
//! as amended). Every attempted action leaves a durable ops line: a completed
//! run logs its captured outcome, a **spawn failure** logs a synthetic failure
//! line (intended argv, error in `stderr`) — so no error class is ever un-logged
//! (§7.3's rendered-fact rule). Split out of [`super`](super) per §12's
//! line-budget discipline; [`super`] re-exports the surface these functions form.
//!
//! Every logging entry point takes an [`Origin`] — the §7.3 attribution
//! (bl-48f8). It is a parameter rather than a derivation because the caller is
//! the only one who knows: `bl close` and `litany message` are told apart by
//! their argv, but a ball-rung start's `litany new` and the composer's are the
//! same bytes. It is stated once, in the verb's own body, where the subject is
//! not a guess.

use std::io;
use std::path::Path;

use crate::cli_outbound::{Chunk, Cli, CliError, ExitInfo, Stream};
use crate::opslog::{self, OpEntry, Origin};

/// The captured result of a completed short verb (§8.2). `exit` follows the
/// shell convention: a plain code passes through, a signal is `128 + signum`,
/// an unobservable status is `-1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Outcome {
    /// True iff the process exited 0 — the verb succeeded.
    pub fn ok(&self) -> bool {
        self.exit == 0
    }
}

/// Drain a live piped stream into its captured [`Outcome`]. A non-zero exit is
/// *not* an error here — a `bl close` gate failure's stderr is the product to
/// log verbatim (§8.2). `pub(crate)`, wrapping [`drain`], so the no-marks knob's
/// `bl conf` seam ([`crate::world::marks`]) drains its reads/writes identically;
/// its one error is a spawn failure (nothing ran).
pub(crate) fn collect(stream: Result<Stream, CliError>) -> io::Result<Outcome> {
    Ok(drain(stream.map_err(io::Error::other)?))
}

/// Consume a live stream to end-of-output, folding stdout/stderr/exit.
fn drain(stream: Stream) -> Outcome {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut exit = ExitInfo::Unknown;
    for chunk in stream {
        match chunk {
            Chunk::Stdout(b) => out.extend(b),
            Chunk::Stderr(b) => err.extend(b),
            Chunk::Exited(e) => exit = e,
        }
    }
    Outcome {
        exit: exit.shell_code(),
        stdout: String::from_utf8_lossy(&out).into_owned(),
        stderr: String::from_utf8_lossy(&err).into_owned(),
    }
}

/// Run one short verb piped in `cwd`, then append its **attempted** outcome to
/// `<state_root>/ops.jsonl` (§8.2, §4.2 as amended). `ts` is the wall-clock
/// stamp minted at the shell boundary. A completed run — any exit, including a
/// non-zero gate failure — is logged and returned; a **spawn failure** appends a
/// synthetic failure line (the intended argv, the error in `stderr`) and returns
/// the error, so no error class is ever un-logged (§7.3's rendered-fact rule).
pub fn run_logged(
    cli: &Cli,
    state_root: &Path,
    ts: &str,
    cwd: &Path,
    args: &[&str],
    origin: Origin,
) -> io::Result<Outcome> {
    let stream = cli.run_in(cwd, args);
    log_attempt(
        cli,
        state_root,
        ts,
        cwd.display().to_string(),
        stream,
        args,
        origin,
    )
}

/// Like [`run_logged`] but with **no explicit cwd**: the child inherits yog's
/// working dir (logged blank, like the detached prompt) while the standing world
/// env (§16.6 W2) still nests it. The world seed's `litany prime` (§16.6 W3):
/// `prime` resolves its target from the standing `LITANY_HOME` (§16.2), not cwd,
/// so no cwd is threaded — and, post-W2, no per-call env either. The nesting
/// rides the world `Cli` every child carries; there is one source for it.
pub fn run_logged_cwdless(
    cli: &Cli,
    state_root: &Path,
    ts: &str,
    args: &[&str],
    origin: Origin,
) -> io::Result<Outcome> {
    let stream = cli.run(args);
    log_attempt(cli, state_root, ts, String::new(), stream, args, origin)
}

/// Append an attempt's ops line and return its outcome — the shared tail of
/// [`run_logged`]/[`run_logged_cwdless`] (§4.2 as amended). A completed run logs
/// its captured `{exit, stdout, stderr}`; a spawn failure logs a synthetic line
/// ([`OpEntry::synthetic_failure`]) and returns the error. `cwd` is the field to
/// record (blank for the cwd-inheriting spawn).
fn log_attempt(
    cli: &Cli,
    state_root: &Path,
    ts: &str,
    cwd: String,
    stream: Result<Stream, CliError>,
    args: &[&str],
    origin: Origin,
) -> io::Result<Outcome> {
    let argv = argv_of(cli, args);
    match stream {
        Ok(stream) => {
            let outcome = drain(stream);
            opslog::append(
                state_root,
                &OpEntry {
                    ts: ts.to_string(),
                    argv,
                    cwd,
                    exit: outcome.exit,
                    stdout: outcome.stdout.clone(),
                    stderr: outcome.stderr.clone(),
                    origin,
                },
            )?;
            Ok(outcome)
        }
        Err(spawn) => {
            let entry =
                OpEntry::synthetic_failure(ts.to_string(), argv, cwd, spawn.to_string(), origin);
            opslog::append(state_root, &entry)?;
            Err(io::Error::other(spawn))
        }
    }
}

/// The argv an ops line records: the resolved binary path, then the verb args.
fn argv_of(cli: &Cli, args: &[&str]) -> Vec<String> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(cli.binary().display().to_string());
    argv.extend(args.iter().map(|s| (*s).to_string()));
    argv
}

/// Append a **completed** non-spawn step's line to `ops.jsonl` (§4.2:
/// `["yog-step", <step>]` at exit 0). The §3.6 unmaking's directory removal logs
/// through this: the deletion is itself an ops event, and a trail that deletes
/// with its subject is not a trail.
pub fn log_step_done(
    state_root: &Path,
    ts: &str,
    cwd: &Path,
    step: &str,
    origin: Origin,
) -> io::Result<()> {
    opslog::append(
        state_root,
        &OpEntry::step_done(ts.to_string(), step, cwd.display().to_string(), origin),
    )
}

/// Append a non-spawn **step-failure** line to `ops.jsonl` (§4.2: `["yog-step",
/// <step>]`, the mint/mkdir/cross-check class that names no binary). The covered
/// encoding the start flow (Z3) logs its non-spawn aborts through, so every
/// error class leaves the §7.3 rendered fact — not a dropped error.
pub fn log_step_failure(
    state_root: &Path,
    ts: &str,
    cwd: &Path,
    step: &str,
    err: &str,
    origin: Origin,
) -> io::Result<()> {
    opslog::append(
        state_root,
        &OpEntry::step_failure(
            ts.to_string(),
            step,
            cwd.display().to_string(),
            err.to_string(),
            origin,
        ),
    )
}
