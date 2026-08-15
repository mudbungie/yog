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
/// when it differs from what is stored (REMOTE §5).
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
    tools::store(&deps.state_root, client, tools).map_err(|e| e.to_string())?;
    Ok(Reply::Advertised)
}

#[cfg(test)]
mod tests;
