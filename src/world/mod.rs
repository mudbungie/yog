//! The nested world yog composes under its own data root (DESIGN §16.2) — the
//! pure `ambient Env → world Env` composition plus the `<yog-data-root>/world/`
//! subtree layout. yog reads the ambient environment once, anchors on
//! `$XDG_DATA_HOME/yog`, and layers a fixed three-var override set over that
//! snapshot; the composed result is itself an [`Env`], so every §5.1 fold
//! re-derives the *nested* location through it (balls state, lernie home, and
//! yog's own `ui.json`/`ops.jsonl`). Brazen's three folds are **not** here and
//! **not** ambient: since the blast-radius ruling they resolve
//! inside the focused workspace's wall, one layer further in ([`wall`]).
//! **This module is the pure
//! composition layer only** — it neither materializes the subtree nor wires the
//! world into any spawn (W2/W3); the submodules do the effectful halves
//! ([`seed`] the lernie home, [`tools`] the agent-tool shims).
//!
//! **The overrides, and why exactly these three (§16.2):**
//!
//! | Var | World value | Nests |
//! |---|---|---|
//! | `LERNIE_HOME` | `world/lernie` | lernie config **and** data (the `lernie_home` collapse) |
//! | `XDG_STATE_HOME` | `world/state` | balls clones/worktrees/op-logs **and** yog's `ui.json`/`ops.jsonl` |
//! | `PATH` | `world/tools:$PATH` | the tool an agent's bash *finds* — yog's own `bl`/`lernie`/`bz` shims, not host binaries (§16.7 W9/W11, [`tools`]) |
//!
//! The first two nest **state**; the third nests the **toolchain** — the same
//! encapsulation argument one layer up (§16.4: an ambient `bl` reads the right
//! paths by inheritance but is not yog's balls implementation). It is a prepend,
//! not a replacement: everything else on the operator's `PATH` still resolves.
//!
//! `XDG_DATA_HOME` alone is left ambient, and it is the world's **anchor**:
//! overriding it would recurse — re-deriving the anchor through the world `Env`
//! must yield the same path (§14). Nothing else is shared. Brazen's config,
//! credentials and model cache used to be, on the reasoning that one host `bz`
//! read them all; the blast-radius ruling reversed that (§16.2, §3.1's blast
//! radius) and they now resolve per workspace through [`wall`], whose one var
//! rides on top of this override set. Everything version-fragile (lernie's
//! home, balls' store layout) is nested here as before.
//!
//! **Design decision — yog's own artifacts move under the world (§16.2, not
//! ambiguous).** `ui.json`/`ops.jsonl` resolve through [`Env::yog_state_root`] =
//! `$XDG_STATE_HOME/yog`, so under the world `Env` they land at
//! `world/state/yog/`. The §16.2 `XDG_STATE_HOME` row ("… **and** yog's own
//! `ui.json`/`ops.jsonl`"), the severability clause ("… and yog's own
//! artifacts"), and §16.6 W1 ("its own two artifacts through the world `Env`")
//! all mandate the move — so yog's artifacts nest with the tools rather than
//! staying at the ambient yog roots, and one `rm -rf $XDG_DATA_HOME/yog` erases
//! the whole world including them.
//!
//! **Task 0 — bl-delivery worktree territory (§16.2 diligence), confirmed from
//! source _and_ empirically.** A child `bl` lands its worktrees under *its own
//! process* `$XDG_STATE_HOME`, so spawning it in the world `Env` (W2) nests
//! every worktree in `world/state`. Source: `balls@main:src/bin/bl-delivery.rs`
//! reads the live env — `Xdg::with(&home, env::var("XDG_CONFIG_HOME")…,
//! env::var("XDG_STATE_HOME")…)` — and `layout.rs::plugin_territory(name) =
//! state_home.join("balls").join("plugins").join(name)` feeds
//! `delivery_path.rs::binding_territory = plugin_territory(plugin).
//! join(invocation_path)`, whose `<id>` child is the worktree. So the worktree
//! is `$XDG_STATE_HOME/balls/plugins/<delivery>/<project-path>/<id>/`, rooted
//! entirely on the child's own `$XDG_STATE_HOME`. Empirically, this task's own
//! `bl claim` (ambient `$XDG_STATE_HOME` = `~/.local/state`) materialized its
//! worktree at `~/.local/state/balls/plugins/bl-delivery/home/u/dev/yog/
//! bl-c68f`. Env inheritance alone nests the worktrees; W2 threads the override
//! into the spawn.

use std::path::{Path, PathBuf};

