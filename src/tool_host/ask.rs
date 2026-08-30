//! **The driver's ask** (REMOTE §3, §5; bl-c907): how a `yog litany` process
//! puts a question to the engine that spawned it.
//!
//! The injection runs inside the **driver**, a child process yog spawned
//! (`src/multiplex/litany.rs`). The facts a `clients` op needs are not all on
//! disk — REMOTE §5 makes *presence* connection-scoped RAM in the engine, on
//! purpose — so the driver cannot read the roster itself. It asks, through the
//! door REMOTE §3 already reserves for the world's own residents:
//!
//! > *"The disk inbox survives for in-world callers. Agents drive yog through
//! > the `yog` PATH shim and the `gestures/` deposit inbox — same machine, same
//! > world, disk is the bus."*
//!
//! So this adds **no verb and no transport**. It is `Query::Clients` — the
//! roster bl-4e08 landed — deposited create-only into `<yog-state-root>/
//! gestures/` and read back out of `replies/`, exactly as `yog gesture` does
//! ([`crate::boundary::sugar`]); the reply is decoded with the one reply codec.
//! The driver resolves the same state root the engine writes because the world
//! hands `XDG_STATE_HOME` down to every child (§16.2), so both fold to
//! `<world>/state/yog`.
//!
//! **Every wait is bounded, and the bound is the router's own** (litany
//! `docs/DESIGN_TOOL_INJECTION.md` §3.3: *"Carry your own deadline … A vanished
//! endpoint is an in-band error result, never a hang … Watch
//! `RoutedCall::stop`"*). A [`Budget`] is a poll count and a tick, and a stop
//! landing mid-wait ends the wait — an engine that never answers is an error
//! string the model reads, never a wedged drive.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{Value, json};

use crate::boundary::deposit;
use crate::boundary::reply::{self, Reply};
use crate::registry::roster::ClientRow;

/// How long the driver waits on the engine: `waits` looks, `tick` apart. A
/// latency knob and a deadline in one — the product is the wall-clock bound,
/// and both halves are injected so a test never sleeps for real.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// How many times to look for the reply before giving up.
    pub waits: u32,
    /// How long to sleep between looks.
    pub tick: Duration,
}

impl Default for Budget {
    /// The production bound: ten seconds, looked at eight times a second. A
    /// roster read is three local file reads behind a 250 ms consumer poll, so
    /// this is two orders of magnitude of headroom — and a bound all the same,
    /// because the engine may be down.
    fn default() -> Self {
        Self {
            waits: 80,
            tick: Duration::from_millis(125),
        }
    }
}

/// The gesture-id seed every driver ask is minted from. Legibility only —
/// uniqueness is the filesystem's ([`deposit::mint`], bl-aa9f).
const SEED: &str = "toolhost";

/// Deposit `request` and wait for its reply envelope, or say why there is
/// none. The four failure sentences are all in-band results: a driver that
/// cannot reach its engine renders an error the model steps on.
pub fn ask(
    state_root: &Path,
    request: &Value,
    budget: Budget,
    stop: &AtomicBool,
) -> Result<Value, String> {
    let id = deposit::mint(state_root, SEED).map_err(|e| format!("gesture id: {e}"))?;
    deposit::deposit(state_root, &id, request).map_err(|e| format!("deposit: {e}"))?;
    for _ in 0..budget.waits {
        if let Some(envelope) = deposit::read_reply(state_root, &id) {
            return Ok(envelope);
        }
        if stop.load(Ordering::Relaxed) {
            return Err("stopped while waiting for the engine".to_owned());
        }
        std::thread::sleep(budget.tick);
    }
    Err("no engine answered; is yog running on this world?".to_owned())
}

/// The workspace's client roster (REMOTE §5.1's `Query::Clients`): registered
/// clients, which are live this instant, and what each advertises — joined by
/// the engine at the moment it is asked, so nothing here is cached and a
/// presence flap needs no invalidation.
///
/// A refusal envelope and an envelope of the wrong kind are the same kind of
/// answer — the engine did not give a roster — and both name what came back.
pub fn roster(
    state_root: &Path,
    workspace: &str,
    budget: Budget,
    stop: &AtomicBool,
) -> Result<Vec<ClientRow>, String> {
    let envelope = ask(
        state_root,
        &json!({ "op": "clients", "workspace": workspace }),
        budget,
        stop,
    )?;
    match reply::decode(&envelope) {
        Ok(Ok(Reply::Clients(rows))) => Ok(rows),
        Ok(Ok(other)) => Err(format!("engine answered {other:?}, not a client roster")),
        Ok(Err(refusal)) => Err(refusal),
        Err(e) => Err(format!("undecodable engine reply: {e}")),
    }
}

#[cfg(test)]
mod tests;
