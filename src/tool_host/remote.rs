//! **The driver's end of the routing leg** (REMOTE §3, §5, §9 step 7;
//! bl-024b): what a loaded remote name actually does when the model calls it.
//!
//! The injection runs inside the **driver**, a child process (see
//! [`super::ask`]), so this is the same deposit round trip the roster read is —
//! `Query`/`Action` envelopes into `<yog-state-root>/gestures/`, replies read
//! back with the one reply codec. No verb and no transport were added for the
//! driver's side either: `invoke` and `capture` are boundary gestures every
//! face gains.
//!
//! **Two gestures rather than one, and that is the whole shape.** The engine's
//! intake is one thread for the world, so a gesture that waited for a tool to
//! finish would stop every other deposit converging for as long as the tool
//! ran. `invoke` therefore queues and answers immediately, and the *waiting* is
//! here, in the child that has nothing else to do — which is also the only
//! process that knows how long this call is worth waiting for.
//!
//! **The deadline is the visible refusal** (REMOTE §5: *"a vanished client is a
//! visible refusal, not a hang"*). Nothing anywhere asks whether the far
//! machine is connected: a tool host holds its connection only while it is
//! waiting, so it is *absent* for the whole time it is busy, and a presence
//! test would refuse the second call of a host that is certainly there. What a
//! machine that never answers earns is this loop running out, in band, naming
//! how long it waited.

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};

use super::ask::{self, Budget};
use super::{Site, loaded};
use crate::boundary::reply::{self, Reply};
use crate::registry::mailbox::Capture;

/// Run one loaded remote name on the machine that advertised it, and answer
/// what it captured — or the sentence saying why nothing did.
pub fn invoke(
    site: &Site,
    entry: &loaded::Entry,
    input: &Value,
    stop: &AtomicBool,
) -> Result<Capture, String> {
    let invocation = routed(
        site,
        &json!({ "op": "invoke", "client": entry.client,
                 "tool": entry.tool.name, "input": input }),
        stop,
    )?
    .0;
    let poll = json!({ "op": "capture", "invocation": invocation });
    for _ in 0..site.patience.waits {
        if let (_, Some(capture)) = routed(site, &poll, stop)? {
            return Ok(capture);
        }
        if stop.load(Ordering::Relaxed) {
            return Err(format!("stopped while {} was running it", entry.client));
        }
        std::thread::sleep(site.patience.tick);
    }
    Err(format!(
        "client {:?} did not answer invocation {invocation} in time; \
         it may be offline, or the tool is still running there",
        entry.client
    ))
}

/// One `Reply::Routed` round trip: the handle, and the capture if there is one
/// yet. A refusal envelope and an envelope of another kind are the same class
/// of answer — the engine did not say what was asked — and both name what came
/// back, exactly as [`ask::roster`] does.
fn routed(
    site: &Site,
    request: &Value,
    stop: &AtomicBool,
) -> Result<(String, Option<Capture>), String> {
    let envelope = ask::ask(&site.state_root, request, site.budget, stop)?;
    match reply::decode(&envelope) {
        Ok(Ok(Reply::Routed {
            invocation,
            capture,
        })) => Ok((invocation, capture)),
        Ok(Ok(other)) => Err(format!(
            "engine answered {other:?}, not a routed invocation"
        )),
        Ok(Err(refusal)) => Err(refusal),
        Err(e) => Err(format!("undecodable engine reply: {e}")),
    }
}

/// How long a driver waits on a *tool* — as against on the engine
/// ([`Budget::default`], which bounds one deposit round trip). Two bounds
/// because they measure two different things: an engine that has not answered
/// in ten seconds is down, and a tool that has not answered in ten seconds is
/// working.
pub fn patience() -> Budget {
    Budget {
        waits: 240,
        tick: std::time::Duration::from_millis(500),
    }
}

#[cfg(test)]
mod tests;
