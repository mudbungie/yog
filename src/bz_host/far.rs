//! **A provider call is an invocation** (REMOTE §5.7, bl-e0c9): the second
//! executor behind [`super::run`]'s one seam — the routed leg a role takes when
//! its provider is `<row>@<client>`.
//!
//! litany spawns the adapter it was handed as `<adapter> --json --provider
//! <name>` and never learns where the call ran. The name is read **here**, in
//! the `bz` arm, against the clients registered in the calling workspace: a
//! bare row — or a name whose suffix is no machine registered *here* — is the
//! linked brazen's, exactly as before ([`of`] answers `None` and the caller
//! takes the path it always took). A qualified one is posted as an `invoke`
//! of that machine's advertised `bz` tool, whose input **is** the command
//! line: `{"args": […], "stdin": <the request>}`, the row unqualified.
//!
//! **It is the routing leg, asked by one more resident of the world**
//! ([`crate::tool_host::remote`]): the same two gestures through the same
//! `gestures/` inbox, because the arm runs inside the driver too. What differs
//! is the read — every poll states `since`, so the far child's output is
//! answered while it is being produced and written to this process's own
//! stdout in the pieces it arrived in. litany appends that to `response.json`
//! as it does for a local call, and nothing downstream can tell.
//!
//! **Two failures, and which one is decided by whether a byte was relayed**
//! (§5.7, *Nothing is retried on the wire*):
//!
//! - **Nothing relayed** — the machine never took the call inside the
//!   router's own patience, or the server refused the ask. The arm answers one
//!   canonical `error` event naming the client and when it last spoke, then the
//!   terminator. The kind is not a retryable one: a retry would queue a second
//!   call behind the first. There is no fallback to the server's own row,
//!   because a call that silently ran on another credential is the implicit
//!   location the name exists to forbid.
//! - **Something relayed** — the stream ends where it stands, the reason goes
//!   to stderr, and the exit is non-zero. litany reads a stream without its
//!   terminator as a call that died and does not retry it.
//!
//! **A run the server ended in doubt is not a third case.** It arrives as a
//! capture like any other — the text that had been posted, the server's
//! sentence on stderr, exit code 1 — and is relayed as one.
//!
//! **Patience is spent only while nobody has taken the call**, and it is a
//! span of the clock. `partial`'s
//! presence on a reply is the lease fact (§5.7); once a machine has posted, the
//! call is worth as long as it runs, and what ends a run that stalls is the
//! server's own thirty-second silence rule, read at this arm's poll.
//!
//! **A stop needs no code here.** The arm is a child in the driver's process
//! group; §3.1's stop signals the group, this process dies under the default
//! disposition, its polls end, and the far run is refused its next post.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use serde_json::json;

use crate::registry::mailbox::stream::offsets_value;
use crate::registry::mailbox::{Offsets, Standing};
use crate::registry::{Client, roster, seen};
use crate::tool_host::ask::Budget;
use crate::tool_host::remote;
use crate::xdg::Env;

/// The advertised tool a box serves model calls through (§5.7).
pub(crate) const TOOL: &str = "bz";
/// The flag whose value names the row — the one spelling litany sends.
const PROVIDER: &str = "--provider";
/// The canonical `kind` of a call nothing was relayed for. brazen's taxonomy
/// passes an unknown tag through verbatim, and no unknown tag is retryable.
const KIND: &str = "far_call";

/// Read `<row>@<client>` off a provider name, against the clients registered
/// in the calling workspace. The longest registered name that fits wins; a
/// name with no row in front of the joiner is not a qualification.
pub(crate) fn split(provider: &str, clients: &[String]) -> Option<(String, String)> {
    clients
        .iter()
        .filter_map(|client| {
            let row = provider.strip_suffix(client.as_str())?.strip_suffix('@')?;
            (!row.is_empty()).then(|| (row.to_owned(), client.clone()))
        })
        .max_by_key(|(_, client)| client.len())
}

/// Every client registered in `workspace`, by name — the set a provider's
/// name is read against, by the arm and by the server's own reads alike.
pub(crate) fn machines(state_root: &Path, workspace: &str) -> Vec<String> {
    roster::members(state_root, workspace)
        .iter()
        .map(Client::name)
        .collect()
}

/// One call bound for another machine: a turn's model call, a model listing
/// or a sign-in — every command line that names a row (§5.7, bl-7467).
pub(super) struct Far {
    /// Where the `gestures/` inbox and the registry live.
    pub(super) state_root: PathBuf,
    /// The registered machine the name selected.
    pub(super) client: String,
    /// The command line as that machine's `bz` takes it — the row unqualified.
    pub(super) argv: Vec<String>,
    /// How long the server may take to answer one ask.
    pub(super) budget: Budget,
    /// How long the call may stand untaken — the router's own bound.
    pub(super) patience: Budget,
    /// Whether stdout is brazen's canonical stream — the data plane's. Only
    /// there is a call nothing was relayed for answered as an `error` event;
    /// a listing or a sign-in says the same sentence on stderr.
    pub(super) events: bool,
}

