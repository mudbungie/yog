//! **A tool host presents its set** (REMOTE §5, bl-4e08) — the executor behind
//! [`Action::Advertise`](crate::boundary::Action::Advertise), beside
//! [`delete_exec`](super::delete_exec) and for its reason: everything else in
//! the chokepoint routes, and these *gate*.
//!
//! The gate here is **who is asking**, not what was named. Every other action
//! is authorized by the workspace it addresses (REMOTE §4's one filter); this
//! one addresses no workspace at all, because a tool set is a fact about a
//! machine and the registration listing already says which workspaces see it.
//! So its authorization is the identity the intake carries — the connection's
//! certificate common name, read exactly where scoping reads it — and an intake
//! that carries none refuses **in band**, with a sentence, rather than being
//! silently dropped: an operator typing this at a terminal has made a category
//! error worth naming, not committed an authentication failure.

use crate::registry::tools::{self, Tool};

use super::Deps;
use crate::boundary::reply::Reply;

/// Validate the set and store it under the caller's own identity, writing only
/// when it differs from what is stored (REMOTE §5) — and **answer whether it
/// wrote** (REMOTE §5.1, bl-66d4).
///
/// [`tools::store`] has always computed that bool and this executor always
/// discarded it, so a box re-presenting an unchanged set and a box restoring a
/// set some other connection blanked were answered the identical `ok`. The
/// second of those is two processes claiming one machine's name, and it reached
/// no log on either side: bl-1462's guards cover the whole of an IDLE host's
/// life, and the window they cannot cover is the one the host opens itself —
/// §5.3's foot is absent for a tool's whole runtime, holds no parked read, and
/// so `superseding` below waves the replacement through. The foot's own half
/// (thrall bl-2d78) re-asserts at the end of every hand-off, which bounds that
/// window to one tool's runtime and then **heals silently**. This is the
/// sentence that makes the heal audible, and it costs one field on one reply.
pub(super) fn advertise(deps: &Deps, tools: &[Tool]) -> Result<Reply, String> {
    let client = &deps.caller.client;
    if client.is_local() {
        return Err(
            "advertise: this intake carries no client identity — a tool set is presented \
             by a connection, and the certificate is what says whose set it is"
                .to_owned(),
        );
    }
    tools::validate(tools)?;
    superseding(deps, tools)?;
    let wrote = tools::store(&deps.state_root, client, tools).map_err(|e| e.to_string())?;
    Ok(Reply::Advertised { wrote })
}

/// **A serving machine's set may not be replaced under it** (REMOTE §5.1,
/// bl-1462). The store is keyed on the identity and was last-writer-wins, so
/// any connection bearing the certificate could blank a healthy host's tools —
/// and by REMOTE §5's own traffic ruling the set is presented once per channel,
/// so the host that is running never learns it was disarmed. The only symptom
/// was invocations refused for a tool that plainly exists.
///
/// The seam is drawn at the one moment the engine can tell the two apart: a
/// **parked follow-class read** is a machine that is serving right now, and a
/// set that **differs** from the one in force is a second party disagreeing
/// with it. A host re-presenting an unchanged set on reconnect writes nothing
/// and never reaches this — which is the ordinary path, so the guard costs it
/// no refusal and no file read.
fn superseding(deps: &Deps, tools: &[Tool]) -> Result<(), String> {
    let client = &deps.caller.client;
    if !deps.caller.mailbox.serving(&client.name())
        || tools::read(&deps.state_root, client) == tools
    {
        return Ok(());
    }
    Err(format!(
        "advertise: {:?} is holding this engine's follow-class read and this is not the \
         set in force — a second connection may not replace a serving machine's tools, \
         because the machine that is serving would never learn it was disarmed. \
         Re-present the set in force, or stop the connection that is serving",
        client.name()
    ))
}

#[cfg(test)]
mod tests;
