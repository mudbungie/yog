//! The **OS confinement backend** (VISION §4.11 item 8, DESIGN §8.6): the one
//! platform-explicit layer a `confinement: required` workspace spends, wired
//! at yog's own lernie spawn seam.
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
//! whose live policy requires confinement has *every* workspace-bound lernie
//! spawn prefixed with the backend argv, probe or no probe. A backend that
//! vanishes between gate and spawn therefore fails the spawn loudly (the exec
//! names `bwrap`) instead of quietly running bare — fail-closed by
//! construction. Severable the other way too: no `confinement:` line is an
//! empty wrapper and a byte-identical spawn.
//!
//! **The support boundary — what the shape clamps and what it leaves.**
//! - *Filesystem writes*: the whole host tree is re-bound read-only; writable
//!   is exactly the derived set — the workspace, the composed world root
//!   (§16.2: lernie home, nested balls state, walls, tools), and the host
//!   `/tmp` (world-writable on the host by design; a private tmpfs would break
//!   cross-process temp coordination for nothing). A project repo outside the
//!   world is **read-only** to a confined drone — its own `bl close` cannot
//!   deliver; that is a stated v1 bound, not an accident.
//! - *Process access*: **not clamped.** A pid namespace dies with its init,
//!   and lernie's short verbs detach-launch drivers that must outlive them
//!   (its ARCH §2.9), so `--unshare-pid` would kill every revived driver at
//!   the verb's exit; `lernie stop` also signals by host pid. The capability
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

use std::path::Path;

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

/// The wrapper argv for a workspace-bound lernie spawn — empty when the live
/// policy requires no confinement (the general path with empty inputs), the
/// full backend argv when it does. Read fresh per spawn: policy is the live
/// config tip, availability is the exec itself.
pub(crate) fn wrapper(world: &crate::xdg::Env, workspace: &Path) -> Vec<String> {
    if super::policy::Policy::read(workspace).confinement_required {
        argv(&crate::world::layout(world).root, workspace)
    } else {
        Vec::new()
    }
}

/// The backend argv: [`BACKEND`], the fixed [`SHAPE`], one writable bind per
/// member of the derived set (the workspace, the world root), then `--` so the
/// wrapped program can never be read as a flag. Pure over its inputs.
fn argv(world_root: &Path, workspace: &Path) -> Vec<String> {
    let mut words: Vec<String> = std::iter::once(BACKEND)
        .chain(SHAPE)
        .map(str::to_owned)
        .collect();
    for dir in [workspace, world_root] {
        let d: String = dir.to_string_lossy().into_owned();
        words.extend(["--bind".to_owned(), d.clone(), d]);
    }
    words.push("--".to_owned());
    words
}

#[cfg(test)]
mod tests;
