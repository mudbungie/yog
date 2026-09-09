//! **The worker's grant converges on what this install can call** — `clients`
//! asserted (DESIGN §8.6, bl-0460) and a name the pinned engine no longer
//! ships pruned (bl-d281) — the third file of the one policy convergence
//! [`execute_ensure_workspace`](super::execute_ensure_workspace) already runs.
//!
//! bl-52b7 made the roster tool part of the **grant**: `Injection::tools`
//! declares nothing at all for a role whose `providers.yaml` `tools:` does not
//! name `clients`, because litany's permit gate is *in the grant OR injected by
//! the host*, and injecting beside a grant is granting. It is the right law —
//! a compactor whose grant is empty had `bash` refused and then ran thirty
//! shell calls on an enrolled machine through `clients {"op":"load"}` — and it
//! left every workspace on the wrong side of it: `clients` is **yog's** tool,
//! litany's shipped template cannot name it, so after bl-52b7 no role in any
//! workspace, new or existing, could reach a foot.
//!
//! # Why this and not the install-wide template override
//!
//! The override (`<world>/litany/template/providers.yaml`, litany's own
//! `TEMPLATE_OVERRIDE_DIR` seam) is the obvious home and it is unavailable in
//! both halves. It replaces a template file **wholesale**, and litany's
//! embedded template is not reachable from here — the crate exports `cmd` and
//! `mint` and nothing else — so yog would have to carry a hand-copy of a file
//! whose own header warns that a copy is drift: *"a name added to the pool and
//! forgotten here is grantless for everyone"*. And it reaches only workspaces
//! **not yet born**, while the defect is in every existing one.
//!
//! So the grant rides the birth-time fold yog already owns (the ruling's own
//! second option): one name asserted in one role's list, in the same single
//! `litany config` pass that authors §8.6's control block and §3.7's
//! instruction glob, **outside the create skip** — so a workspace made a moment
//! ago and one made last week both converge at their next start. Every other
//! byte of the file is the operator's, litany's tool names included.
//!
//! **It is not bl-7fc8's deleted grant path returning.** That one re-asserted
//! `message`/`dispatch` — names litany's template already carried — so it read
//! the role, found them present and authored nothing, a no-op kept alive by its
//! own stale comments. This asserts the one name litany's template *cannot*
//! carry, because the tool is not litany's.
//!
//! # The prune, and why it is not narrowing (bl-d281)
//!
//! Since bl-d281 the same pass also **removes** a granted name this install
//! cannot call: the pin moves and a `providers.yaml` does not, so a workspace
//! born before litany retired `multi_tool` still granted it, the grant gate
//! still passed it, the §8.6 control had no row for it and classified it
//! `opaque`, and the shipped table held — a park on every call, which is the
//! defect bl-d06c reported and guarded only for the *shipped* grant. Old
//! workspaces converge at their next start instead.
//!
//! It is still not tool-name narrowing (VISION §4.11's rejection: *"tool-name
//! narrowing returns nowhere, because the workhorse tool (`bash`) is every
//! class at once"*). Narrowing removes names whose **effects** are distrusted,
//! and what an invocation may do is still adjudicated per call by the §8.6
//! control, exactly as before. The prune removes only names **no invocation of
//! which can execute**: its floor is the engine's whole shipped pool plus
//! anything the operator's own `capability.yaml` speaks for, so it can never
//! take away a capability the model would otherwise have had. [`keep`] is the
//! whole of that predicate and neither half is a list yog writes.
//!
//! **Machinery is never granted.** Only [`WORKER_ROLE`] — litany's *roots are
//! workers*, and the roles this door does not touch are the checkpoint pair:
//! the compactor, whose empty grant is the confinement bl-52b7 restored, and
//! the reviewer, whose grant is stated as its confinement in litany's own
//! template. A role an operator adds is the operator's own grant to write.
//!
//! # A grant and its description are one fact (bl-7d33)
//!
//! bl-0460 wrote the grant alone and that broke every conversation on main:
//! litany's *descriptions-always* rule (ARCH §3.3) refuses a **fork** whose
//! role grants a tool the governing config commit does not describe — before
//! any injection is consulted, so an injected tool is no exception. Its own
//! words for why are this module's argument back at it: *"`providers.yaml` and
//! `descriptions/**` disagree, and both live in that one commit"*. Every
//! `/prompt` answered `started` and died at the fork with
//! `no descriptions/tools/clients.json`. The two files move together now, in
//! the same pass, or neither moves — and the check is per file, so a workspace
//! already carrying bl-0460's half-written state converges at its next start.
//!
//! The schema written is [`clients::schema`] itself, which is the very value
//! [`ToolInjection::tools`](crate::tool_host) declares, so the committed
//! description and the wire declaration cannot disagree. It is the honest one
//! for the second reader too: since litany's bl-55b1 cut,
//! `descriptions/tools/` **is** the callable set an agent reads with `bash`.
//!
//! **The pin does not decide this.** litany 0.0.10 and 0.0.11 run the same
//! check with the same input — `clients` is yog's tool in both, and 0.0.11's
//! `BUILTIN_TOOLS` gains `remember`, not this — so the fix is the same
//! whichever version the manifest names.

