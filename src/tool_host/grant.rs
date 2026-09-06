//! **The role's grant, read where litany writes it** (round-1 triage ruling 3,
//! bl-52b7): what `providers.yaml` says the calling agent's role may call.
//!
//! litany's grant gate is one line (its `prompt/dispatch/tool_step/permit.rs`):
//! a role may call a tool that is **in its `tools:` grant OR injected by the
//! host**. The `||` is the whole story. yog's injection was the second half
//! unconditionally — `clients` declared for every agent of every role — so any
//! role could widen itself by loading, and a role's grant bounded nothing that
//! a machine advertised. Measured: a **compactor**, whose grant is empty by
//! litany's own template and whose deletion-only confinement its
//! `docs/PRINCIPLES.md` states as a guarantee, had `bash` refused correctly by
//! that gate, then called `clients {"op":"load"}` and made thirty arbitrary
//! shell calls on an enrolled machine, inside the engine's own git worktrees.
//!
//! **The ruling: a grant is a grant.** `clients` is part of it, not injected
//! beside it — so a role that is not granted `clients` is declared nothing at
//! all, its loaded set included, and an empty grant is empty. The injection
//! stops being an escape hatch from the law litany states and becomes a thing
//! the law governs.
//!
//! Two facts, each read from its single existing home; yog stores neither.
//!
//! 1. **The role** is litany's dispatch commit subject, `dispatch: <role>
//!    [<agent-id>]` (its ARCH §2.5, `prompt/role.rs`: *"a child agent's role
//!    lives in its dispatch commit subject … no sidecar role table"*). A root
//!    agent's subject lacks the prefix, and roots are workers — the same
//!    default litany's own reader applies, not a case of its own.
//! 2. **The grant** is `roles.<role>.tools` in the `providers.yaml` of the
//!    config that **governs** this agent (§9.4's own derivation, the one every
//!    other yog surface asks), read through §9.4's block grammar so the
//!    picker, the fork composer and this reader cannot disagree about what a
//!    config file says.
//!
//! **Every failure is an empty grant, and that is fail-closed.** A workspace
//! that is not there, a repository git will not read, a role the file does not
//! declare, a `tools:` line that is not the flow sequence litany writes: each
//! answers "this role is granted nothing", which declares nothing. The
//! opposite default — treating an unreadable config as full trust — is the
//! thing this ball is about.

use std::path::Path;

use crate::config_edit::branch::{config_file, governing_config};
use crate::git_tree::AGENT_REF_PREFIX;
use crate::model_pick::grammar::{PROVIDERS_YAML, ROLES, entry_field, flow_members};

/// Workspace subdirectory holding the bare repository (litany ARCH §2.2).
const REPO_DIR: &str = "repo.git";
// The agent's **branch** is `agents/<id>`, not the bare id (litany ARCH §2.3),
// and both reads below address the branch — so the prefix comes from
// `git_tree`, the module that already enumerates that namespace, rather than
// from a second spelling here.
/// Subject prefix of a child's dispatch commit (litany ARCH §2.5).
const DISPATCH_PREFIX: &str = "dispatch: ";
/// The role every root agent resolves — litany's own default, spelled here
/// because its constant is crate-private (the same reason the engine-act names
/// are yog's own spelling).
const WORKER: &str = "worker";
/// The four-space field a role's grant rides in.
const TOOLS: &str = "tools";

/// The tools `agent`'s role is granted in `workspace`. Empty for every reason
/// a grant cannot be read — see the module doc.
pub fn of(workspace: &Path, agent: &str) -> Vec<String> {
    let branch = format!("{AGENT_REF_PREFIX}{agent}");
    let Ok(governing) = governing_config(workspace, &branch) else {
        return Vec::new();
    };
    let Ok(bytes) = config_file(workspace, &governing.oid, PROVIDERS_YAML) else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    entry_field(&text, ROLES, &role(workspace, agent, &branch), TOOLS)
        .and_then(|value| flow_members(&value))
        .unwrap_or_default()
}

/// The role recorded in the dispatch commit that founds `agent`'s branch, or
/// [`WORKER`] when there is none — a root, whose subject lacks the prefix.
///
/// The `--grep` is anchored on the exact `[<agent>]` tail, so only the agent's
/// *own* dispatch commit matches and never a descendant's `[<agent>-<sub>]`;
/// exactly one commit per branch carries it, so `-n 1` is the whole answer.
/// litany's own reader is `prompt::role::derive`, which is private to a crate
/// that exposes only `cmd` — so the pattern crosses as text, exactly as the
/// engine-act names do.
fn role(workspace: &Path, agent: &str, branch: &str) -> String {
    let repo = workspace.join(REPO_DIR);
    let pattern = format!("^dispatch: .+ \\[{agent}\\]$");
    crate::git_env::output(
        crate::git_env::git()
            .arg("--git-dir")
            .arg(&repo)
            .args(["log", "-n", "1", "--format=%s", "-E", "--grep"])
            .arg(&pattern)
            .arg(branch),
    )
    .ok()
    .filter(|out| out.status.success())
    .and_then(|out| named(String::from_utf8_lossy(&out.stdout).trim()))
    .unwrap_or_else(|| WORKER.to_owned())
}

/// The role a dispatch subject names, or `None` for any other subject.
fn named(subject: &str) -> Option<String> {
    let rest = subject.strip_prefix(DISPATCH_PREFIX)?;
    let (role, _) = rest.rsplit_once(" [")?;
    (!role.is_empty()).then(|| role.to_owned())
}

#[cfg(test)]
mod tests;
