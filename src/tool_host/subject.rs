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
//! **Every miss is an in-band refusal that names the way out.** No consenting
//! advertiser: the sentence says what the operator enrolls and what the model
//! can do instead (load a host-bound instance, which runs in that box's own
//! directory). More than one: a config ambiguity, refused naming every
//! claimant — one adjudication must stand for exactly one execution on one
//! machine (REMOTE §5, no broadcast). The engine unreachable: the transport
//! sentence every other ask already renders.
//!
//! **Front-door-only and ship-inert hold.** The lane is the same adjudicate →
//! mailbox → execute → capture pipeline every call takes; the server still
//! executes nothing, and a workspace with no consenting thrall refuses in
//! band, which is the posture working.

use ::litany::cmd::{RoutedCall, RoutedCapture};

use super::{Site, capture, loaded, remote};
use crate::registry::roster::ClientRow;

/// Answer one workspace-subject attempt: route it to the workspace's one
/// consenting advertiser with the conversation's cwd on the invocation, or
/// render the refusal that names the way out.
pub fn answer(site: &Site, call: &RoutedCall<'_>) -> RoutedCapture {
    let roster = match super::ask::roster(&site.state_root, &site.workspace, site.budget, call.stop)
    {
        Ok(rows) => rows,
        Err(reason) => return capture(call.name, Err(reason)),
    };
    match verdict(&roster, call.name) {
        Ok(entry) => routed(site, &entry, call),
        Err(refusal) => capture(call.name, Err(refusal)),
    }
}

/// The lane's selection: exactly one consenting advertiser is an answer, and
/// everything else is a sentence. Pure over the roster, so every arm is
/// reachable without an engine.
fn verdict(roster: &[ClientRow], name: &str) -> Result<loaded::Entry, String> {
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
        1 => Ok(consenting.remove(0)),
        0 => Err(unconsented(name, &advertisers)),
        _ => Err(format!(
            "{} machines consent to run {name} in this conversation's working \
             directory ({}), and one execution needs one machine: the operator \
             must leave \"subject_cwd\": true on exactly one entry",
            consenting.len(),
            names(
                &consenting
                    .iter()
                    .map(|e| e.client.clone())
                    .collect::<Vec<_>>()
            ),
        )),
    }
}

/// The zero-consent refusal, in the two shapes it honestly has: machines
/// advertise the name but none consents, or nothing advertises it at all.
/// Both name the remedy, because the reader is a model and the fixer is an
/// operator.
fn unconsented(name: &str, advertisers: &[String]) -> String {
    if advertisers.is_empty() {
        return format!(
            "no tool of that name is loaded in this conversation and no machine \
             of this workspace advertises {name}; use the clients tool to see \
             this workspace's machines and load what one advertises — or the \
             operator enrolls a thrall on the box that holds this server's \
             worktrees, with \"subject_cwd\": true on a {name} entry in its \
             tools.json"
        );
    }
    format!(
        "{} advertises {name}, but no machine of this workspace consents to run \
         it in this conversation's working directory; load it with the clients \
         tool to run it in that machine's own directory, or the operator adds \
         \"subject_cwd\": true to the {name} entry in tools.json on the box \
         that holds this server's worktrees",
        names(advertisers),
    )
}

/// A comma-joined machine list for a sentence.
fn names(clients: &[String]) -> String {
    clients.join(", ")
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

#[cfg(test)]
mod tests;
