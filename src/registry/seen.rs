//! **When this client was last connected** (REMOTE §4.1, §5; bl-d542) — the
//! third fact a registration carries, beside the durable advertisement and the
//! live presence.
//!
//! **Two facts could not tell a ghost from a sleeping machine.** A roster row
//! is a name, a `present` bool and an advertised set: `present` is `false` for
//! a machine that spoke ten seconds ago and for one that never once connected,
//! and on a terminal seat it is `false` for everything, because every verb
//! opens and closes its own connection. So a long-lived engine's roster becomes
//! a list of names — and the operator's own `rm` (§4.1: no gesture manages
//! registrations) is unusable, because no row says which is safe to delete.
//!
//! **It is a file because it changes at the rate of a CONNECTION.** §5's two
//! facts are split on their rates of change: an advertisement changes when an
//! operator reconfigures a machine and is durable; presence changes with every
//! network blip and is RAM. A last-seen changes when a client speaks, which is
//! the durable side of that line — and durable is the whole point, since the
//! question it answers ("is that machine real") is asked about a client that is
//! not connected now.
//!
//! ```text
//! <yog-state-root>/clients/<client>/seen   unix seconds, one line
//! ```
//!
//! Beside `tools.json` in the same directory and for the same reason: a
//! per-client fact, not a per-registration one — a machine last spoke at one
//! time, whatever it is registered in.
//!
//! **Absence is "never", and it is the answer that matters.** A client with no
//! file has not connected since the engine learned to write one, which is the
//! honest reading of an unreadable or missing stamp — never a zero, which would
//! be a date.

use std::path::Path;

use super::Client;

/// The stamp's own name under a client's registry directory.
pub const SEEN: &str = "seen";

/// Record that `client` spoke at `unix`. Best-effort by construction: the
/// caller is a request already being answered, and a stamp that could not be
/// written must not refuse the gesture it was riding — the roster loses a
/// column, which is exactly the state it was in before this existed.
pub(crate) fn mark(state_root: &Path, client: &Client, unix: i64) {
    let dir = super::dir(state_root, client);
    if std::fs::create_dir_all(&dir).is_ok() {
        let _ = std::fs::write(dir.join(SEEN), format!("{unix}\n"));
    }
}

/// When `client` last spoke, or `None` for a client that never has. A file that
/// is missing, unreadable or not a number reads as never: all three say the
/// same thing about this client, and telling them apart would be three answers
/// to one question.
pub fn read(state_root: &Path, client: &Client) -> Option<i64> {
    std::fs::read_to_string(super::dir(state_root, client).join(SEEN))
        .ok()?
        .trim()
        .parse()
        .ok()
}

#[cfg(test)]
mod tests;
