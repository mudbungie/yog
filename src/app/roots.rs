//! The enumeration roots one instance watches and walks (§7.1) — the boot-time
//! fold of the composed world, and the four derived paths every root read goes
//! through. Its own file per §12's line budget: `app/mod.rs` is the model, and
//! this is the address book it was built from.

use crate::binding;
use std::path::PathBuf;

/// The enumeration roots the model watches and walks (§7.1): yog's own flat
/// names root, the litany data root (foreign workspaces + replays), and the yog
/// state root (`ui.json`). Built once from [`crate::xdg::Env`] by the shell.
#[derive(Debug, Clone)]
pub struct Roots {
    pub yog_data: PathBuf,
    pub litany_data: PathBuf,
    pub yog_state: PathBuf,
    /// The balls per-project clones dir (`$XDG_STATE_HOME/balls/clones/`, §5.1
    /// #1) — project enumeration and the `BallsClones` watch (§7.1).
    pub balls_clones: PathBuf,
    /// The operator's home dir (`~`, §3.4) — the bare rung's driver cwd.
    pub home: PathBuf,
    /// The composed world (§16.2) the four roots above were folded from at
    /// boot, kept beside them because the §8.5 config family
    /// ([`Action::ApplyConfig`](crate::boundary::Action::ApplyConfig) and its
    /// two siblings) folds *destinations* rather than roots — brazen's
    /// `config.toml`, litany's config root, the staging root — and asks the
    /// **linked** brazen for its provider table through the same snapshot. It
    /// is the source of the four, never a second copy of them: nothing reads a
    /// root back off it.
    pub world: crate::xdg::Env,
}

impl Roots {
    /// Fold the roots out of a composed world (§16.2) — the one derivation, so
    /// the window and the windowless face cannot address different trees. Pure
    /// path arithmetic over the snapshot; nothing reads disk.
    pub fn of(world: &crate::xdg::Env) -> Roots {
        Roots {
            yog_data: world.yog_data_root(),
            litany_data: world.litany_data_root(),
            yog_state: world.yog_state_root(),
            balls_clones: world.balls_clones_dir(),
            home: world.home_dir(),
            world: world.clone(),
        }
    }

    /// yog's flat names root (`$XDG_DATA_HOME/yog/workspaces/`, §3.1).
    pub(super) fn names(&self) -> PathBuf {
        binding::names_root(&self.yog_data)
    }
    pub(crate) fn workspaces(&self) -> PathBuf {
        self.litany_data.join("workspaces")
    }
    pub(super) fn replays(&self) -> PathBuf {
        self.litany_data.join("replays")
    }
    pub(super) fn ui_json(&self) -> PathBuf {
        self.yog_state.join("ui.json")
    }
}

#[cfg(test)]
mod tests;
