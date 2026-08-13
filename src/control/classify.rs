//! The **effect vocabulary** (VISION §4.11 item 1) and the classification of one
//! invocation into it.
//!
//! The vocabulary classifies **invocations, never tool names**. That is the
//! whole reason the shipped grant stays whole-pool (bl-7fc8): `bash` is every
//! class at once, so a per-name allow-list is theatre and only per-invocation
//! adjudication can tell a `ls` from a `curl | sh`.
//!
//! Six classes, ordered by how far each reaches past the job:
//!
//! | Class | Reaches |
//! |---|---|
//! | [`Read`](Effect::Read) | observes only |
//! | [`TargetWrite`](Effect::TargetWrite) | the writable root, or the world's own substrates through their gated verbs |
//! | [`Process`](Effect::Process) | mints agents or processes beyond the invocation |
//! | [`OpenWorld`](Effect::OpenWorld) | past the root and the world: network egress, host writes, a `cd` out |
//! | [`Destructive`](Effect::Destructive) | irreversible loss: history rewrite, forced refs, deletion past git's reach |
//! | [`Secret`](Effect::Secret) | credentials and environment |
//!
//! Built-ins carry an **intrinsic map**; two of them (`cd`, `apply_patch`) are
//! judged against the writable root at consult time, and `bash` goes to the
//! operator ruleset ([`super::bash`]). Everything the map does not name — an
//! external `lernie-tool-*` binary, a tool a later lernie adds — is
//! [`OpenWorld`](Effect::OpenWorld): classification error fails toward the
//! widest class short of loss, so an unrecognised effect is never mistaken for
//! a read and stays something an override or a floor can catch.
//!
//! **Substrate verbs are target writes, not exemptions.** `message` and the
//! world's `bl`/`lernie` shims mutate the world's own substrates *through their
//! gated verbs* — the delivery law, the front door — which is literally the
//! second half of the target-write definition. They therefore pass by the
//! ordinary table rather than by a bypass, and the control keeps ruling host
//! effects rather than deliveries.

use super::root::Root;
use super::wire::Request;

/// A tool's reach, in the six-class vocabulary. Ordered: a higher variant is a
/// wider reach, which is what lets a compound command take the worst of its
/// parts without a table of pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Effect {
    Read,
    TargetWrite,
    Process,
    OpenWorld,
    Destructive,
    Secret,
}

impl Effect {
    /// The class in the operator's words — the noun a reason line uses.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Effect::Read => "read",
            Effect::TargetWrite => "target write",
            Effect::Process => "process",
            Effect::OpenWorld => "open-world",
            Effect::Destructive => "destructive",
            Effect::Secret => "secret",
        }
    }

    /// The class a policy file's word names, or `None` for anything else. The
    /// inverse of [`word`](Effect::word), read off the same list both ways —
    /// so an operator writes the vocabulary the reason lines already speak.
    /// `target-write` is the one word with a hyphen where the sentence has a
    /// space: a policy row is one token per field.
    pub fn of(word: &str) -> Option<Effect> {
        [
            Effect::Read,
            Effect::TargetWrite,
            Effect::Process,
            Effect::OpenWorld,
            Effect::Destructive,
            Effect::Secret,
        ]
        .into_iter()
        .find(|e| e.word().replace(' ', "-") == word)
    }

    /// The wider of two reaches — the fold a compound `bash` command uses.
    #[must_use]
    pub fn worst(self, other: Effect) -> Effect {
        if other > self { other } else { self }
    }
}

/// A classified invocation: its reach, and the one clause that says why. The
/// clause is what a refusal hands the model and a hold hands the operator, so
/// it names concrete things (the command, the path) and never a doc coordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classified {
    pub effect: Effect,
    pub why: String,
}

impl Classified {
    fn new(effect: Effect, why: impl Into<String>) -> Self {
        Self {
            effect,
            why: why.into(),
        }
    }
}

