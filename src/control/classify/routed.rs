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
//! 2. **Else the class the operator STATED for that name.**
//! 3. **Else, an input naming a `url` (or `urls`) is
//!    [`OpenWorld`](Effect::OpenWorld)** — the class whose meaning is network
//!    reach (bl-c6f0). The same reading as item 1, on the other operand a
//!    routed schema carries.
//! 4. **Otherwise [`Opaque`](Effect::Opaque)**, which the shipped table holds.
//!    A tool whose reach this control cannot read is parked for the operator,
//!    never passed.
//!
//! **The row is how a nameable tool stops being opaque** (bl-b65d). An MCP tool
//! reaches this control as a routed name with an input shaped by its server's
//! schema — `box2_fetch {"url": …}` — so there is no command line to read and
//! (before item 3) every call of it held. The way out is a `rules:` row keyed
//! on the whole host-qualified name:
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
use super::{ADDRESS, COMMAND, Classified, Effect, Request, Root};
use serde_json::Value;

/// Classify one invocation of a name the intrinsic map does not hold: the
/// command line if there is one, else the operator's row, else the address the
/// input names, else the opaque hold.
pub(super) fn classify(request: &Request, root: &Root, policy: &Policy) -> Classified {
    let far = far(root);
    let command = request.field(COMMAND);
    if !command.trim().is_empty() {
        return super::super::bash::classify(&command, &far, policy);
    }
    stated(request, &far, policy)
        .or_else(|| addressed(request))
        .unwrap_or_else(|| opaque(request))
}

/// The writable root **as it stands on the other machine**: empty (bl-1772).
///
/// The doc above has always said a path on the foot classifies to the wider
/// class, and for an *absolute* operand it did. A **relative** one did not: the
/// lexer resolves it against the agent's own cwd, which is the engine's
/// worktree and inside the root — so `cd /srv/data/blobs && rm -f -- *` read as
/// `rm` on a bare `*` inside the writable root and classified target write,
/// while the identical `rm -f /srv/data/blobs/*` classified destructive. The
/// model found that spelling in three steps and deleted 115 MB from a machine
/// the operator was never asked about.
///
/// The reframe is that the special case was the root itself. **This control
/// vouches for no path on a foot**, absolute or relative, so the routed leg's
/// writable set is empty and every `ByRoot` row takes its `outside` class. A
/// `cd` chain then classifies exactly as the direct form, because both are
/// judged against the same nothing — no modelling of `cd`, no glob expansion,
/// and no new arm for either. `cwd` and `home` are kept so an operand still
/// resolves to a path a reason line can name.
fn far(root: &Root) -> Root {
    Root {
        cwd: root.cwd.clone(),
        writable: Vec::new(),
        home: root.home.clone(),
    }
}

/// The class the **operator stated** for this routed name (bl-b65d), or `None`
/// when they have stated none.
fn stated(request: &Request, root: &Root, policy: &Policy) -> Option<Classified> {
    let row = policy.stated(&request.name)?;
    let words = [request.name.clone()];
    let found = super::super::bash::matched(&row, &request.name, &words, root);
    Some(Classified::new(
        found.effect,
        format!(
            "a `rules:` row in {CAPABILITY_YAML} states what `{}` reaches",
            request.name
        ),
    ))
}

/// The class an input that **names an address off this machine** lands in
/// (bl-c6f0): [`OpenWorld`](Effect::OpenWorld), which is what that class means.
///
/// This is the command line's own reading applied to the other operand a
/// routed schema carries. A `url` is not the invocation's assertion about its
/// effect — the thing bl-72bd deleted every path to believing — it is an
/// operand, exactly as a command line is, and the operand says the call
/// reaches the network. Reading it is what makes an MCP fetch tool usable
/// without a row per box: `thrall mcp pin`'s fetch entry is
/// `{"url": …, "max_length": …, "raw": …}` and no line will ever appear in it.
///
/// Three things bound it. It is asked **after** the operator's row and after
/// the command line, so neither is softened by it. It answers only for an
/// input that actually names an address — anything else keeps the hold. And it
/// can only ever answer open-world: no shape of input reaches `read` here.
///
/// **What it does not solve.** A tool whose input names a url and whose act is
/// wider than fetching it — a delete keyed on a resource address — reads
/// open-world and passes, because an operand names the reach and not the verb.
/// The answer to that is the row, which outranks this: `rules:` /
/// `  box2_delete_object: destructive` states what the operator knows and this
/// control cannot see.
fn addressed(request: &Request) -> Option<Classified> {
    let field = ADDRESS
        .iter()
        .find(|key| names_address(&request.input, key))?;
    Some(Classified::new(
        Effect::OpenWorld,
        format!(
            "`{}` carries no command line, and its `{field}` input names an address off this \
             machine — network reach",
            request.name
        ),
    ))
}

/// Whether `input` names an address under `key`: a non-blank string, or a list
/// holding one. Total over every shape — an off-schema value names nothing,
/// which is the hold rather than an error.
fn names_address(input: &Value, key: &str) -> bool {
    let said = |value: &Value| value.as_str().is_some_and(|s| !s.trim().is_empty());
    match input.get(key) {
        Some(Value::Array(items)) => items.iter().any(said),
        Some(value) => said(value),
        None => false,
    }
}

/// The hold for a name whose reach this control could not read at all, spelling
/// the row that would end it (bl-b65d).
fn opaque(request: &Request) -> Classified {
    Classified::new(
        Effect::Opaque,
        format!(
            "{name} is not a tool this control implements and its input carries no command \
             line, so what it reaches cannot be read — held rather than passed. To state what \
             it reaches, add a `rules:` row to this workspace's {CAPABILITY_YAML}: \
             `{name}: <class>`, where <class> is one of {classes}; a key ending in `_` \
             states it for every tool one box advertises",
            name = request.name,
            classes = Effect::reach_words(),
        ),
    )
}
