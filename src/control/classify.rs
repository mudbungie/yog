//! The **effect vocabulary** (VISION §4.11 item 1) and the classification of one
//! invocation into it.
//!
//! The vocabulary classifies **invocations, never tool names**. That is the
//! whole reason the shipped grant stays whole-pool (bl-7fc8): `bash` is every
//! class at once, so a per-name allow-list is theatre and only per-invocation
//! adjudication can tell a `ls` from a `curl | sh`.
//!
//! Seven classes. Six are reaches, ordered by how far each goes past the job;
//! the seventh is the absence of one:
//!
//! | Class | Reaches |
//! |---|---|
//! | [`Read`](Effect::Read) | observes only |
//! | [`TargetWrite`](Effect::TargetWrite) | the writable root, or the world's own substrates through their gated verbs |
//! | [`Process`](Effect::Process) | mints agents or processes beyond the invocation |
//! | [`OpenWorld`](Effect::OpenWorld) | past the root and the world: network egress, host writes, a `cd` out |
//! | [`Destructive`](Effect::Destructive) | irreversible loss: history rewrite, forced refs, deletion past git's reach |
//! | [`Secret`](Effect::Secret) | credentials and environment |
//! | [`Opaque`](Effect::Opaque) | **unknown** — this control could not read what the invocation does |
//!
//! **There is no arm from a tool NAME to a passing class** (bl-72bd). Names are
//! folded into a closed enum first ([`intrinsic::Known`]) and matched
//! exhaustively, so a name added without a row does not compile; everything the
//! enum does not name goes to [`routed`], whose two answers are the command
//! line's own class and [`Opaque`](Effect::Opaque). The arm this replaced read
//! `other => OpenWorld`, and open-world passes: a foot's `box2_shell` running
//! `rm -rf` was therefore passed unread while the engine's own `bash` refused
//! the same line. Falling off a match into the most permissive class is the one
//! answer nobody chose, and it is now unrepresentable rather than merely fixed.

use super::root::Root;
use super::wire::Request;

/// The input field a command line rides in — litany's own `bash` schema and
/// every thrall shell tool's ([`routed`]), said once here so the built-in and
/// the routed lane read the same field name and cannot drift.
const COMMAND: &str = "command";

/// The intrinsic map: the closed set of names this control implements a row
/// for, and the row each carries.
mod intrinsic;
/// The two intrinsic rows judged against the writable root at consult time.
mod operand;
/// The fail-closed lane for every name the intrinsic map does not hold
/// (bl-72bd) — a foot's routed tool, or anything a later litany adds.
mod routed;

/// A tool's reach, in the seven-class vocabulary. Ordered: a higher variant is
/// a wider reach, which is what lets a compound command take the worst of its
/// parts without a table of pairs. [`Opaque`](Effect::Opaque) is highest
/// because an unread invocation may be any of them — the fold has to carry the
/// unknown outward, never let a known part bury it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Effect {
    Read,
    TargetWrite,
    Process,
    OpenWorld,
    Destructive,
    Secret,
    Opaque,
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
            Effect::Opaque => "opaque",
        }
    }

    /// The class in a **policy file's** spelling — the sentence's word with its
    /// space hyphenated, because a policy row is one token per field.
    /// `target-write` is the only word the two spellings differ on.
    pub fn policy_word(self) -> String {
        self.word().replace(' ', "-")
    }

    /// The six **reaches**, widest last: the classes an operator may state a
    /// tool's reach as. [`Opaque`](Effect::Opaque) is not among them — it is
    /// the absence of a reach, which only a `table:` row ever names — so this
    /// is the list a hold sentence offers and the list `of` reads plus that
    /// one named exception, rather than a second copy of the vocabulary.
    const REACHES: [Effect; 6] = [
        Effect::Read,
        Effect::TargetWrite,
        Effect::Process,
        Effect::OpenWorld,
        Effect::Destructive,
        Effect::Secret,
    ];

    /// The class a policy file's word names, or `None` for anything else. The
    /// inverse of [`policy_word`](Effect::policy_word), read off the same list
    /// both ways — so an operator writes the vocabulary the reason lines
    /// already speak.
    pub fn of(word: &str) -> Option<Effect> {
        Effect::REACHES
            .into_iter()
            .chain([Effect::Opaque])
            .find(|e| e.policy_word() == word)
    }

    /// The reach words a `rules:` row accepts, as a hold sentence offers them
    /// (bl-b65d). The operator is told the way out in the vocabulary the file
    /// actually reads, and it cannot drift from what [`of`](Effect::of)
    /// accepts because both are this one list.
    pub fn reach_words() -> String {
        Effect::REACHES
            .iter()
            .map(|e| e.policy_word())
            .collect::<Vec<_>>()
            .join(", ")
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
    /// Build one. `pub(crate)` rather than private since bl-72bd split the
    /// intrinsic map and the routed lane out of this file: the two arms of one
    /// classification are two modules now, and both mint this.
    pub(crate) fn new(effect: Effect, why: impl Into<String>) -> Self {
        Self {
            effect,
            why: why.into(),
        }
    }
}

/// **The compactor's checkpoint pair** — the two names litany injects from the
/// calling role's own procedure and that its shipped `providers.yaml` says is
/// "never declarable here". They are machinery, not the agent's acts: the pair
/// writes the conversation's own summary onto its compactor branch and
/// nominates that same conversation's files, and nothing else.
///
/// It exists for the **floor** ([`judge::Answers::ruling`](super::judge::Answers::ruling))
/// and for nothing else. A floor is a statement about what the AGENT may do to
/// the world; the compaction procedure is confined by construction rather than
/// by adjudication — the same argument the grant already makes for it (bl-52b7)
/// — so a floor that held it would queue the operator a decision with no
/// decision in it and stall compaction under exactly the policy an operator
/// sets when they are most worried (bl-a821).
///
/// It reads the closed [`intrinsic::Known`] enum rather than the strings, so
/// the pair has one home: a rename upstream moves [`intrinsic::Known::of`] and
/// this follows it.
pub fn checkpoint(name: &str) -> bool {
    matches!(
        intrinsic::Known::of(name),
        Some(intrinsic::Known::WriteSummary | intrinsic::Known::MarkForDeletion)
    )
}

/// **Which leg an invocation runs on** (bl-1772). The engine's own tools run on
/// the machine the operator is sitting at; every other name is a tool a
/// registered machine advertises, executed on that machine (REMOTE §5). The
/// classifier already folds the two apart, and the *judgment* needs the same
/// fact, because a refusal delivered to the model is only an answer where the
/// model is not the party that can walk around it (see
/// [`Ruling::for_the_operator`](super::judge::Ruling::for_the_operator)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leg {
    /// The engine's own machine: an intrinsic name, adjudicated in band.
    Engine,
    /// A machine a foot administers: everything else.
    Routed,
}

impl Leg {
    /// The leg one tool name runs on — the intrinsic map's own answer, so the
    /// two readers of "is this ours" cannot drift.
    pub fn of(name: &str) -> Leg {
        if intrinsic::Known::of(name).is_some() {
            Leg::Engine
        } else {
            Leg::Routed
        }
    }
}

/// Classify one invocation. Total over every tool name and every input shape:
/// an input that does not match its schema simply yields no operands, and a
/// name no row names goes to the [`routed`] lane rather than to a default arm.
pub fn classify(request: &Request, root: &Root, policy: &super::policy::Policy) -> Classified {
    match intrinsic::Known::of(&request.name) {
        Some(known) => intrinsic::row(known, request, root, policy),
        None => routed::classify(request, root, policy),
    }
}

#[cfg(test)]
mod tests;