use crate::xdg::Env;

/// The per-agent **balls space** (§16.3): which clone bundle and which balls
/// config home one agent's task tracking lives in, and the `YOG_MARKS` var that
/// carries it down a descent.
pub mod marks;

/// The per-workspace **wall** (§3.1, §16.2 as amended): the env layer that puts
/// brazen's config, credentials and model cache inside one workspace's sphere.
pub mod wall;

/// The two world escape hatches `yog env` / `yog exec` (§8.4, §16.6 W6) — the
/// human counterpart to §16.4's agent tools. Pure argv → plan; `main.rs`
/// dispatches (print, or spawn-and-exit) before eframe.
pub mod hatch;

/// The world's agent tools (§16.4, §16.7 W9): the `<world>/tools/` shim seeding
/// and the `PATH` entry that makes those shims what an agent's bash finds.
pub mod tools;

/// Which seat may open a window (§16.4, bl-3ff4): the guard that keeps the
/// world's `yog` shim from becoming an agent's way to paint on the operator's
/// desktop. See the module doc.
pub mod seat;

/// The `<yog-data-root>/world/` subtree (§16.2). Every path is computed from the
/// ambient anchor; nothing is stored. `root` backs materialization (W3); every
/// other field anchors an override [`compose`] layers into the world `Env` —
/// `lernie` → `LERNIE_HOME`, `state` → `XDG_STATE_HOME`, `tools` → the head of
/// `PATH` (§16.7 W9), which is also the dir the shim is seeded into.
pub struct Layout {
    /// `<yog-data-root>/world` — the subtree root; one `rm -rf` severs the world.
    pub root: PathBuf,
    /// `world/lernie` → `LERNIE_HOME` (lernie config **and** data).
    pub lernie: PathBuf,
    /// `world/state` → `XDG_STATE_HOME` (balls state **and** yog's artifacts).
    pub state: PathBuf,
    /// `world/tools` — the agent-tool shim territory (§16.4, §16.7 W9/W11): yog
    /// seeds `bl`/`lernie`/`bz` re-exec shims here ([`tools::ensure_shim`]) and
    /// puts the directory at the head of the world's `PATH`, so an agent's bare
    /// `bl` is yog's embedded balls.
    pub tools: PathBuf,
}

/// Compute the world layout from the ambient env's data-root anchor
/// ([`Env::yog_data_root`] = `$XDG_DATA_HOME/yog`). Pure; no IO. Delegates to
/// [`layout_under`], the `Env`-free core.
pub fn layout(ambient: &Env) -> Layout {
    layout_under(&ambient.yog_data_root())
}

/// The [`Env`]-free core [`layout`] delegates to: the world subtree under a
/// yog data-root anchor path (§16.2), pure path algebra. Seeding (W3) and the
/// start flow derive the world layout straight from `PlanInputs::yog_data_root`
/// through here, without re-snapshotting the process env into an [`Env`].
pub fn layout_under(yog_data_root: &Path) -> Layout {
    let root = yog_data_root.join("world");
    Layout {
        lernie: root.join("lernie"),
        state: root.join("state"),
        tools: root.join("tools"),
        root,
    }
}

/// `LERNIE_HOME` — nests lernie config **and** data onto [`Layout::lernie`].
const LERNIE_HOME: &str = "LERNIE_HOME";
/// `XDG_STATE_HOME` — nests balls state **and** yog's artifacts onto
/// [`Layout::state`], and is re-pointed one layer in by [`inhabit_space`] when
/// an agent's own §16.3 space is the balls state this process addresses.
const XDG_STATE_HOME: &str = "XDG_STATE_HOME";
/// `PATH` — puts [`Layout::tools`] in front of the ambient search path, so an
/// agent's bare `bl` is the world's shim (§16.7 W9, [`tools::prepend_path`]).
const PATH: &str = "PATH";

