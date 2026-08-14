//! The per-workspace **wall** (DESIGN §3.1, §16.2 as amended by the
//! blast-radius ruling) — the env layer yog composes *on top of* the world
//! (§16.2) for a workspace-bound read or spawn.
//!
//! The world nests yog's substrate away from the operator's ambient tools; the
//! wall nests one workspace's **settings** away from every other workspace's.
//! The ruling, verbatim (§3.1): *"workspaces are an entirely separate space;
//! essentially an app-wide blast radius. Different sets of conversations,
//! settings, providers, all of it."* Brazen's three folds — config,
//! credentials, model cache (§5.1 #19/#22/#23) — resolve here and nowhere else.
//!
//! **One var carries it: `YOG_WALL`.** It names the wall root, and every other
//! per-workspace location is derived from it ([`BrazenPaths`]) — one fact, one
//! home, no second var to drift. Setting it on a workspace-bound spawn is
//! enough for the whole descendant tree: the fired `lernie` loop inherits it,
//! lernie hands its own environment to every tool subprocess it spawns
//! (lernie ARCH §3.3), and a bare `bz` in an agent's bash is the world's shim
//! re-entering yog (§16.7 W9/W12) — which folds the wall back out of its own
//! process env. So the wall is set once, at the edge that knows the workspace,
//! and no downstream seat has to be told.
//!
//! **Why a yog-owned var and not brazen's own knobs.** `BRAZEN_CONFIG` selects
//! only the config file; credentials and the model cache fold off
//! `$XDG_DATA_HOME` / `$XDG_CACHE_HOME`, and `$XDG_DATA_HOME` is the world's
//! anchor — overriding it would recurse (§14). One var that names the wall
//! root, plus yog's own store seams reading it ([`crate::bz_host::store`]),
//! moves all three with no anchor and no second code path.
//!
//! **The name is the wall** (§3.1: *"the name names this wall and nothing
//! else — it is the dir leaf and the ball claimant"*), so the wall root is
//! keyed by the workspace's leaf. Leaves are unique across the three roots by
//! construction (§3.1 refuses a name equal to an existing leaf under any of
//! them), so a foreign or replay workspace gets its own wall by the same fold.

use std::path::{Path, PathBuf};

use crate::xdg::Env;

/// The one var naming the focused workspace's wall root (§16.2 as amended).
/// Absent = no wall: a process that is inside no workspace, which has no
/// providers to read (nothing is ambient but the roster).
pub const YOG_WALL: &str = "YOG_WALL";

/// The walls dir under a world root: `<world>/walls`. One leaf per workspace,
/// so §3.6's deletion takes the sphere's settings down with the sphere
/// ([`crate::delete`]).
pub fn walls_dir(world_root: &Path) -> PathBuf {
    world_root.join("walls")
}

/// The wall root of the workspace named `name`: `<world>/walls/<name>`. Pure
/// path algebra over the world layout ([`super::layout`]); no IO, nothing
/// stored — the wall is a query on the workspace's name, like every other
/// §5.1 fact.
pub fn root(world: &Env, name: &str) -> PathBuf {
    walls_dir(&super::layout(world).root).join(name)
}

/// The [`Env`]-free core: the wall root under a world root. The start flow and
/// §3.6's deletion both hold the anchor without an `Env` to re-snapshot.
pub fn root_under(world_root: &Path, name: &str) -> PathBuf {
    walls_dir(world_root).join(name)
}

/// The wall root of the workspace at `workspace`, whose §3.1 leaf names it.
pub fn root_of(world: &Env, workspace: &Path) -> PathBuf {
    root(world, &crate::naming::leaf(workspace))
}

/// The wall's spawn layer: the `(var, value)` pairs a workspace-bound child
/// gets **on top of** the world's own [`overrides`](super::overrides). Layered
/// through [`Cli::and_env`](crate::cli_outbound::Cli::and_env), the seam the
/// start flow's `YOG_NAME` stamp already rides (§8.1).
pub fn pairs(world: &Env, workspace: &Path) -> Vec<(String, String)> {
    vec![(
        YOG_WALL.to_owned(),
        root_of(world, workspace).to_string_lossy().into_owned(),
    )]
}

/// The wall standing in `env`, back as the pairs a spawn needs — the inverse of
/// [`pairs`] for a seat that already holds the lensed `Env` rather than the
/// workspace path. Empty outside a wall, which layers nothing.
pub fn pairs_of(env: &Env) -> Vec<(String, String)> {
    env.wall()
        .map(|w| vec![(YOG_WALL.to_owned(), w.to_string_lossy().into_owned())])
        .unwrap_or_default()
}

/// The wall's read lens: the world `Env` with this workspace's wall standing,
/// so every brazen fold through it resolves inside that sphere. Idempotent —
/// the value is a pure function of the anchor and the name, so re-lensing an
/// `Env` that already carries a wall replaces it rather than stacking.
pub fn env(world: &Env, workspace: &Path) -> Env {
    world.with_overrides(&[(YOG_WALL, &root_of(world, workspace).to_string_lossy())])
}

/// The wall's read lens for an *optional* focus — the shape every surface that
/// folds from the focused workspace has (§11: a workspace may not be focused,
/// and the empty world has none at all). No focus means no wall, which is the
/// general path with an empty input rather than a special case: the brazen
/// folds answer `None` and the surface renders its guard.
pub fn env_opt(world: &Env, workspace: Option<&Path>) -> Env {
    match workspace {
        Some(ws) => env(world, ws),
        None => world.without(YOG_WALL),
    }
}

#[cfg(test)]
mod tests;
