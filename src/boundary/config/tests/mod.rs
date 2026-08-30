//! The §9 config family's executors (bl-3f46), driven end to end against a
//! hermetic world: a real `config.toml` validated by the **linked** brazen, a
//! real `models.yaml` written unjudged (bl-3ffa), a recorder `litany`/`bl` for
//! the two spawning halves, and a real-git workspace for the §9.4 pick.
//!
//! Nothing here mocks a pipeline — every case writes the file the gesture
//! claims to write, or refuses for the reason the pipeline refuses. This file
//! is the shared world; [`files`] drives the destinations and [`knobs`] the
//! §16.3 knob and the §9.4 pick.

use crate::boundary::answer::answer;
use crate::boundary::config::ConfigFile;
use crate::boundary::dispatch::{Deps, dispatch};
use crate::boundary::reply::Reply;
use crate::boundary::tests::snapshot;
use crate::boundary::{Action, Query};
use crate::cli_outbound::Cli;
use crate::test_support::world_under;
use crate::ui_state::UiState;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod browse;
mod files;
mod knobs;
mod reads;

/// A brazen config declaring one row, so the effective table has a name this
/// suite can pick and gate on.
pub(super) const ACME: &str = "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\n\
                               base_url = \"https://acme.test\"\nauth = \"none\"\n";

/// A hermetic world under `root` with brazen's config written, the litany
/// config root and yog's state root created, and both verb binaries injected.
pub(super) fn deps_at(root: &Path, litany: &Path, bl: &Path) -> Deps {
    let world = world_under(root);
    fs::create_dir_all(root.join("litany/workflows")).unwrap();
    fs::create_dir_all(world.yog_state_root()).unwrap();
    // brazen's config is the **wall's** now (§16.2 as amended), so the fixture
    // writes where the focused workspace's fold reads.
    let brazen = crate::test_support::wall_paths(root).config;
    fs::create_dir_all(brazen.parent().expect("the wall's brazen dir")).unwrap();
    fs::write(&brazen, ACME).unwrap();
    Deps {
        litany: Cli::new(litany),
        bl: Cli::new(bl),
        state_root: world.yog_state_root(),
        yog_binary: root.join("yog"),
        world,
        home: root.join("home"),
        yog_data_root: root.join("data/yog"),
        balls_state_root: root.join("state/balls"),
        snapshot: Arc::new(snapshot(Path::new("/ws"), "alba", vec![], vec![])),
        caller: crate::boundary::dispatch::Caller::default(),
    }
}

/// The same deps with `workspaces` **enumerated** (REMOTE §8, bl-f5f6): a
/// gesture addresses a sphere by NAME, and a name resolves only against the
/// workspace set the snapshot publishes — so a fixture acting on some other
/// sphere must publish it, exactly as the worker publishes what it found on
/// disk. Nothing else about the deps changes.
pub(super) fn seeing(deps: &Deps, workspaces: &[&Path]) -> Deps {
    let mut snap = (*deps.snapshot).clone();
    snap.workspaces
        .extend(workspaces.iter().map(|path| crate::binding::Workspace {
            path: (*path).to_path_buf(),
            kind: crate::binding::WorkspaceKind::Named {
                name: crate::naming::leaf(path),
            },
        }));
    Deps {
        snapshot: Arc::new(snap),
        ..deps.clone()
    }
}

/// The common case: no spawn expected, so both binaries are unspawnable.
pub(super) fn quiet(root: &Path) -> Deps {
    deps_at(
        root,
        Path::new("/definitely/not/a/litany-xyz"),
        Path::new("/definitely/not/a/bl-xyz"),
    )
}

/// An executable `#!/bin/sh` script. The caller holds `SPAWN_LOCK` across
/// write + spawn (the ETXTBSY discipline).
pub(super) fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

/// Fire one action through the real chokepoint, so what is tested is the
/// gesture and not a private helper.
pub(super) fn fire(deps: &Deps, action: &Action) -> Result<Reply, String> {
    let mut ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    dispatch(deps, &mut ui, "T0", action)
}

/// Ask one query through the real chokepoint (bl-0164) — [`fire`]'s read-only
/// twin, so a config read is tested through the same `answer` a deposit and
/// the frame both call, not a private helper.
pub(super) fn ask(deps: &Deps, query: &Query) -> Result<Reply, String> {
    let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    answer(query, deps, &ui, 0)
}

/// Write a brazen config into `workspace`'s **own** wall — where the executor
/// reads once a gesture names that sphere (bl-fcd5). `deps_at` seeds the
/// [`fixture_workspace`](crate::test_support::fixture_workspace)'s; a test
/// that acts on some other workspace seeds that one with this.
pub(super) fn seed_wall(deps: &Deps, workspace: &Path, text: &str) {
    let config = crate::config_edit::brazen::BrazenPaths::in_wall(&crate::world::wall::root_of(
        &deps.world,
        workspace,
    ))
    .config;
    fs::create_dir_all(config.parent().expect("the wall's brazen dir")).unwrap();
    fs::write(&config, text).unwrap();
}

/// The brazen destination, naming the fixture's own workspace — the wall the
/// fixture writes into (bl-fcd5: the gesture carries its sphere).
pub(super) fn brazen_file() -> ConfigFile {
    ConfigFile::Brazen {
        workspace: crate::naming::leaf(&(crate::test_support::fixture_workspace())),
    }
}

pub(super) fn applying(file: ConfigFile, text: &str) -> Action {
    Action::ApplyConfig {
        file,
        text: text.to_owned(),
    }
}
