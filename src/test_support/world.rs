//! The **fixture world** (DESIGN §16.2): the hermetic composed env every test
//! that touches a §9 destination, a §16.3 space or a brazen fold reads and
//! writes through, plus the workspace whose wall it stands in.
//!
//! Its own file at §12's cap, on a real seam: everything in [`super`] fakes an
//! *effect* (a clock, a filesystem, a fixture writer), while this composes a
//! *world* — a set of paths derived by the same production folds the executors
//! use, so a fixture can never write where nothing reads (bl-fcd5).

use std::path::{Path, PathBuf};

/// A hermetic composed world (§16.2) rooted under `root`, **with a wall
/// standing**: every fold a §9 destination resolves through — brazen's three
/// wall locations, litany's config root, yog's state (and so the staging root),
/// `$HOME` — lands inside it, so a test that drives a config gesture writes
/// only into its own tempdir. The wall is set because brazen is unreachable
/// without one (§16.2 as amended): a fixture with no wall is a fixture whose
/// `bz` refuses, which is [`no_wall`]'s job to say deliberately.
pub(crate) fn world_under(root: &Path) -> crate::xdg::Env {
    let at = |sub: &str| root.join(sub).display().to_string();
    let base = crate::xdg::Env::from_pairs([
        ("HOME", at("home")),
        ("XDG_CONFIG_HOME", at("config")),
        ("XDG_DATA_HOME", at("data")),
        ("XDG_STATE_HOME", at("state")),
        ("XDG_CACHE_HOME", at("cache")),
        ("LITANY_HOME", at("litany")),
        // Deliberately pre-set and deliberately inert: brazen's config is the
        // wall's (§16.2 as amended), so a fixture that leaves this standing
        // proves an ambient value never wins.
        ("BRAZEN_CONFIG", at("brazen.toml")),
    ]);
    // The wall is folded by the **production** lens, off [`fixture_workspace`]
    // (bl-fcd5): a §9 gesture now names its workspace and the executor derives
    // the wall from that name, so a fixture whose standing `YOG_WALL` were
    // some other dir would write where nothing reads. One fold, one wall.
    crate::world::wall::env(&base, &fixture_workspace())
}

/// The workspace every §9 fixture names in its gesture. Its **leaf** is what
/// the wall fold keys on (§3.1), so this path and [`wall_paths`] resolve the
/// same three brazen files by construction.
pub(crate) fn fixture_workspace() -> PathBuf {
    PathBuf::from("/ws")
}

/// The brazen locations a [`world_under`] fixture writes and reads (§16.2 as
/// amended) — the [`fixture_workspace`]'s own wall, derived exactly as the
/// executor derives it.
pub(crate) fn wall_paths(root: &Path) -> crate::config_edit::brazen::BrazenPaths {
    crate::config_edit::brazen::BrazenPaths::in_wall(&crate::world::wall::root_of(
        &world_under(root),
        &fixture_workspace(),
    ))
}

/// A hermetic world with **no** wall — a seat inside no workspace, where every
/// brazen fold answers `None` and `bz` refuses (§16.2 as amended).
pub(crate) fn no_wall(root: &Path) -> crate::xdg::Env {
    world_under(root).without(crate::world::wall::YOG_WALL)
}

/// The world every fixture that never touches a §9 destination hands over: a
/// hermetic one under a path that does not exist, so an unexpected config
/// write fails loudly instead of reaching the operator's real home.
pub(crate) fn no_world() -> crate::xdg::Env {
    world_under(Path::new("/nonexistent/yog-test-world"))
}
