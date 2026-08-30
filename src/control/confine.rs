//! The **OS confinement backend** (VISION §4.11 item 8, DESIGN §8.6): the one
//! platform-explicit layer a `confinement: required` workspace spends, wired
//! at yog's own litany spawn seam.
//!
//! **One backend, named per platform.** Linux is **bubblewrap** (`bwrap`),
//! shelled as a subprocess exactly the way certificates are minted by shelling
//! to `openssl` (CLAUDE rule 6's "one recipe") — never linked, no new
//! dependency, no `unsafe`. No other platform has a backend wired, so on any
//! other OS the [`gate`] keeps the standing refusal, naming the OS. Never a
//! silent fallback.
//!
//! **Availability is derived, never stored** ([`available`]): the probe runs
//! the exact sandbox [`SHAPE`] the wrap spends, at the moment of the birth —
//! a `bwrap` that is absent, or a kernel that refuses it user namespaces,
//! *is* the unavailability, with no field anywhere to go stale.
//!
//! **The wrap is unconditional under the policy** ([`wrapper`]): a workspace
//! whose live policy requires confinement has *every* workspace-bound litany
//! spawn prefixed with the backend argv, probe or no probe. A backend that
//! vanishes between gate and spawn therefore fails the spawn loudly (the exec
//! names `bwrap`) instead of quietly running bare — fail-closed by
//! construction. Severable the other way too: no `confinement:` line is an
//! empty wrapper and a byte-identical spawn.
//!
//! **The support boundary — what the shape clamps and what it leaves.**
//! - *Filesystem writes*: the whole host tree is re-bound read-only; writable
//!   is exactly the derived set ([`writable`]) — the workspace, the composed
//!   world root (§16.2: litany home, nested balls state, walls, tools), the
//!   host `/tmp` (world-writable on the host by design; a private tmpfs would
//!   break cross-process temp coordination for nothing), and the **bound
//!   project repo** when the §3.2 claimant join names one ([`bound_project`],
//!   bl-34b1) — without which a ball-rung drone could not run its own `bl
//!   close`.
//! - *Process access*: **not clamped.** A pid namespace dies with its init,
//!   and litany's short verbs detach-launch drivers that must outlive them
//!   (its ARCH §2.9), so `--unshare-pid` would kill every revived driver at
//!   the verb's exit; `litany stop` also signals by host pid. The capability
//!   boundary's `process` class governs.
//! - *Environment*: **not clamped.** The spawn's env is already composed
//!   explicitly at yog's boundary (the §16.2 world + wall folds, the git
//!   scrub); the backend passes it through unchanged.
//! - *Network*: **not clamped.** The drone's model calls are HTTPS from its
//!   own process tree — `--unshare-net` would sever the loop from its brain.
//!
//! One consequence to know: killing the wrapper mid-verb (a dropped
//! [`Stream`](crate::cli_outbound::Stream)) orphans the wrapped child — still
//! confined (its mount namespace persists), just unsupervised until it exits.
//! `--die-with-parent` is deliberately absent: it would tie every driver to
//! yog's own lifetime, which §8.1 forbids.

use std::path::{Path, PathBuf};

/// The Linux backend's program name, PATH-resolved at exec like every host
/// tool. Its presence on a box is the derived fact — never a stored one.
pub(crate) const BACKEND: &str = "bwrap";

/// The fixed sandbox shape, shared verbatim by the probe and the wrap so the
/// availability proven is the availability spent: the host tree read-only,
/// a minimal `/dev`, a fresh `/proc`, and the host `/tmp` writable.
const SHAPE: [&str; 10] = [
    "--ro-bind",
    "/",
    "/",
    "--dev",
    "/dev",
    "--proc",
    "/proc",
    "--bind",
    "/tmp",
    "/tmp",
];

/// The §4.11 item-8 birth gate, at both doors a drone is born through
/// (`dispatch::prompt` and the `Fork` arm): a workspace whose live policy
/// requires confinement fires nothing unless this platform's backend proves
/// itself right now. Absence of the policy line gates nothing (severability);
/// a refusal names the workspace, the policy file, and exactly why.
pub(crate) fn gate(workspace: &Path) -> Result<(), String> {
    if !super::policy::Policy::read(workspace).confinement_required {
        return Ok(());
    }
    available().map_err(|why| refusal(workspace, &why))
}

/// Whether confinement works *here*, derived at the asking: the platform's
/// backend, then the probe. `Err` carries the reason in the operator's terms.
pub(crate) fn available() -> Result<(), String> {
    probe(Path::new(backend_for(std::env::consts::OS)?))
}

/// The platform's one backend. Only Linux has one wired; every other OS is an
/// explicit refusal naming itself, which is what "platform-explicit" means —
/// no probe on a platform whose answer is already no.
fn backend_for(os: &str) -> Result<&'static str, String> {
    if os == "linux" {
        Ok(BACKEND)
    } else {
        Err(format!("no confinement backend is wired for {os}"))
    }
}

