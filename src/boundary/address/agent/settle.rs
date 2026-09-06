//! **A conversation is not addressable the instant its name is minted, and a
//! refusal that arrives inside that window is a race rather than an answer**
//! (bl-802a) — so the disk rung of [`resolve_agent`](super::resolve_agent)
//! *holds* before it says the name means nothing.
//!
//! The start flow mints the §3.3 conversation name and hands it back the moment
//! the **detached** `litany prompt` has been launched (§8.1, §13.3): the driver
//! writes `agents/<id>` and its `name` blob from its own process, a second or
//! two later. Between those two moments the engine had already answered its own
//! receipt's name with *"unknown conversation"* — the identical sentence a name
//! that never existed earns — so `start` then `follow`, the first pair every
//! operator types and the whole of a two-seat handoff, failed every time it was
//! typed. Measured at 2/2 on consecutive starts, self-healing within one to two
//! seconds.
//!
//! **The reframe is that "unknown" was never instantly knowable.** A name is
//! minted, then materialized; a read that lands in that gap has not learned
//! anything yet. So this is not a wait bolted onto the start's receipt — which
//! would make the property true only for a client holding one — but the disk
//! rung declining to conclude before the conversation could possibly have
//! appeared. Every client gets it: the seat that was handed the name, a script
//! that types it, a second seat told to go look.
//!
//! **What it costs, and to whom.** A name that resolves — every steady-state
//! read — pays one look and nothing else, because rungs one and two answer
//! first and this one answers on its first try. Only a name with **no** answer
//! pays the hold, and that is exactly the case that cannot yet be sure. The
//! bound is the driver's launch, not a network's: [`SETTLE_LOOKS`] retries
//! [`SETTLE_TICK`] apart, three seconds in total, after which the refusal is
//! the one it always was. A genuinely wrong name therefore refuses in three
//! seconds instead of instantly — the deliberate trade, on the consumer that
//! can afford it (a boundary pass may be inside a `bl close`, §8.5).
//!
//! The look is not free — [`living_agents`](crate::git_tree::living_agents) is
//! one `for-each-ref` plus one `git show` per ref — so the window is spent in
//! **few, coarse** looks rather than many fine ones. Two knobs, the
//! [`Follow`](crate::boundary::follow::Follow) hold's own shape, and a test
//! names a short pair rather than sleeping for real.

use std::path::Path;
use std::time::Duration;

/// How many times the disk rung looks again before it refuses, after the first
/// look that found nothing. Six looks [`SETTLE_TICK`] apart is three seconds —
/// past the one-to-two the sighting measured, with room for a loaded box.
pub(super) const SETTLE_LOOKS: u32 = 6;
/// How long between two looks. Coarse on purpose: each look forks `git` once
/// per living agent, so the window is worth more spent in a few looks than in
/// many.
pub(super) const SETTLE_TICK: Duration = Duration::from_millis(500);

/// The **held** disk rung: what `needle` addresses among `workspace`'s living
/// agents, looking again `looks` times `tick` apart before answering `None`.
///
/// `Ok(None)` is the refusal's own evidence — the name did not appear in the
/// window — and `Err` (a name two living agents wear) ends the hold at once,
/// because a second look cannot make an ambiguity into an answer.
pub(super) fn on_disk(
    workspace: &Path,
    needle: &str,
    looks: u32,
    tick: Duration,
) -> Result<Option<String>, String> {
    let mut left = looks;
    loop {
        if let Some(id) = super::within(&crate::git_tree::living_agents(workspace), needle)? {
            return Ok(Some(id));
        }
        if left == 0 {
            return Ok(None);
        }
        left -= 1;
        std::thread::sleep(tick);
    }
}
