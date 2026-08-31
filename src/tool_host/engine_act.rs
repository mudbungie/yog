//! **The compactor's procedure pair, answered as an engine act** (REMOTE §5.4,
//! bl-dfce): `write_summary` and `mark_for_deletion`.
//!
//! The seam inversion made the router *total* — while an injection is installed
//! it answers every invocation the agent makes, and nothing resolves a binary
//! behind it. litany has a second source of injected tool definitions besides
//! the host: the calling role's own procedure, which today is exactly these two
//! names. Under the inversion they reach yog's router like any other name.
//!
//! **They are not tools on a machine, so they are not a thrall's.** The
//! subject-locality invariant (REMOTE §5: *"a tool executes where its subject
//! lives"*) decides it and nothing else has to. `write_summary` writes the
//! conversation's own summary onto the compactor branch; `mark_for_deletion`
//! nominates that same conversation's files. Their subject is the conversation,
//! the conversation lives on the server, and the server is what yog holds — so
//! no machine is involved, no thrall is involved, and REMOTE §12's *front door
//! only* governs execution **on a machine**, which this is not. Shipping them
//! to a thrall would send a box that does not hold the world a request about
//! it. The operator's ruling states the principle it falls out of: context
//! management happens in yog.
//!
//! **litany defines what the pair does; yog only decides where.** The acts
//! themselves are the engine's (its `ARCH §2.7` compactor toolset: the summary
//! numbering, the refusal to nominate the dispatch entry, the staged `git rm`),
//! and none of it is restated here. yog performs them by re-entering the
//! engine's own front door — `<driver_target> tool <name>`, the third hop
//! litany's own resolution addressed before the inversion, with the caller
//! identity on the child's environment and the `tool_use` input on its stdin.
//! The child is yog, re-executed under the `litany` namespace by the world's
//! shim, standing in the same world; nothing of the compactor's semantics is
//! reimplemented, so the pair has exactly one definition and it is upstream's.
//!
//! **The tool control still sees them, and yog has no say in that.** litany
//! adjudicates in the tool window, *before* the executor is entered, for every
//! name including an injected one — so `yog tool-control` judges these two
//! exactly as it judges every other invocation, and the router (which lives
//! inside execution) could not exempt them if it wanted to. There is no
//! carve-out at the chokepoint, and none was needed.

use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use ::litany::cmd::{RoutedCall, RoutedCapture};

use crate::cli_outbound::{Chunk, Cli, StreamPoll};

/// **The engine-act name set, closed and enumerated here and nowhere else.**
/// Six rows since bl-77be, in two families, each admitted by the
/// subject-locality invariant (REMOTE §5: *"a tool executes where its subject
/// lives"*) and by nothing else:
///
/// - the compactor's procedure pair (`write_summary`, `mark_for_deletion`,
///   REMOTE §5.4, bl-dfce) — the conversation's own summary and files;
/// - the conversation-subject worker grants (REMOTE §5.4 as amended by
///   bl-77be): `dispatch` mints and launches a child conversation on the
///   workspace the server holds, `message` deposits into another
///   conversation's inbox, `load_skill` copies a server-disk skill into the
///   agent's server-disk worktree, and `cd` writes the agent's
///   working-directory mark, a ref on the workspace. None of them is work on
///   a *machine*, so none of them is a thrall's — routing them anywhere would
///   send a box that does not hold the world a request about it.
///
/// What is deliberately NOT here: `bash`, `read_file`, `apply_patch` — acts
/// at the conversation's working directory, which is machine work and takes
/// the worktree lane ([`super::subject`]); and `multi_tool`, which litany's
/// own step loop fans out before any router sees it. A seventh row is a
/// deliberate act with this audit's question asked again — never a prefix
/// test or a name shape, which is how a closed set stops being closed. The
/// strings are yog's own spelling: the engine keeps its constants
/// crate-private, so the names cross as text exactly as they do in the
/// model's `tool_use` block.
pub const NAMES: [&str; 6] = [
    "write_summary",
    "mark_for_deletion",
    "dispatch",
    "message",
    "load_skill",
    "cd",
];

/// The `litany` verb the built-in front door answers under.
const VERB: &str = "tool";

/// The workspace root a tool's caller identity is read from, litany's spelling.
const CONV_REPO: &str = "LITANY_CONV_REPO";

/// The agent id half of the same identity, litany's spelling.
const CONV_BRANCH: &str = "LITANY_CONV_BRANCH";

/// How often a running child is looked at — a latency knob on the answer, not
/// on the act.
const POLL: Duration = Duration::from_millis(20);

/// Whether `name` is one of the engine acts this module performs.
pub fn is(name: &str) -> bool {
    NAMES.contains(&name)
}

/// Perform one engine act and answer what it captured, or the sentence saying
/// why nothing did. A failure is the in-band non-zero refusal every other
/// answer on this seam is — the model reads it and steps on.
pub fn perform(driver_target: &Path, deadline: Duration, call: &RoutedCall<'_>) -> RoutedCapture {
    match run(driver_target, deadline, call) {
        Ok(capture) => capture,
        Err(reason) => super::capture(call.name, Err(reason)),
    }
}

/// The re-entry itself: the identity on the environment, the input on stdin,
/// and the child's own three facts back untouched.
///
/// **Both waits are bounded** (litany's stated router obligations: carry your
/// own deadline, watch the stop flag). Returning early drops the stream, and
/// the drop is the kill — the same SIGTERM-then-SIGKILL cascade every other
/// child of this crate ends by.
fn run(
    driver_target: &Path,
    deadline: Duration,
    call: &RoutedCall<'_>,
) -> Result<RoutedCapture, String> {
    let mut stream = Cli::new(driver_target)
        .and_env(vec![
            (
                CONV_REPO.to_owned(),
                call.workspace.to_string_lossy().into_owned(),
            ),
            (CONV_BRANCH.to_owned(), call.agent.to_owned()),
        ])
        .run_input(
            // The caller's resolved working directory (litany bl-ddaa), which
            // is the §3.3 contract for in-process built-ins — a relative `cd`
            // resolves against where the agent stands, not against the
            // workspace root.
            Some(call.cwd),
            call.input.to_string().as_bytes(),
            &[VERB, call.name],
        )
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    loop {
        match stream.try_next() {
            StreamPoll::Ready(Chunk::Stdout(bytes)) => out.extend(bytes),
            StreamPoll::Ready(Chunk::Stderr(bytes)) => err.extend(bytes),
            StreamPoll::Ready(Chunk::Exited(info)) => {
                return Ok(RoutedCapture {
                    stdout: out,
                    stderr: err,
                    exit_code: info.shell_code(),
                });
            }
            StreamPoll::Pending => {
                if call.stop.load(Ordering::Relaxed) {
                    return Err("stopped while the engine was working".to_owned());
                }
                if started.elapsed() >= deadline {
                    return Err(format!("the engine did not answer within {deadline:?}"));
                }
                std::thread::sleep(POLL);
            }
        }
    }
}

#[cfg(test)]
mod tests;