use std::path::Path;

use crate::config_edit::branch::edit::DraftFile;
use crate::control::policy::Policy;
use crate::model_pick::WORKER_ROLE;
use crate::model_pick::grammar::{
    PROVIDERS_YAML, ROLES, TOOLS, entry_field, flow_members, flow_value, upsert_field,
};
use crate::tool_host::clients;

/// Which names the grant may keep — the pinned engine's own set, and the
/// operator's own statement about anything outside it.
mod keep;

#[cfg(test)]
mod tests;

/// Worktree-relative home of the committed tool schemas (litany ARCH §3.3),
/// spelled here because litany's own constant is crate-private — the same
/// reason [`tool_host::grant`](crate::tool_host) spells the field names it
/// reads.
const SCHEMA_PATH: &str = "descriptions/tools/clients.json";

/// `workspace`'s drift on `config/<config>` — the grant and the description it
/// is worthless without, empty when the tip already carries both. That is the
/// steady state, which stages nothing and spawns nothing. The lineage is a
/// parameter for §8.7's reason: what a drone may call must be authored where
/// the drone forks.
pub fn drift(workspace: &Path, config: &str) -> Vec<DraftFile> {
    let read = |file: &str| crate::control::author::committed(workspace, config, file);
    let Some(base) = read(PROVIDERS_YAML) else {
        return Vec::new();
    };
    let want = authored(&base, &Policy::read(workspace));
    if !granted(&want) {
        // Nothing granted, so nothing to describe: a worker with no `tools:`
        // is a role granted nothing, and yog does not invent a description for
        // a tool it did not grant.
        return Vec::new();
    }
    let mut drafts = Vec::new();
    if want != base {
        drafts.push(DraftFile {
            rel_path: PROVIDERS_YAML.to_owned(),
            bytes: want.into_bytes(),
        });
    }
    let schema = described();
    if read(SCHEMA_PATH).as_deref() != Some(schema.as_str()) {
        drafts.push(DraftFile {
            rel_path: SCHEMA_PATH.to_owned(),
            bytes: schema.into_bytes(),
        });
    }
    drafts
}

/// The committed description of [`clients::NAME`]: the schema the injection
/// declares, verbatim, so the config commit and the wire cannot disagree.
/// Trailing newline because it is a committed text file.
fn described() -> String {
    let mut out = serde_json::to_string_pretty(&clients::schema()).unwrap_or_default();
    out.push('\n');
    out
}

/// Whether `text` grants the worker [`clients::NAME`] — asked of the *authored*
/// file, so a grant this pass is about to write counts as granted.
fn granted(text: &str) -> bool {
    entry_field(text, ROLES, WORKER_ROLE, TOOLS)
        .and_then(|value| flow_members(&value))
        .is_some_and(|members| members.iter().any(|name| name == clients::NAME))
}

/// `base` with `roles.worker.tools` converged on what this install can
/// actually call: every name [`keep`] passes, plus [`clients::NAME`]. A
/// **fixed point** — a file already carrying that list comes back byte for
/// byte, and so does one this has nothing to say about.
///
/// **The prune is the second half of bl-d06c** (bl-d281). A grant is durable
/// and a pin is not, so a workspace born before a retirement keeps naming what
/// the engine no longer ships; every call of such a name classifies `opaque`,
/// the shipped table holds it, and the conversation parks with nothing said.
/// Converging here rather than at the moment of the park is what makes an old
/// workspace fix itself at its next start. `keep`'s own header carries why the
/// floor is the engine's whole pool and why the operator's `rules:` row is the
/// one thing that can widen it.
///
/// **A worker that declares no `tools:` at all is left alone**, and that is the
/// grant's own semantics rather than a gap: an absent list is a role granted
/// nothing, which is a thing an operator may mean, and inventing a list to put
/// one name in would be yog answering a question the file already answered. The
/// same holds for a `tools:` that is not the flow sequence litany writes, and
/// for a rewrite the grammar declines — every one of them is the operator's own
/// file, unchanged.
pub fn authored(base: &str, policy: &Policy) -> String {
    let Some(members) = entry_field(base, ROLES, WORKER_ROLE, TOOLS).and_then(|v| flow_members(&v))
    else {
        return base.to_owned();
    };
    let mut want: Vec<String> = members
        .iter()
        .filter(|name| keep::keep(name, policy))
        .cloned()
        .collect();
    if !want.iter().any(|name| name == clients::NAME) {
        want.push(clients::NAME.to_owned());
    }
    if want == members {
        return base.to_owned();
    }
    upsert_field(
        PROVIDERS_YAML,
        base,
        ROLES,
        WORKER_ROLE,
        TOOLS,
        &flow_value(&want),
    )
    .unwrap_or_else(|_| base.to_owned())
}
