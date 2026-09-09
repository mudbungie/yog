//! **Which names a worker's grant may keep** (bl-d281) — the predicate the
//! §8.6 birth fold prunes against, derived from two things yog does not write
//! down and never from a list in this file.
//!
//! A `providers.yaml` is a durable file: a name granted once stays granted
//! through every pin bump, and a name the engine later **retires** therefore
//! outlives the thing it named. What that costs is not an error message. The
//! grant gate passes it (it is in the role's list), the §8.6 control has no
//! intrinsic row for it and so classifies it `opaque`, the shipped table
//! **holds** opaque — and the conversation parks on every single call, with
//! the operator looking at tool calls and no reply. That is the bl-d06c park,
//! observed on a workspace whose lineage still granted litany's retired
//! `multi_tool`; bl-d06c guarded the *shipped* grant against it, and this is
//! the other half, for the grants already on disk.
//!
//! # The two ways a name earns its place
//!
//! **The engine can execute it** ([`ships`]). One subtraction over sets yog
//! owns no half of, so a pin bump moves it and nothing here is edited.
//!
//! **Or the operator has spoken about it** ([`keep`]). A `rules:` row in the
//! workspace's `capability.yaml` naming a class for the name is the operator
//! saying *I know what this reaches* — the one existing explicit signal that
//! distinguishes a stale grant from a deliberate one, and the only shape in
//! which a granted foreign name is callable at all rather than held on every
//! call. That is the REMOTE §5.4 worktree lane's own configuration: a bare
//! name a registered machine advertises and consents to run at the
//! conversation's cwd. Without the row the lane's calls would park exactly as
//! the retired built-in's did, so the predicate prunes precisely the names
//! nothing can execute and nobody has vouched for.
//!
//! **This is not tool-name narrowing** (VISION §4.11's standing rejection:
//! *"tool-name narrowing returns nowhere, because the workhorse tool (`bash`)
//! is every class at once"*). Narrowing removes a name whose effects are
//! distrusted, and the answer to that is per-invocation adjudication. This
//! removes a name **no invocation of which can execute** — its floor is the
//! engine's whole shipped pool, so it can never take a capability the model
//! would otherwise have had.

use crate::control::policy::Policy;
use crate::tool_host::{clients, engine_act};

/// Whether the **pinned engine** can execute `name` at all:
///
/// > `litany::cmd::BUILTIN_TOOLS` — the names `litany tool <name>` answers to,
/// > exported by the engine for exactly this reader — plus
/// > [`engine_act::NAMES`], which adds the compactor's injected procedure pair
/// > (in no `providers.yaml`, granted by nobody, and kept here so a fold over
/// > a hand-written file could not delete one), plus yog's own
/// > [`clients::NAME`], which litany's template cannot carry because the tool
/// > is not litany's.
///
/// Read against the same two sets `tool_host::subject::performs` subtracts, so
/// "the engine ships it" has one answer in this crate.
pub(super) fn ships(name: &str) -> bool {
    ::litany::cmd::BUILTIN_TOOLS.contains(&name) || engine_act::is(name) || name == clients::NAME
}

/// Whether the worker's grant may keep `name`: the engine ships it, or
/// `policy` states its reach. Anything else classifies `opaque` on every call
/// and holds, which is a park rather than a grant.
///
/// The policy handed in is [`Policy::read`]'s — the §8.6 control's own reader,
/// at `config/default`'s live tip — so the file that decides a name's class
/// and the file that decides whether the grant may name it are the same file,
/// read the same way, and cannot disagree.
pub(super) fn keep(name: &str, policy: &Policy) -> bool {
    ships(name) || policy.stated(name).is_some()
}
