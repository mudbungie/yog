//! **`clients` is granted to the role that works, and never to machinery**
//! (DESIGN §8.6, bl-0460) — the third file of the one policy convergence
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
//! carry, because the tool is not litany's. And it is not tool-name narrowing
//! either (VISION §4.11's rejection): it only ever adds, and what an invocation
//! may **do** is still adjudicated per call by the §8.6 capability control.
//!
//! **Machinery is never granted.** Only [`WORKER_ROLE`] — litany's *roots are
//! workers*, and the roles this door does not touch are the checkpoint pair:
//! the compactor, whose empty grant is the confinement bl-52b7 restored, and
//! the reviewer, whose grant is stated as its confinement in litany's own
//! template. A role an operator adds is the operator's own grant to write.

use std::path::Path;

use crate::config_edit::branch::edit::DraftFile;
use crate::model_pick::WORKER_ROLE;
use crate::model_pick::grammar::{
    PROVIDERS_YAML, ROLES, TOOLS, entry_field, flow_members, flow_value, upsert_field,
};
use crate::tool_host::clients;

#[cfg(test)]
mod tests;

/// `workspace`'s `providers.yaml` drift on `config/<config>`, or `None` when
/// that tip already grants the worker `clients` — the steady state, which
/// stages nothing and spawns nothing. The lineage is a parameter for §8.7's
/// reason: what a drone may call must be authored where the drone forks.
pub fn drift(workspace: &Path, config: &str) -> Option<DraftFile> {
    let base = crate::control::author::committed(workspace, config, PROVIDERS_YAML)?;
    let want = authored(&base);
    (want != base).then(|| DraftFile {
        rel_path: PROVIDERS_YAML.to_owned(),
        bytes: want.into_bytes(),
    })
}

/// `base` with [`clients::NAME`] in `roles.worker.tools`. A **fixed point**: a
/// file that already grants it comes back byte for byte, and so does one this
/// has nothing to say about.
///
/// **A worker that declares no `tools:` at all is left alone**, and that is the
/// grant's own semantics rather than a gap: an absent list is a role granted
/// nothing, which is a thing an operator may mean, and inventing a list to put
/// one name in would be yog answering a question the file already answered. The
/// same holds for a `tools:` that is not the flow sequence litany writes, and
/// for a rewrite the grammar declines — every one of them is the operator's own
/// file, unchanged.
pub fn authored(base: &str) -> String {
    let Some(members) = entry_field(base, ROLES, WORKER_ROLE, TOOLS).and_then(|v| flow_members(&v))
    else {
        return base.to_owned();
    };
    if members.iter().any(|name| name == clients::NAME) {
        return base.to_owned();
    }
    let mut granted = members;
    granted.push(clients::NAME.to_owned());
    upsert_field(
        PROVIDERS_YAML,
        base,
        ROLES,
        WORKER_ROLE,
        TOOLS,
        &flow_value(&granted),
    )
    .unwrap_or_else(|_| base.to_owned())
}