/// Built-in tool names, as lernie spells them.
const READ_FILE: &str = "read_file";
const LOAD_SKILL: &str = "load_skill";
const MESSAGE: &str = "message";
const DISPATCH: &str = "dispatch";
const MULTI_TOOL: &str = "multi_tool";
const APPLY_PATCH: &str = "apply_patch";
const CD: &str = "cd";
const BASH: &str = "bash";

/// Classify one invocation. Total over every tool name and every input shape:
/// an input that does not match its schema simply yields no operands, which
/// lands it in the same open-world arm an unknown tool does.
pub fn classify(request: &Request, root: &Root, policy: &super::policy::Policy) -> Classified {
    match request.name.as_str() {
        READ_FILE => Classified::new(Effect::Read, "reads a file"),
        LOAD_SKILL => Classified::new(
            Effect::TargetWrite,
            "writes a skill body into the agent worktree",
        ),
        MESSAGE => Classified::new(
            Effect::TargetWrite,
            "deposits into another agent's inbox through the world's own gated verb",
        ),
        DISPATCH => Classified::new(
            Effect::Process,
            "mints an agent, under the harness's own budget and depth gates",
        ),
        // The envelope itself observes nothing and changes nothing: lernie
        // adjudicates each inner invocation through this same seam, so passing
        // the wrapper structurally is the general path, not an exemption.
        MULTI_TOOL => Classified::new(
            Effect::Read,
            "an envelope whose every inner invocation is adjudicated on its own",
        ),
        APPLY_PATCH => patch(&request.field("input"), root),
        CD => move_to(&request.field("path"), root),
        BASH => super::bash::classify(&request.field("command"), root, policy),
        other => Classified::new(
            Effect::OpenWorld,
            format!("{other} is not a tool this control can classify"),
        ),
    }
}

/// A `cd`: [`Read`](Effect::Read) inside the writable root (moving is not
/// writing), [`OpenWorld`](Effect::OpenWorld) out of it — a move out of the root
/// is how every later relative operand leaves it.
fn move_to(path: &str, root: &Root) -> Classified {
    let dest = root.resolve(path);
    if root.holds(&dest) {
        Classified::new(
            Effect::Read,
            format!("moves to {} inside the writable root", dest.display()),
        )
    } else {
        Classified::new(
            Effect::OpenWorld,
            format!("moves to {}, outside the writable root", dest.display()),
        )
    }
}

/// An `apply_patch`: a target write when every file the envelope names resolves
/// inside the writable root, open-world otherwise. An envelope naming no file
/// at all patches nothing and reads as a write of nothing.
fn patch(envelope: &str, root: &Root) -> Classified {
    let paths = patch_paths(envelope);
    if root.holds_all(&paths) {
        Classified::new(
            Effect::TargetWrite,
            "patches files inside the writable root",
        )
    } else {
        Classified::new(
            Effect::OpenWorld,
            format!(
                "patches {}, outside the writable root",
                outside(&paths, root)
            ),
        )
    }
}

/// The first operand of `paths` that falls outside the root, for the reason
/// line. Total: the caller only asks when one exists, and an empty answer would
/// still read as a sentence.
fn outside(paths: &[String], root: &Root) -> String {
    paths
        .iter()
        .find(|p| !root.holds(&root.resolve(p)))
        .cloned()
        .unwrap_or_default()
}

/// Every path an `apply_patch` envelope names — its `Add File` / `Delete File` /
/// `Update File` sections and any `Move to` destination.
fn patch_paths(envelope: &str) -> Vec<String> {
    const MARKERS: [&str; 4] = [
        "*** Add File: ",
        "*** Delete File: ",
        "*** Update File: ",
        "*** Move to: ",
    ];
    envelope
        .lines()
        .filter_map(|line| {
            MARKERS
                .iter()
                .find_map(|m| line.trim_end().strip_prefix(m))
                .map(|p| p.trim().to_owned())
        })
        .filter(|p| !p.is_empty())
        .collect()
}

#[cfg(test)]
mod tests;