/// The world's fixed override set (§16.2) as `(var, nested-value)` pairs — the
/// **single source of truth** for which vars nest and to what, consumed
/// both by [`compose`] (folded into the world `Env` so every §5.1 read nests)
/// and by every world spawn (layered onto each child through
/// [`Cli::resolve_in_world`](crate::cli_outbound::Cli::resolve_in_world), W2).
/// Reads-derive-through-compose and spawns-inherit-these are therefore one fact:
/// the dir yog watches and the dir a spawned `bl` writes are the same path.
/// The set is **workspace-free by construction**: everything here is a pure
/// function of the yog data-root anchor, so one world serves every sphere. What
/// is per-workspace rides one layer in ([`wall::pairs`]), layered onto a
/// workspace-bound spawn on top of these.
///
/// **Idempotent under re-composition.** Every value is a pure function of the
/// yog data-root anchor, which the world leaves ambient — so applying this set
/// to a `Env` that already carries it reproduces it exactly. The `PATH` prepend
/// carries that property explicitly ([`tools::prepend_path`]), which is what
/// lets `marks`/`config_edit` re-derive the overrides from the **world** `Env`
/// (not the ambient one) without stacking a second tools entry.
pub fn overrides(ambient: &Env) -> Vec<(String, String)> {
    let l = layout(ambient);
    vec![
        (
            LERNIE_HOME.to_owned(),
            l.lernie.to_string_lossy().into_owned(),
        ),
        (
            XDG_STATE_HOME.to_owned(),
            l.state.to_string_lossy().into_owned(),
        ),
        (
            PATH.to_owned(),
            tools::prepend_path(&l.tools, ambient.search_path()),
        ),
    ]
}

/// **Stand this process in the world** (§16.2, bl-81c9): apply [`overrides`] to
/// the process's OWN environment, so a caller that cannot be handed an [`Env`]
/// still resolves the nested roots.
///
/// [`compose`] is the world as a *value* — enough for every yog fold, and for
/// every child yog spawns (the overrides ride the `Command`). It is not enough
/// for an **in-process substrate arm** (§16.7): `yog bl` hands balls an `Edge`
/// but balls' plugin chain spawns children that resolve `$XDG_STATE_HOME`
/// through their own `getenv`, and the linked lernie reads `LERNIE_HOME` with
/// no injection seam at all. For those the world has to *be* the environment —
/// so the arm folds it in once, at the process edge, and every read and every
/// descendant follows. A bare `yog bl` / `yog lernie` typed at an ambient shell
/// is then the same world `yog exec bl …` hands out, which is what the
/// namespaces advertise.
///
/// **Idempotent, by the same property [`overrides`] has:** every value is a
/// pure function of the ambient anchor, which the world never overrides, and
/// the `PATH` prepend recognizes its own entry — so re-entry (the spawned case,
/// where the parent already folded) reproduces the identical set and stacks
/// nothing. That dissolves the "already composed" case rather than testing for
/// it.
pub(crate) fn inhabit() {
    crate::cli_outbound::sys::set_env(&overrides(&Env::from_env()));
}

/// **Stand this process in a §16.3 space** (bl-c21d): re-point
/// [`XDG_STATE_HOME`] at the space's own state home, layered on [`inhabit`]'s
/// override set exactly as [`marks::pairs`] and [`wall::pairs`] layer onto it
/// for a spawn — one layer in, last write wins.
///
/// It is [`inhabit`]'s ruling one layer down, and it exists for the identical
/// reason. balls folds its clone bundle **and** its plugin territories
/// (`bl-delivery`'s `work/<id>` worktrees, `bl-tracker`'s mirror) and its
/// attempts tree off ONE fact — its state home — so a host that supplies that
/// fact through an [`Edge`](balls::edge::Edge) alone supplies it to the linked
/// crate and to nothing balls spawns: `bl-delivery` rebuilds its own
/// `balls::layout::Xdg` from `$XDG_STATE_HOME` in its process env and holds no
/// `Edge` at all. So an own space kept its store and lost its worktrees, into
/// whatever state home the process happened to stand in.
///
/// **Unconditional, and that is the point:** an absent `YOG_MARKS` resolves the
/// *world's* space, whose state home is the value [`inhabit`] just wrote — so the
/// fold is a no-op there rather than a case, and there is no "own or not" branch
/// to keep in step with [`marks::space`].
pub(crate) fn inhabit_space(space: &marks::Space) {
    crate::cli_outbound::sys::set_env(&[(
        XDG_STATE_HOME.to_owned(),
        space.state.to_string_lossy().into_owned(),
    )]);
}

/// Compose the world `Env`: the ambient snapshot plus the nesting
/// [`overrides`] (§16.2). `XDG_DATA_HOME` is left ambient — it is the anchor,
/// and nothing else is shared. The result is itself an [`Env`]; pass it to any
/// §5.1 fold to derive the nested location, or through [`wall::env`] first for
/// the folds that live inside one workspace's sphere.
pub fn compose(ambient: &Env) -> Env {
    let ov = overrides(ambient);
    let slice: Vec<(&str, &str)> = ov.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    ambient.with_overrides(&slice)
}

pub mod seed;

#[cfg(test)]
mod tests;