/// Run the backend over the exact [`SHAPE`] with a trivial command: exit 0 is
/// availability. A spawn failure (no binary) and a non-zero exit (a kernel
/// refusing user namespaces) are the two honest unavailabilities, each named.
fn probe(program: &Path) -> Result<(), String> {
    let mut args: Vec<&str> = SHAPE.to_vec();
    args.extend(["--", "true"]);
    let outcome = crate::actions::verbs::collect(crate::cli_outbound::Cli::new(program).run(&args))
        .map_err(|e| format!("its backend could not run: {e}"))?;
    if outcome.ok() {
        Ok(())
    } else {
        Err(format!(
            "its backend {} refused the sandbox shape (exit {}): {}",
            program.display(),
            outcome.exit,
            outcome.stderr.trim()
        ))
    }
}

/// The refusal, in the standing form the doors have always rendered: the
/// workspace, the policy line, the why, and the one lawful exit (delete the
/// line — policy is config, so removing it removes no code).
fn refusal(workspace: &Path, why: &str) -> String {
    format!(
        "{}: its capability policy declares `confinement: required`, and {why} — no drone is \
         fired. Remove the line from {} on config/default to fire without one.",
        workspace.display(),
        super::policy::CAPABILITY_YAML,
    )
}

/// The wrapper argv for a workspace-bound litany spawn — empty when the live
/// policy requires no confinement (the general path with empty inputs), the
/// full backend argv when it does. Read fresh per spawn: policy is the live
/// config tip, availability is the exec itself.
pub(crate) fn wrapper(world: &crate::xdg::Env, workspace: &Path) -> Vec<String> {
    if super::policy::Policy::read(workspace).confinement_required {
        argv(&writable(world, workspace))
    } else {
        Vec::new()
    }
}

/// The **derived writable set**: yog's own two places — the workspace and the
/// composed world root (§16.2, which is where the nested balls state, and so
/// every `work/<id>` checkout and every clone, already lives) — plus the bound
/// project repo when this workspace claimed one. `/tmp` is not here: it rides
/// [`SHAPE`], because it is a fact about the host and not about the birth.
///
/// **What is not there is not bound.** `bwrap` refuses a `--bind` whose source
/// is absent, so an orphaned project (§3.5's "project clone gone" — legitimate,
/// and leaving workspaces unaffected) would otherwise fail every birth in the
/// workspace that claimed it. One rule over the whole set rather than a case
/// for the one member that can vanish, and it can only ever *narrow* the set.
fn writable(world: &crate::xdg::Env, workspace: &Path) -> Vec<PathBuf> {
    [workspace.to_path_buf(), crate::world::layout(world).root]
        .into_iter()
        .chain(bound_project(world, workspace))
        .filter(|dir| dir.is_dir())
        .collect()
}

/// The **bound project repo** (bl-34b1) — the one member of the writable set
/// that is not one of yog's own places, and the one a ball-rung drone cannot
/// finish without: `bl close` advances a ref in the project it claimed from,
/// and the `work/<id>` checkout's gitdir lives inside that repo's
/// `.git/worktrees/`. Read-only, the rung's own delivery fails.
///
/// **Derived, never stored, and the same derivation at both doors.** It is the
/// §3.2 claimant join the §4.11 writable *root* already spends
/// ([`super::root::claimed`]): the last `bl claim <id> --as <name>` row on
/// yog's own ops trail, stamped with this workspace's leaf, whose `cwd` **is**
/// the project the claim ran in. A workspace encodes no project path (§3.5) and
/// a revived driver carries no payload — `litany message` and `litany advance`
/// reach [`wrapper`] with the workspace and nothing else — so a project taken
/// off a *birth parameter* would have confined every revival more tightly than
/// the fire it resumes. The trail is durable, so both doors derive the
/// identical set, with nothing carried and no field to go stale.
///
/// A workspace that never claimed through yog yields `None` — the general path
/// with an empty join — and so does a ball an agent claimed for *itself*
/// mid-conversation, which leaves no yog-side row (§3.2's stated limit): the
/// same bound the writable root already draws, in one place rather than two.
///
/// **A fact yog owns, not one yog trusts.** The trail lives under the world
/// root, which is itself writable here, so a forged claim row could name any
/// directory. That is the writable root's standing hazard, unchanged — a drone
/// that can write the trail already owns its own adjudication — and VISION
/// §4.11 item 8 scopes it out by construction: this layer bounds the
/// write-*accident* class. The invariant that holds is the root's own: no fact
/// the agent is *meant* to control is read here. No `cd` mark, no payload.
fn bound_project(world: &crate::xdg::Env, workspace: &Path) -> Option<PathBuf> {
    let entries = crate::opslog::tail(&world.yog_state_root(), usize::MAX);
    super::root::claimed(&entries, &crate::naming::leaf(workspace)).map(|(project, _)| project)
}

/// The backend argv: [`BACKEND`], the fixed [`SHAPE`], one writable bind per
/// member of the derived set, then `--` so the wrapped program can never be
/// read as a flag. Pure over its inputs.
fn argv(writable: &[PathBuf]) -> Vec<String> {
    let mut words: Vec<String> = std::iter::once(BACKEND)
        .chain(SHAPE)
        .map(str::to_owned)
        .collect();
    for dir in writable {
        let d: String = dir.to_string_lossy().into_owned();
        words.extend(["--bind".to_owned(), d.clone(), d]);
    }
    words.push("--".to_owned());
    words
}

#[cfg(test)]
mod tests;
