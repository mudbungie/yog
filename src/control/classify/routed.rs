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
//! 2. **Everything else is the class the operator STATED for that name, and
//!    otherwise [`Opaque`](Effect::Opaque)**, which the shipped table holds. A
//!    tool whose reach this control cannot read is parked for the operator,
//!    never passed.
//!
//! **The row is how a nameable tool stops being opaque** (bl-b65d). An MCP tool
//! reaches this control as a routed name with an input shaped by its server's
//! schema — `box2_fetch {"url": …}` — so there is no command line to read and
//! every call of it holds. The way out is a `rules:` row keyed on the whole
//! host-qualified name:
//!
//! ```yaml
//! rules:
//!   box2_fetch: open-world
//! ```
//!
//! Host-qualified because the same server on two boxes is two trust decisions
//! (REMOTE §5: locality rides in the name). The class is the **operator's own
//! statement** of what that tool on that box reaches — informed by what `thrall
//! mcp pin` printed of the server's annotations, and by nothing the wire
//! carries: an advertisement states no effect (REMOTE §5.1) and this control
//! infers none from one. That is why only the operator's rows are consulted and
//! the shipped ruleset is not ([`Policy::stated`](super::super::policy::Policy::stated)),
//! and why a row does not outrank a shell's own line — a name row is asked only
//! where there is no line to read. It is a class the operator states,
//! adjudicated per invocation, never a name allowed (bl-7fc8 stands).
//!
//! **The hold says so.** The opaque sentence names the row to write, spelled
//! with the actual name and the class words the file accepts, because a park
//! whose remedy the operator has to go and find is a park they answer by
//! reflex — the `NOT_A_REMEDY` discipline of bl-68e1, from the other direction.
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

use super::super::policy::{CAPABILITY_YAML, Policy};
use super::{COMMAND, Classified, Effect, Request, Root};

/// Classify one invocation of a name the intrinsic map does not hold.
pub(super) fn classify(request: &Request, root: &Root, policy: &Policy) -> Classified {
    let command = request.field(COMMAND);
    if command.trim().is_empty() {
        return stated(request, root, policy);
    }
    super::super::bash::classify(&command, root, policy)
}

/// The class the **operator stated** for this routed name, or the opaque hold
/// that says how to state one (bl-b65d).
fn stated(request: &Request, root: &Root, policy: &Policy) -> Classified {
    let Some(row) = policy.stated(&request.name) else {
        return Classified::new(
            Effect::Opaque,
            format!(
                "{name} is not a tool this control implements and its input carries no command \
                 line, so what it reaches cannot be read — held rather than passed. To state what \
                 it reaches, add a `rules:` row to this workspace's {CAPABILITY_YAML}: \
                 `{name}: <class>`, where <class> is one of {classes}",
                name = request.name,
                classes = Effect::reach_words(),
            ),
        );
    };
    let words = [request.name.clone()];
    let found = super::super::bash::matched(&row, &request.name, &words, root);
    Classified::new(
        found.effect,
        format!(
            "a `rules:` row in {CAPABILITY_YAML} states what `{}` reaches",
            request.name
        ),
    )
}
