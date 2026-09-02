//! **The worktree lane** (REMOTE §5.4 as amended by bl-77be): what a granted,
//! unqualified name does when the model calls it.
//!
//! A name that is not the `clients` tool, not an engine act and not a loaded
//! host-qualified instance is a **workspace-subject attempt** — `bash`,
//! `read_file`, `apply_patch`, or any pool tool an operator granted. Its
//! subject is the conversation's working tree, the worktree lives on the
//! server's box, and the subject-locality invariant (REMOTE §5: *"a tool
//! executes where its subject lives"*) therefore already chose the executing
//! box — which is why a bare name is not a call with an implicit location,
//! and why nothing here contradicts §5's locality-rides-in-the-name rule for
//! *loaded* instances.
//!
//! **Consent picks the machine, and the operator authored it.** The lane
//! routes to the ONE client registered in this workspace that both advertises
//! the name and consents to workspace-cwd execution (`"subject_cwd": true` on
//! that entry in its own tools.json — REMOTE §5.2's document, the severable
//! gate). The invocation carries the conversation's resolved working
//! directory ([`RoutedCall::cwd`], litany's own mark-or-worktree resolution),
//! which is REMOTE §5's *"an invocation carries its subject's location"*
//! landed. The consenting box is normally §5.4's co-located thrall — the
//! normal install, because it actually holds the server's worktrees.
//!
//! **When no machine consents, the engine performs its own built-in**
//! ([`PERFORMED`], REMOTE §5.4 as amended by bl-5710, operator ruling
//! 2026-08-31: *ship some basic tools — a default install must be able to
//! write a file*). The lane is a ladder, ordered by how explicit the
//! operator's intent is: a consenting machine is an enrollment plus a
//! `subject_cwd` key, so it wins; with no consenting machine the executing
//! box is the one subject-locality already named — the server's own, which
//! holds the worktree — and the act is performed at the engine's own front
//! door ([`super::engine_act::perform`]), the same `<driver_target> tool
//! <name>` re-entry the compactor pair takes, at the conversation's resolved
//! cwd with the caller identity on the child's environment. yog restates none
//! of what the three names *do*: the definitions are the engine's built-ins
//! and the front door is the one way in.
//!
//! **Every remaining miss is an in-band refusal that names the way out — and
//! only a way out that is one** (bl-68e1). A name the engine does not
//! implement and no machine consents to: the sentence names the operator's
//! config edit, the only act that puts the work where its subject is, and
//! says outright that loading a machine's tool is not that act, because a
//! loaded invocation carries no directory and lands in the far process's
//! inherited one (`refusal::NOT_A_REMEDY`, which carries that drive's
//! evidence). More than one consenting machine: a config ambiguity, refused
//! naming every claimant — one adjudication must stand for exactly one
//! execution on one machine (REMOTE §5, no broadcast). The engine
//! unreachable: the transport sentence every other ask already renders.
//!
//! **The sentences live one level down** (`subject/refusal.rs`): [`verdict`]
//! decides which rung a name lands on, and that file decides what a landing
//! on a refusing rung says.
//!
//! **Front-door-only holds; ship-inert now stops short of the worktree.** The
//! lane is still the same adjudicate → mailbox → execute → capture pipeline
//! for a routed call, and the server still executes nothing **in its own
//! process** — [`PERFORMED`] crosses the engine's front door as a child, which
//! is what §12's invariant asks. What changed is the posture's reach: a
//! server that refuses every act on its own worktrees is not inert, it is
//! dead, and the operator ruled that a default install writes a file.

use std::path::Path;
use std::time::Duration;

use ::litany::cmd::{RoutedCall, RoutedCapture};

use super::{Site, capture, loaded, remote};
use crate::registry::roster::ClientRow;

