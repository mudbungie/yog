//! **Why the latest model call failed**, in the provider's own words (bl-9b88)
//! — the one home for that sentence, read in the same pass §3.5 already reads
//! the latest `response.json`.
//!
//! litany can fail a model call in exactly two places, and until bl-9b88 yog
//! read only one of them:
//!
//! - **In band.** brazen speaks every failure it reaches on stdout (litany
//!   ARCH §4.4), so a [`Framing::Failed`](super::Framing) tail carries an
//!   `error` event and that event *is* the sentence
//!   ([`error_text`](super::error_text)).
//! - **Out of band.** The adapter died before it reached that contract — a
//!   credential-less provider row, a malformed brazen config, an unreadable
//!   credstore — leaving an **empty `response.json`** beside its own
//!   `stderr.log` (litany ARCH §2.3: *"Empty on an ordinary run … bytes here
//!   mean the adapter failed outside that contract"*). Nothing in band says a
//!   thing, which is the shape the live sighting wore: every conversation in a
//!   workspace launched a driver, every model call refused, and the roster
//!   painted a conversation that simply never answers.
//!
//! **One fact, two readings, and the framing gates which one is paid.** A
//! `Complete` tail is a call that worked, so a healthy conversation pays no
//! syscall here; only a `Failed` or `Killed` tail reads anything, and `Failed`
//! reads bytes already in hand. That is why `meta.json` — the §7.3 wound's
//! second observation — is not consulted: the wound is a *per-step* badge that
//! must tell a settled step from an unsettled one, while the question here is
//! the agent's latest call, which the framing has already answered.
//!
//! The **raw** evidence is what rides on [`Agent::failure`](super::Agent),
//! because two consumers want two different amounts of it: the auth heuristic
//! ([`crate::login::auth::looks_auth`]) scans the whole event line — status
//! codes and reason phrases included — and the §11 row says only
//! [`clause`]. Storing the clause and re-deriving the flag from it would
//! narrow the heuristic's input to whatever survived the trim.

use std::path::Path;

use super::terminal::{Framing, Settled, error_text};
use crate::steps_view::records::STDERR_FILE;

/// How much of a failure a **row** says (§11: a row is a glance). The clause
/// is the operator's sighting; the whole of it is the steps surface's
/// `stderr.log` and `auth_failed`, one query deeper.
const CLAUSE_CAP: usize = 120;

/// The latest model call's failure, verbatim, or `None` when it did not fail.
///
/// `step` is the latest step's directory and `response` its already-read
/// bytes; `settled` is the §4.4 reading of those same bytes, so nothing here
/// walks them twice.
pub(super) fn failure(step: &Path, response: &[u8], settled: Settled) -> Option<String> {
    match settled.framing {
        // In band: the error event, exactly as `login::auth` has always scanned it.
        Framing::Failed => error_text(response),
        // Out of band: the adapter's own complaint, through the crate's two
        // standing bounds on how much of a capture yog reads and how much of
        // one a surface says — the same pair the §7.3 wound spends.
        Framing::Killed => {
            let captured = crate::opslog::detached::captured(&step.join(STDERR_FILE));
            let words = crate::opslog::rows::stderr_tail(captured.trim());
            (!words.is_empty()).then_some(words)
        }
        // The call reached its own end. Whether the *turn* did is `truncated`'s
        // question, not this one.
        Framing::Complete => None,
    }
}

/// The row-altitude **first clause** of a failure: the provider's `message`
/// when the evidence is a JSON error event carrying one — an operator reads a
/// sentence, not a wire frame — else the evidence itself; first line only,
/// capped at [`CLAUSE_CAP`] characters, on the §11 row preview's own
/// discipline.
pub(crate) fn clause(raw: &str) -> String {
    let said = message_of(raw).unwrap_or_else(|| raw.to_string());
    said.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim()
        .chars()
        .take(CLAUSE_CAP)
        .collect()
}

/// The `message` field of a JSONL error event, when the evidence is one and it
/// carries a non-empty string there. Absent for an adapter's plain-text stderr
/// and for an event that names only a status code — both of which then say
/// themselves, which is as much as is known.
fn message_of(raw: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let said = value.get("message")?.as_str()?;
    (!said.trim().is_empty()).then(|| said.to_string())
}

#[cfg(test)]
mod tests;
