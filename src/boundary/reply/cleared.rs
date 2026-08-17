//! The composer's draft-clearing predicate — its own file at §12's per-file
//! budget (bl-40ab), on a real seam: a [`Reply`] is what the boundary answers,
//! and whether an answer *clears the draft that asked for it* is a question a
//! seat asks about one, not a property of the type.

use super::Reply;

/// Whether a dispatch outcome was clean — the draft-clearing predicate the
/// composer reads (§5.3: RAM until *sent*): a captured run must have exited 0;
/// any other reply is its action's success by construction; a refusal is not.
pub fn cleared(result: &Result<Reply, String>) -> bool {
    match result {
        Ok(Reply::Outcome(outcome)) => outcome.ok(),
        Ok(_) => true,
        Err(_) => false,
    }
}
