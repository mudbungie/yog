//! **Where one config gesture lands** (§9): the [`ConfigFile`] destination, the
//! workspace it names, and that name located. Its own file at §12's cap
//! (bl-f5f6) on the seam the `action` and `query` rosters are already cut on: a
//! *destination* is a datum every seat constructs, and the pipelines that spend
//! it are [`super`]'s. The addressing rides with the datum because only the
//! destination can answer which sphere it is for.

use super::Deps;
use crate::config_edit::branch::edit::EditOrigin;
use std::path::PathBuf;

/// Where one [`ApplyConfig`](crate::boundary::Action::ApplyConfig) lands (§9). The
/// destination decides the pipeline, so the gesture carries no mode flag: a
/// brazen draft is `bz`-validated, a lernie-global one is provider-gated, and a
/// lineage file is committed by `lernie config`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFile {
    /// One workspace's own brazen `config.toml` (§9.1, §16.2's wall) —
    /// validated by the linked `bz` before it lands, because `bz` is the only
    /// lawful parser of that schema.
    ///
    /// **It names its workspace, exactly as [`Branch`](ConfigFile::Branch)
    /// does** (bl-fcd5). Since the blast-radius ruling this file lives inside a
    /// wall, so the destination is not "brazen's config" but "*this sphere's*
    /// brazen config" — a fact the gesture must carry, because a headless seat
    /// has no focus to derive it from and a teleoperator could otherwise not
    /// reach provider config at all. The seat states it (`--ws`, or the
    /// envelope's `workspace`); the window fills it from focus; a gesture that
    /// names neither is refused where every other missing target is, at the
    /// edge.
    Brazen { workspace: String },
    /// lernie's global `models.yaml` (§9.2).
    LernieModels,
    /// One `workflows/<name>.yaml` (§9.2). The name must be a safe single-file
    /// basename; anything else is refused before a byte is staged.
    LernieWorkflow { name: String },
    /// yog's own `cadence.yaml` — the clock's periods (§7.2), on the same §9
    /// editor discipline because the file is a file.
    Cadence,
    /// One file on a per-workspace config lineage (§9.3): staged, then handed
    /// to `lernie config`, whose `$EDITOR` callback re-enters this binary and
    /// copies it over the checkout. lernie commits; yog never writes inside a
    /// workspace.
    Branch {
        workspace: String,
        /// The lineage's bare name — `lernie config <ws> <lineage>`.
        lineage: String,
        /// Advance it, fork it off a source, or start a fresh orphan (§9.3).
        origin: EditOrigin,
        /// The checkout-relative path this text is.
        path: String,
    },
}

impl ConfigFile {
    /// The workspace this destination names (REMOTE §8), or `None` for the
    /// three that name no world — lernie's globals and yog's own cadence file.
    /// The §9 family's own half of [`Action::workspace`](crate::boundary::Action), where
    /// it belongs: the destination decides, and only it can say.
    pub fn workspace(&self) -> Option<String> {
        match self {
            ConfigFile::Brazen { workspace } | ConfigFile::Branch { workspace, .. } => {
                Some(workspace.clone())
            }
            ConfigFile::LernieModels | ConfigFile::LernieWorkflow { .. } | ConfigFile::Cadence => {
                None
            }
        }
    }
}

/// The wall a destination names, **located** (REMOTE §8, bl-f5f6) — the empty
/// path for the three that name none, which is the general path with no input:
/// no arm that takes it reads this value. The single home of "which workspace
/// is this config file's", so the write, the read and
/// [`Action::workspace`](crate::boundary::Action) cannot disagree.
pub(super) fn located(deps: &Deps, file: &ConfigFile) -> Result<PathBuf, String> {
    match file.workspace() {
        Some(name) => deps.snapshot.ws_path(&name),
        None => Ok(PathBuf::new()),
    }
}
