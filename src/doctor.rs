//! **Is this box wired up?** (DESIGN §8.5, REMOTE §8; bl-28f4) — the one read
//! that asks every question a first conversation depends on, at once, and says
//! the act that fixes each one it can.
//!
//! **Every fact here is already derived somewhere.** The wire material is
//! [`material::read_dir`](crate::wire::material::read_dir)'s three answers, the
//! endpoint is `wire/address`, the bound port is the listener's own
//! ([`Listening`](crate::wire::Listening)), the wall's readiness is the
//! `Prompt` door's own gate ([`signin`](crate::boundary::dispatch::signin)) and
//! the roster is §5's join. **Nothing new is stored, computed twice or invented
//! here** — a doctor that re-judged a wall would be a second authority for the
//! decision the door already makes, and the second one is the one that drifts.
//! What was missing was the *asking*: each gate was met one at a time, after a
//! fresh failure, and each refusal was individually excellent and locally
//! blind.
//!
//! **The rows say what was read, and a failed one says the act.** That is the
//! shape of the best refusal in the suite — *"sign in first: no provider in
//! this workspace's wall holds a credential … `/login <provider>`"* — asked
//! before the failure instead of at it.
//!
//! **There is no tally field.** A seat that wants "17 ok, 2 notes" counts the
//! rows it was handed; a number beside them would be the same fact stored
//! twice, and the two would disagree the first time a row was added.
//!
//! **It reads and never writes.** A diagnostic must work on a box that is
//! broken in the way it is diagnosing, so nothing here provisions, mints,
//! founds or signs in — every remedy is a sentence naming the operator's own
//! act.

use std::path::Path;

use crate::boundary::dispatch::Deps;

/// The engine-wide checks — wire material, the endpoint, the listener, the
/// identity every substrate spawn signs with.
mod engine;
/// The per-workspace checks — the wall the first conversation needs, and who is
/// registered to reach it.
mod workspace;

#[cfg(test)]
mod tests;

/// One check, as every seat renders it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// What was examined — `wire`, `address`, `listener`, `git`, `wall`,
    /// `clients`. One word, so a seat can key on it.
    pub check: String,
    /// Whether this box passes it. A row that is merely informational passes.
    pub ok: bool,
    /// What was read, in a sentence — the fact, never a verdict.
    pub fact: String,
    /// The act that fixes it, present exactly when [`ok`](Row::ok) is false. A
    /// remedy on a passing row would be advice nobody asked for; a failing row
    /// without one is the dead end this gesture exists to end.
    pub remedy: Option<String>,
}

impl Row {
    /// A passing row: the fact, and nothing to do about it.
    fn ok(check: &str, fact: String) -> Self {
        Self {
            check: check.to_owned(),
            ok: true,
            fact,
            remedy: None,
        }
    }

    /// A failing row: the fact, and the act.
    fn bad(check: &str, fact: String, remedy: String) -> Self {
        Self {
            check: check.to_owned(),
            ok: false,
            fact,
            remedy: Some(remedy),
        }
    }
}

/// Examine this box, and — when the gesture named one — the workspace with it.
///
/// The engine rows always answer, because the questions they ask are the ones a
/// box with no workspace at all still has; the workspace rows ride only when a
/// workspace was named, since a seat with none selected is asking about the
/// box.
pub fn examine(deps: &Deps, workspace: Option<(&str, &Path)>) -> Vec<Row> {
    let mut rows = engine::rows(deps);
    if let Some((name, path)) = workspace {
        rows.extend(workspace::rows(deps, name, path));
    }
    rows
}
