//! **One world has one engine** (DESIGN §8.5, §16.2; bl-1d9b) — the exclusion,
//! taken before the engine consumes anything and released when it drops.
//!
//! A second `yog` booted on a world the first still holds used to print its
//! bind refusal and **carry on**. It had no listener, so no seat could reach
//! it, but it still drained the world's `gestures/` inbox and still held its
//! own [`Slots`](crate::registry::mailbox) — its own `seq`, its own `live` map,
//! its own presence map. Two consumers on one inbox is two mailboxes minting
//! into one `inv-N` namespace: a routed tool call answered `no invocation
//! "inv-5" is in flight` when the handle missed, and — the reason this is a p1
//! — handed back **another invocation's capture** when it collided. Presence
//! forked the same way: one engine answered "not connected right now" while the
//! other, one second later, answered "connected right now", and both sentences
//! reached one model.
//!
//! **Why a lock and not the bind.** The bind is the natural exclusion and it
//! cannot be this one: a self-provisioned box's `wire/address` says
//! `127.0.0.1:0` (REMOTE §8, bl-dc14 — so two engines in two *worlds* never
//! contend for a process-global port), and two `:0` binds both succeed on
//! different kernel-chosen ports. Recording a concrete port at mint time would
//! make the bind exclusive again and would put a listener in the ephemeral
//! range, where a boot that finds its own port taken by some outbound
//! connection is an engine that will not start for a reason no operator can
//! see. The lock states the invariant directly, at the one place it means
//! something — the world — and is independent of whether a wire exists at all.
//!
//! **Why an advisory file lock and not a pid file.** The lock is held by an
//! open file description, so the kernel releases it when this process ends **by
//! any means** — a clean drop, a `SIGKILL`, an OOM. There is no stale record to
//! reap, no liveness probe, and nothing to get wrong about a pid that has been
//! reused. The lock file itself is a durable artifact of no interest: its
//! *content* is never read, only the lock on it.
//!
//! **The one window it cannot close, stated rather than papered over.** An open
//! file description is shared with every `fork`, so a child forked while this
//! descriptor was open holds the lock too, until it closes it — which its own
//! `exec` does, since std opens with `O_CLOEXEC`. The window is therefore
//! fork-to-exec, microseconds, and it can only ever *delay* a next engine, never
//! admit a second one. Nothing in production is inside it: a restart is a new
//! process launched long after the old one's children have exec'd. The suite
//! sees it, because it forks continuously, and `engine::tests` waits it out
//! rather than pretending it is not there.

use std::fs::{File, OpenOptions};
use std::path::Path;

/// The lock file's leaf, under the yog state root (§5.2) beside `ui.json` and
/// `ops.jsonl` — yog's own artifacts, which is what this is.
pub(crate) const LOCK: &str = "engine.lock";

/// The held exclusion. Owns the open file; dropping it closes the descriptor,
/// which is what releases the lock — the §7.2 shutdown shape with nothing to
/// signal and nothing to join.
pub(crate) struct Sole {
    _held: File,
}

/// Claim this world for this engine, or say who has it.
///
/// The refusal names the file, because that is the one place an operator can
/// look to see the fact: `fuser`/`lsof` on it names the holding process, and
/// nothing yog could write there would be more current than the lock itself.
pub(crate) fn take(state_root: &Path) -> Result<Sole, String> {
    let path = state_root.join(LOCK);
    std::fs::create_dir_all(state_root)
        .map_err(|e| format!("engine lock {}: {e}", path.display()))?;
    let held = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| format!("engine lock {}: {e}", path.display()))?;
    match held.try_lock() {
        Ok(()) => Ok(Sole { _held: held }),
        Err(_) => Err(format!(
            "another engine is already running on this world ({}) — one world has one engine",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests;
