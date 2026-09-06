//! **The fail-closed lane** (bl-72bd, ruling 2 of the round-1 triage): what
//! this control answers for a name the intrinsic map does not hold.
//!
//! Almost every such name today is a **routed** one — a tool a registered
//! machine advertises, presented to the model host-qualified (`box2_shell`,
//! REMOTE §5) and executed on that machine. The rest is anything a later
//! litany adds. Neither is a name this control implements, and the answer for
//! both is the same:
//!
//! 1. **An input carrying a command line is classified by that command line,
//!    exactly as the engine's own `bash` is.** A foot's shell is a shell. The
//!    field is `command` — the one the engine's `bash` reads and the one every
//!    thrall tool schema of that shape declares — so the two spellings have one
//!    home and cannot drift.
//! 2. **Everything else is [`Opaque`](Effect::Opaque)**, which the shipped
//!    table holds. A tool whose reach this control cannot read is parked for
//!    the operator, never passed.
//!
//! **Why the old answer was backwards.** The arm this replaces was
//! `other => OpenWorld`, and the shipped table passes open-world — so
//! `box2_shell {"command": "find /srv/data/blobs -delete"}` was passed without
//! a word while the identical line through the engine's own `bash` classified
//! destructive and was refused. The control was strictest about the machine
//! the operator is sitting at and blind about the remote boxes a foot exists
//! to administer, which is the leg whose blast radius the operator cannot see.
//!
//! **The command line is read against the LOCAL writable root, and that is the
//! point.** A `ByRoot` row resolves its operands against the agent's own
//! worktree, which is on the server; a path on the foot's machine is not in it
//! and classifies to the wider class. Ruling 2 says a routed shell is
//! classified by its command line *exactly as* the engine's bash is, and this
//! is what that costs and what it buys: the same table, and a bias toward the
//! wider class on the leg the adjudicator cannot see into (REMOTE §5's honesty
//! clause).
//!
//! **The operator can undo it in one line, which is what keeps it severable**
//! (DESIGN §8.6): a workspace that wants the old behaviour writes `table:` /
//! `  opaque: pass` into its `capability.yaml`. Absence stays the shipped
//! default, and the shipped default is now the closed one.

use super::super::policy::Policy;
use super::{COMMAND, Classified, Effect, Request, Root};

/// Classify one invocation of a name the intrinsic map does not hold.
pub(super) fn classify(request: &Request, root: &Root, policy: &Policy) -> Classified {
    let command = request.field(COMMAND);
    if command.trim().is_empty() {
        return Classified::new(
            Effect::Opaque,
            format!(
                "{} is not a tool this control implements and its input carries no command \
                 line, so what it reaches cannot be read — held rather than passed",
                request.name
            ),
        );
    }
    super::super::bash::classify(&command, root, policy)
}