/// The far executor for `argv`, or `None` where the linked brazen answers: a
/// route that names no row (the provider listing, token counting, the
/// server), no `--provider`, no workspace wall, or a name no client
/// registered in this workspace fits.
pub(super) fn of(argv: &[String], env: &Env) -> Option<Far> {
    use brazen::Route::{ListModels, Login, Run};
    let route = brazen::route(argv);
    if !matches!(route, Run | ListModels | Login) {
        return None;
    }
    let at = argv.iter().position(|arg| arg == PROVIDER)? + 1;
    let workspace = crate::naming::leaf(&env.wall()?);
    let state_root = env.yog_state_root();
    let (row, client) = split(argv.get(at)?, &machines(&state_root, &workspace))?;
    let argv = argv
        .iter()
        .enumerate()
        .map(|(i, arg)| if i == at { row.clone() } else { arg.clone() })
        .collect();
    Some(Far {
        state_root,
        client,
        argv,
        budget: Budget::default(),
        patience: remote::patience(),
        events: matches!(route, Run),
    })
}

/// Write one appended piece through, flushed — a piece is on the pipe before
/// the next poll is asked — and count it into `held`, which is both how much
/// has been relayed and the `since` the next poll states.
fn pass(
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    held: &mut Offsets,
    piece: (&str, &str),
) -> Result<(), String> {
    held.stdout += piece.0.len() as u64;
    held.stderr += piece.1.len() as u64;
    stdout
        .write_all(piece.0.as_bytes())
        .and_then(|()| stdout.flush())
        .and_then(|()| stderr.write_all(piece.1.as_bytes()))
        .map_err(|e| format!("relaying the stream: {e}"))
}

impl Far {
    /// Make the call: a command line and a standard input in, the three
    /// streams out — the far child's, in the pieces they arrived in.
    pub(super) fn run(
        &self,
        stdin: &mut dyn Read,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> i32 {
        let mut held = Offsets::default();
        match self.call(stdin, stdout, stderr, &mut held) {
            Ok(code) => code,
            Err(reason) if held.stdout > 0 || !self.events => {
                let _ = writeln!(stderr, "yog bz: {reason}");
                1
            }
            Err(reason) => refuse(stdout, reason),
        }
    }

    fn call(
        &self,
        stdin: &mut dyn Read,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
        held: &mut Offsets,
    ) -> Result<i32, String> {
        let mut request = String::new();
        stdin
            .read_to_string(&mut request)
            .map_err(|e| format!("reading the request: {e}"))?;
        // Never set: a stop reaches this process as a signal (module doc).
        let stop = AtomicBool::new(false);
        let ask = |gesture: serde_json::Value| -> Result<(String, Standing), String> {
            remote::routed(&self.state_root, self.budget, &gesture, &self.client, &stop)
        };
        let (invocation, _) = ask(json!({
            "op": "invoke", "client": self.client, "tool": TOOL,
            "input": { "args": self.argv, "stdin": request },
        }))?;
        // The patience is a span of the clock, not a count of polls: a poll
        // is a deposit round trip on top of its tick, and two hundred and
        // forty of them measured three minutes where the sentence said two.
        // **Nor is it spent while the machine is there** (REMOTE §3.3,
        // bl-d9ad): one that spoke a moment ago is working, this call is
        // queued behind its job, and busy means wait.
        let (mut taken, mut asked) = (false, std::time::Instant::now());
        loop {
            let (_, standing) = ask(json!({
                "op": "capture", "invocation": invocation,
                "since": offsets_value(*held),
            }))?;
            if let Some(piece) = standing.partial {
                taken = true;
                pass(stdout, stderr, held, (&piece.stdout, &piece.stderr))?;
            }
            if let Some(capture) = standing.capture {
                pass(stdout, stderr, held, (&capture.stdout, &capture.stderr))?;
                return Ok(capture.exit_code);
            }
            if !taken && seen::recent(&self.state_root, &self.client) {
                asked = std::time::Instant::now();
            } else if !taken && asked.elapsed() >= self.patience.span() {
                return Err(self.untaken());
            }
            std::thread::sleep(self.patience.tick);
        }
    }

    /// The sentence a call nobody took earns: who, for how long, and when that
    /// machine last spoke (§5.1's `last_seen`, read off the same file).
    fn untaken(&self) -> String {
        let last = Client::parse(&self.client)
            .ok()
            .and_then(|client| seen::read(&self.state_root, &client))
            .map_or_else(
                || "it has never connected".to_owned(),
                |at| {
                    format!(
                        "it last connected at {}",
                        crate::ui_state::iso8601_extended(at)
                    )
                },
            );
        format!(
            "client {:?} did not take this call within {} seconds of last speaking; \
             {last}. It is asleep, or gone. Nothing ran, and nothing was sent to this \
             server's own provider instead",
            self.client,
            self.patience.span().as_secs()
        )
    }
}

/// Answer a call nothing was relayed for: one canonical `error` event, then
/// the terminator, and the exit code brazen's own table gives the kind.
fn refuse(stdout: &mut dyn Write, message: String) -> i32 {
    let error = brazen::CanonicalError {
        kind: brazen::ErrorKind::Other(KIND.to_owned()),
        message,
        provider_detail: None,
        retry_after_seconds: None,
    };
    let code = i32::from(error.exit_code());
    for event in [brazen::Event::Error(error), brazen::Event::End] {
        let line = serde_json::to_string(&event).unwrap_or_default();
        let _ = writeln!(stdout, "{line}");
    }
    code
}

#[cfg(test)]
mod tests;
