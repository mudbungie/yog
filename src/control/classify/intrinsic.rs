//! **The intrinsic map** (VISION §4.11 item 1): the closed set of tool names
//! this control implements a row for, and the row each one carries.
//!
//! Split out of [`super`] at §12's budget when bl-72bd deleted the
//! fall-off-the-match arm, and the split is the mechanism rather than a tidy:
//! a name is folded into [`Known`] *before* anything classifies it, and
//! [`row`] then matches that enum **exhaustively**, with no catch-all. A
//! fifteenth name cannot be added without a class being chosen for it, and no
//! name can reach a passing class by default — which is exactly what
//! `other => OpenWorld` used to do to every routed foot tool.
//!
//! Three families sit in here, and each is here for its own reason:
//!
//! - **litany's own pool** (`read_file`, `load_skill`, `message`, `dispatch`,
//!   `apply_patch`, `cd`, `bash`, `python`, `remember`, `search_history`,
//!   `read_tool_output`). `cd` and
//!   `apply_patch` are judged against the writable root at consult time and
//!   `bash` goes to the operator ruleset ([`super::super::bash`]); the rest
//!   carry a fixed class.
//! - **the compactor's procedure pair** (`write_summary`, `mark_for_deletion`)
//!   — litany injects them from the calling role's own procedure and yog
//!   performs them at the engine's front door (REMOTE §5.4), so the control
//!   sees them like any other name.
//! - **yog's own roster tool** (`clients`, REMOTE §5).
//!
//! **Substrate verbs are target writes, not exemptions.** `message` and the
//! world's `bl`/`litany` shims mutate the world's own substrates *through their
//! gated verbs* — the delivery law, the front door — which is literally the
//! second half of the target-write definition. They therefore pass by the
//! ordinary table rather than by a bypass, and the control keeps ruling host
//! effects rather than deliveries.

use super::{COMMAND, Classified, Effect, Request, Root};

/// Every tool name the intrinsic map holds a row for. Closed by construction:
/// [`of`](Known::of) is the only way in and [`row`] is exhaustive over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Known {
    ReadFile,
    LoadSkill,
    Message,
    Dispatch,
    ApplyPatch,
    Cd,
    Bash,
    Python,
    Remember,
    SearchHistory,
    ReadToolOutput,
    WriteSummary,
    MarkForDeletion,
    Clients,
}

impl Known {
    /// The row this name names, or `None` for a name the [`super::routed`]
    /// lane owns. The strings are the model's own spelling of them.
    pub(super) fn of(name: &str) -> Option<Known> {
        match name {
            "read_file" => Some(Known::ReadFile),
            "load_skill" => Some(Known::LoadSkill),
            "message" => Some(Known::Message),
            "dispatch" => Some(Known::Dispatch),
            "apply_patch" => Some(Known::ApplyPatch),
            "cd" => Some(Known::Cd),
            "bash" => Some(Known::Bash),
            "python" => Some(Known::Python),
            "remember" => Some(Known::Remember),
            "search_history" => Some(Known::SearchHistory),
            "read_tool_output" => Some(Known::ReadToolOutput),
            "write_summary" => Some(Known::WriteSummary),
            "mark_for_deletion" => Some(Known::MarkForDeletion),
            "clients" => Some(Known::Clients),
            _ => None,
        }
    }
}

/// The class one known name carries for this invocation. Exhaustive over
/// [`Known`] — no catch-all arm, so the compiler asks the question for every
/// name ever added.
pub(super) fn row(
    known: Known,
    request: &Request,
    root: &Root,
    policy: &super::super::policy::Policy,
) -> Classified {
    match known {
        Known::ReadFile => Classified::new(Effect::Read, "reads a file"),
        Known::SearchHistory => Classified::new(
            Effect::Read,
            "searches the conversation's own history, which it observes and does not touch",
        ),
        // The recovery the §3.3 bounded projection's own cut marker names
        // (litany 0.0.12, upstream bl-9a6e): it pages bytes back out of
        // `steps/<agent-id>/<NNN>/tools/<tool-id>/output.json`, a diagnostic
        // record litany already wrote, under a domain bound that is one string
        // equality against the caller's own `LITANY_CONV_BRANCH`. It reads one
        // file the agent itself produced and touches nothing, so it is Read for
        // the reason `search_history` is. `Remember`'s note applies with more
        // force here: without a row it would fall to the routed lane and be
        // held Opaque on every call — an operator answer demanded for the one
        // way out of a cut the harness imposed, which would leave the marker
        // pointing at a door the model cannot open.
        Known::ReadToolOutput => Classified::new(
            Effect::Read,
            "pages this agent's own captured tool output, which it reads and does not touch",
        ),
        // The one lawful door from a step to a durable fact (litany 0.0.11,
        // upstream bl-3c11): it appends to `facts.md` in a config commit on
        // `proposal/<agent-id>`, a branch NO lineage points at until the
        // operator accepts it. So it is the world's own substrate written
        // through the world's own gated verb — the second half of the
        // target-write definition — and deliberately not open-world: it cannot
        // reach past the workspace, and it cannot advance the lineage its own
        // conversation runs on, which is the act litany now refuses under
        // `LITANY_TOOL_ID` (upstream bl-d273, yog bl-baed). Without a row here
        // it would fall to the routed lane and be held Opaque on every call —
        // an operator answer demanded for the door that exists so a model does
        // not go looking for `litany config`.
        Known::Remember => Classified::new(
            Effect::TargetWrite,
            "proposes one durable fact onto a staged branch the operator accepts or drops",
        ),
        Known::LoadSkill => Classified::new(
            Effect::TargetWrite,
            "writes a skill body into the agent worktree",
        ),
        Known::Message => Classified::new(
            Effect::TargetWrite,
            "deposits into another agent's inbox through the world's own gated verb",
        ),
        Known::WriteSummary => Classified::new(
            Effect::TargetWrite,
            "writes the conversation's own summary onto the compactor branch",
        ),
        Known::MarkForDeletion => Classified::new(
            Effect::TargetWrite,
            "stages the conversation's own files for deletion inside its worktree, where git still holds them",
        ),
        Known::Dispatch => Classified::new(
            Effect::Process,
            "mints an agent, under the harness's own budget and depth gates",
        ),
        // A program the model authored, whose effect no input field states.
        // Open-world by intrinsic row rather than by falling off a match
        // (bl-72bd): it is the widest class short of loss, which is what the
        // ruleset gives an unmatched `bash` program for the same reason, and
        // the shipped table passes it exactly as it did before (bl-1ef1).
        Known::Python => Classified::new(
            Effect::OpenWorld,
            "runs a program the model authored, whose reach no input field states",
        ),
        Known::Clients => clients(&request.field("op")),
        Known::ApplyPatch => super::operand::patch(&request.field("input"), root),
        Known::Cd => super::operand::move_to(&request.field("path"), root),
        Known::Bash => super::super::bash::classify(&request.field(COMMAND), root, policy),
    }
}

/// A `clients` op (REMOTE §5): `load` writes the agent's own loaded-tool
/// document through yog's own gated verb, which is the second half of the
/// target-write definition; every other op — and an off-schema input, which
/// names none — only reads the roster.
fn clients(op: &str) -> Classified {
    if op == "load" {
        Classified::new(
            Effect::TargetWrite,
            "loads a client's tool into this agent's own loaded set",
        )
    } else {
        Classified::new(Effect::Read, "reads the workspace's client roster")
    }
}
