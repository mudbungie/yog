//! **Running one invocation locally** (REMOTE §5.2, bl-024b): the far end of
//! the routing leg, where a tool actually happens.
//!
//! It is lernie's own tool contract, unchanged (its ARCH §3.3): the
//! `tool_use.input` JSON on stdin, bytes on stdout, the exit code the verdict.
//! So a tool host's executable is the same kind of program a local pool tool
//! is, and the capture that comes back is the same three facts.
//!
//! **The deadline is the [`Stream`]'s drop**, which is the SIGTERM-then-SIGKILL
//! cascade the crate already owns ([`cli_outbound`](crate::cli_outbound)): a
//! child that has not finished by the deadline is left, and leaving it kills
//! it. There is no second termination path to keep in step, and nothing here
//! knows how a signal is posted.
//!
//! **Bytes become text here, once.** A capture crosses the wire as JSON and
//! ends up as a model's tool result, which is text — so the transcode happens
//! at the one place bytes stop being bytes, and nothing downstream carries an
//! encoding case. A tool whose output is not UTF-8 loses exactly the bytes no
//! `String` can name, which is the same trade every other §11 file read makes.

use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::config::Local;
use crate::cli_outbound::{Chunk, Cli, StreamPoll};
use crate::registry::mailbox::{Capture, Invocation};

/// How often a running child is looked at. A latency knob on the *answer*, not
/// on the run: the child streams into its pipes regardless.
const POLL: Duration = Duration::from_millis(20);

/// The verdict a child that outran its deadline earns — the shell's own
/// convention for `timeout`, so an operator reading a transcript recognizes it.
pub const TIMED_OUT: i32 = 124;

/// The verdict a name this machine cannot run earns — the shell's own
/// convention for "command not found", and REMOTE §5's *"a client refuses a
/// tool it no longer carries"* answered at the end that actually knows.
pub const NO_SUCH_TOOL: i32 = 127;

/// Run `invocation` against this machine's `set`, within `deadline`.
///
/// Every outcome is a [`Capture`]: a tool that ran, a tool that overran, a name
/// this host does not carry, and a command that could not be spawned at all are
/// four exit codes and four sentences, never four kinds of failure. The caller
/// posts whichever one it got, because an invocation that earned no answer
/// would be the hang the whole leg exists to exclude.
pub fn execute(set: &[Local], invocation: &Invocation, deadline: Duration) -> Capture {
    let Some(local) = super::config::position(set, &invocation.tool).and_then(|at| set.get(at))
    else {
        return refused(
            NO_SUCH_TOOL,
            &format!(
                "this machine no longer carries a tool called {:?}",
                invocation.tool
            ),
        );
    };
    match spawn(local, &invocation.input, deadline) {
        Ok(capture) => capture,
        Err(reason) => refused(NO_SUCH_TOOL, &reason),
    }
}

/// The spawn and the drain. The `Err` is a fork that never happened — a missing
/// binary, an unusable working directory — which is the one thing that is not
/// the child's own verdict.
fn spawn(local: &Local, input: &Value, deadline: Duration) -> Result<Capture, String> {
    let (head, args) = local
        .command
        .split_first()
        .ok_or("the command is an empty argv")?;
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut stream = Cli::new(head)
        .run_input(
            local.cwd.as_deref().map(Path::new),
            input.to_string().as_bytes(),
            &args,
        )
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    loop {
        match stream.try_next() {
            StreamPoll::Ready(Chunk::Stdout(bytes)) => out.extend(bytes),
            StreamPoll::Ready(Chunk::Stderr(bytes)) => err.extend(bytes),
            StreamPoll::Ready(Chunk::Exited(info)) => {
                return Ok(captured(&out, &err, info.shell_code()));
            }
            StreamPoll::Pending => {
                if started.elapsed() >= deadline {
                    // Returning drops the stream, and the drop is the kill.
                    err.extend(
                        format!("\nyog tool-host: no answer within {deadline:?}; terminated\n")
                            .into_bytes(),
                    );
                    return Ok(captured(&out, &err, TIMED_OUT));
                }
                std::thread::sleep(POLL);
            }
        }
    }
}

/// The three facts, transcoded once.
fn captured(out: &[u8], err: &[u8], exit_code: i32) -> Capture {
    Capture {
        stdout: String::from_utf8_lossy(out).into_owned(),
        stderr: String::from_utf8_lossy(err).into_owned(),
        exit_code,
    }
}

/// A refusal this machine makes about itself, in the same three facts — so the
/// model reads it exactly as it reads a tool that failed, which is what it is.
fn refused(exit_code: i32, reason: &str) -> Capture {
    Capture {
        stdout: String::new(),
        stderr: format!("yog tool-host: {reason}\n"),
        exit_code,
    }
}

#[cfg(test)]
mod tests;
