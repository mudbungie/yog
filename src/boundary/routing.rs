//! **The routing leg's server half** (REMOTE §3, §5, §9 step 7; bl-024b): the
//! four arms that put an invocation into a tool host's mailbox and bring its
//! capture back.
//!
//! One module for both chokepoints' arms, because they are one mechanism read
//! from four sides — two acts ([`dispatch`](super::dispatch::dispatch)'s) and
//! two reads ([`answer`](super::answer::answer)'s) over the one
//! [`Mailbox`](crate::registry::mailbox::Mailbox). Splitting them by
//! mutate/populate would put half of one hand-off in each of two files and
//! leave nowhere to state the invariant that binds them.
//!
//! **The ask never inverts** (REMOTE §3): the server speaks only when spoken
//! to, at both ends. A driver asks it to queue a call and asks again for the
//! capture; a tool host asks for its next work and posts what it got. Nothing
//! here writes down a connection, and nothing here waits on the intake that
//! runs it — [`invoke`] returns the instant the call is queued, because the
//! deposit consumer is one thread for the whole world and a tool takes as long
//! as a tool takes.
//!
//! **Adjudication is untouched and still runs first.** By the time a call
//! reaches [`invoke`] the tool control (§8.6) has already judged it in the
//! driver; this leg is transport, and REMOTE §5 is honest that what happens on
//! the far machine is beyond the adjudicator's reach.

use crate::registry::mailbox::{Call, Completion, Offsets, Standing, Verb};
use crate::registry::{Client, tools};

use super::dispatch::Deps;
use super::reply::Reply;

/// The family's one door: whichever end of the hand-off was asked for.
pub(super) fn route(deps: &Deps, ts: &str, verb: &Verb) -> Result<Reply, String> {
    match verb {
        Verb::Invoke(call) => invoke(deps, ts, call),
        Verb::Complete(done) => complete(deps, ts, done),
    }
}

/// Queue one call for the machine that advertised it, and answer the handle.
///
/// The one thing checked here is REMOTE §5's own staleness correction — *"a
/// client refuses a tool it no longer carries"* — asked where it can still be
/// answered cheaply, against what that client advertises **now**. Presence is
/// deliberately not checked: a tool host holds a connection only while it is
/// waiting, so a presence test would refuse the second call of a busy host
/// (see [`crate::registry::mailbox`]). What makes a vanished client visible is
/// the asker's own deadline.
///
/// **And a connection reaches only a machine it shares a workspace with**
/// (REMOTE §4, §5.7; bl-e0c9). A call addresses a machine and names no
/// workspace, so before this a scoped client could queue work for a box in a
/// workspace it was never registered in. The registrations of the two are both
/// files, so the test is their intersection, and a machine outside it is
/// answered in the staleness sentence — absent-shaped, as §4 requires. An
/// in-world caller is unscoped (§3) and is held to its workspace where it
/// resolves the name: the loaded set, the worktree lane and the `bz` arm each
/// read the calling workspace's own roster.
fn invoke(deps: &Deps, ts: &str, call: &Call) -> Result<Reply, String> {
    let client = Client::parse(&call.client)?;
    let caller = &deps.caller.client;
    let shared = caller.is_local()
        || !crate::registry::registered(&deps.state_root, caller)
            .is_disjoint(&crate::registry::registered(&deps.state_root, &client));
    if !shared
        || !tools::read(&deps.state_root, &client)
            .iter()
            .any(|tool| tool.name == call.tool)
    {
        return Err(format!(
            "client {:?} advertises no tool {:?} right now",
            call.client, call.tool
        ));
    }
    let invocation =
        deps.caller
            .mailbox
            .post(ts.parse().unwrap_or(0), &deps.caller.client.name(), call);
    Ok(Reply::Routed {
        invocation,
        standing: Standing::default(),
    })
}

/// A tool host posts for one invocation — a piece of its output, or the
/// capture that ends it (REMOTE §5.7). The identity that may is the intake's,
/// exactly as it is for an advertisement and for the read below — so an
/// in-world caller is refused in band with a sentence, and a handle addressed
/// to another machine is **absent** (REMOTE §4).
fn complete(deps: &Deps, ts: &str, done: &Completion) -> Result<Reply, String> {
    let client = connected(deps, "complete")?;
    let standing = deps.caller.mailbox.complete(
        ts.parse().unwrap_or(0),
        &client,
        &done.invocation,
        &done.post,
    )?;
    Ok(Reply::Routed {
        invocation: done.invocation.clone(),
        standing,
    })
}

/// **The follow-class read** (REMOTE §3): this client's next work, waited for.
/// It blocks the calling intake for the mailbox's hold — which is a connection
/// thread and never the deposit consumer, because an in-world caller is refused
/// before the wait rather than parked in it.
///
/// **One reader per identity** (REMOTE §5.1, bl-1462): a second connection
/// presenting the same certificate while one is parked here is refused in band.
/// Two readers on one queue is two processes claiming one machine's name — the
/// newcomer would take work the first is waiting for, and neither end would
/// learn it.
pub(super) fn invocations(deps: &Deps) -> Result<Reply, String> {
    let client = connected(deps, "invocations")?;
    Ok(Reply::Invocations(deps.caller.mailbox.take(&client)?))
}

/// The asker's poll: the capture if the far machine has answered, nothing yet
/// if it has not, and the absent sentence for a handle this caller did not
/// post. It never waits — the patience belongs to the caller, who is the only
/// one that knows how long its tool is worth waiting for — **and it is what
/// keeps the run alive** (REMOTE §5.7): every poll stamps the slot at `now`.
/// With `since`, the text from the asker's own offsets rides back as it
/// arrives.
pub(super) fn capture(
    deps: &Deps,
    now: i64,
    invocation: &str,
    since: Option<Offsets>,
) -> Result<Reply, String> {
    let standing =
        deps.caller
            .mailbox
            .collect(now, &deps.caller.client.name(), invocation, since)?;
    Ok(Reply::Routed {
        invocation: invocation.to_owned(),
        standing,
    })
}

/// The intake's client identity, or the in-band refusal an intake that carries
/// none earns — the [`advertise`](super::dispatch) precedent, and its reason
/// exactly: a caller who typed this at a terminal made a category error worth
/// naming, not an authentication failure worth hiding.
fn connected(deps: &Deps, verb: &str) -> Result<String, String> {
    let client = &deps.caller.client;
    if client.is_local() {
        return Err(format!(
            "{verb}: this intake carries no client identity — a tool host's work \
             is addressed to the certificate a connection presented, and an \
             in-world caller has none"
        ));
    }
    Ok(client.name())
}

#[cfg(test)]
mod tests;
