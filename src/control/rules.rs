//! The **shipped bash ruleset** (VISION §4.11 item 1): the default classification
//! of a shell segment's program into the effect vocabulary.
//!
//! This table is *policy shipped as data*, not logic. It is the severable
//! default the per-workspace policy config replaces or extends (bl-765d); until
//! that config exists, absence is these rows — the `cadence.yaml` pattern, one
//! layer down.
//!
//! Three properties decide the shape:
//!
//! 1. **First match wins, most specific first.** `git push --force` must be read
//!    as destructive before `git push` is read as open-world and long before
//!    plain `git` is read as a target write.
//! 2. **Unmatched is open-world.** There is no catch-all row, deliberately: a
//!    program the table does not name is a program whose reach nobody has
//!    stated, so it takes the widest class short of loss rather than a narrow
//!    one it might not deserve. That is why interpreters (`python`, `node`,
//!    `sh`) are absent — an interpreter's reach is its script's, which no rule
//!    can see.
//! 3. **Some rows classify by operand.** `rm` inside the writable root is the
//!    ordinary work of a build tree; the same `rm` outside it is loss the repo
//!    cannot give back. One row says both ([`Reach::ByRoot`]).
//!
//! The rows that pass without qualification are the ones VISION §4.11 item 8
//! names honestly: `cargo` and `make` execute arbitrary code from the tree they
//! build, and they pass anyway, because refusing them refuses the job. The wall
//! that stops *that* is OS confinement, later and platform-explicit.

use super::classify::Effect;

/// **The shipped rows**, split off at §12's budget: this file is the grammar a
/// row is written in, `table` is the policy written in it.
mod table;

pub use table::{DEFAULT, SECRET_FRAGMENTS};

/// How a rule decides its segment's reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// The class regardless of operands.
    Fixed(Effect),
    /// The class depends on where the segment's path operands land: `inside`
    /// when every one of them resolves inside the writable root, `outside`
    /// otherwise.
    ByRoot { inside: Effect, outside: Effect },
}

/// One row of the ruleset: `(program, qualifying words, reach)`. A tuple rather
/// than a struct so a row stays one readable line — the table is data, and a
/// field-per-line rendering of ninety rows buries the policy it states.
///
/// - **program** — matched against the segment's leading word's basename.
/// - **qualifying words** — all must appear in the segment for the row to bite.
///   A short flag matches inside a bundle, so `-f` bites on `-fd` (see
///   [`super::bash::has_word`]); [`ANY`] means the program alone decides.
/// - **reach** — the class the row yields.
pub type Rule = (&'static str, &'static [&'static str], Reach);

/// No qualifying words: the program alone decides the row.
pub const ANY: &[&str] = &[];