/// **The lane's last rung, closed and enumerated here and nowhere else**
/// (bl-5710): the worktree names the engine itself implements, performed at
/// its own front door when no machine consents to run them.
///
/// Three rows, and each is admitted by the same two facts together: its
/// subject is the conversation's working tree, which lives on the server's
/// box; and `litany tool <name>` is an implementation the engine already
/// ships, so performing it restates nothing. A pool name an operator granted
/// is on neither footing — the engine has no implementation to reach — so it
/// keeps the refusal that names the enrollment. A fourth row is a deliberate
/// act with both questions asked again, never a prefix test or a name shape.
/// The strings are yog's own spelling: the engine keeps its built-in
/// constants crate-private, so the names cross as text exactly as they do in
/// the model's `tool_use` block.
pub const PERFORMED: [&str; 3] = ["apply_patch", "bash", "read_file"];

/// Where one workspace-subject attempt is executed, once the roster has been
/// read. Three outcomes because the lane has three: a machine the operator
/// enrolled and consented, the engine's own front door, or a sentence.
enum Lane {
    /// Route to this machine, with the conversation's cwd on the invocation.
    Machine(Box<loaded::Entry>),
    /// Perform it here, at the engine's front door ([`PERFORMED`]).
    Engine,
    /// Nothing can run it; this is what the model reads instead.
    Refused(String),
}

/// Answer one workspace-subject attempt: route it to the workspace's one
/// consenting advertiser with the conversation's cwd on the invocation,
/// perform it at the engine's own front door when the engine implements it
/// and no machine consents, or render the refusal that names the way out.
pub fn answer(
    driver_target: &Path,
    deadline: Duration,
    site: &Site,
    call: &RoutedCall<'_>,
) -> RoutedCapture {
    let roster = match super::ask::roster(&site.state_root, &site.workspace, site.budget, call.stop)
    {
        Ok(rows) => rows,
        Err(reason) => return capture(call.name, Err(reason)),
    };
    match verdict(&roster, call.name) {
        Lane::Machine(entry) => routed(site, &entry, call),
        Lane::Engine => super::engine_act::perform(driver_target, deadline, call),
        Lane::Refused(refusal) => capture(call.name, Err(refusal)),
    }
}

/// The lane's selection, pure over the roster so every rung is reachable
/// without an engine: exactly one consenting advertiser is that machine, no
/// consenting advertiser is the engine's front door for a name it implements
/// and a sentence for one it does not, and more than one is an ambiguity.
fn verdict(roster: &[ClientRow], name: &str) -> Lane {
    let mut consenting: Vec<loaded::Entry> = Vec::new();
    let mut advertisers: Vec<String> = Vec::new();
    for row in roster {
        for tool in &row.tools {
            if tool.name != name {
                continue;
            }
            advertisers.push(row.client.clone());
            if tool.subject_cwd {
                consenting.push(loaded::Entry {
                    client: row.client.clone(),
                    tool: tool.clone(),
                });
            }
        }
    }
    match consenting.len() {
        1 => Lane::Machine(Box::new(consenting.remove(0))),
        0 if PERFORMED.contains(&name) => Lane::Engine,
        0 => Lane::Refused(refusal::unconsented(name, &advertisers)),
        _ => Lane::Refused(refusal::ambiguous(
            name,
            &consenting
                .iter()
                .map(|e| e.client.clone())
                .collect::<Vec<_>>(),
        )),
    }
}

/// The routing leg with the subject's location on the invocation — the far
/// machine's own three facts back untouched, exactly as the loaded lane
/// passes them, so the model cannot tell a routed tool from a local one.
fn routed(site: &Site, entry: &loaded::Entry, call: &RoutedCall<'_>) -> RoutedCapture {
    let cwd = call.cwd.to_string_lossy().into_owned();
    match remote::invoke(site, entry, call.input, Some(&cwd), call.stop) {
        Ok(got) => RoutedCapture {
            stdout: got.stdout.into_bytes(),
            stderr: got.stderr.into_bytes(),
            exit_code: got.exit_code,
        },
        Err(reason) => capture(call.name, Err(reason)),
    }
}

/// The lane's sentences, which decide what a refusing rung says.
mod refusal;

#[cfg(test)]
mod tests;
