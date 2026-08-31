//! **Where one config gesture lands** (§9): the [`ConfigFile`] destination and
//! the workspace it names. Its own file at §12's cap (bl-f5f6) on the seam the
//! `action` and `query` rosters are already cut on: a *destination* is a datum
//! every seat constructs, and the pipelines that spend it are [`super`]'s. The
//! addressing rides with the datum because only the destination can answer
//! which sphere it is for — and since bl-523f that answer *is* the gesture's
//! address, read by the one workspace table
//! ([`address::workspace`](crate::boundary::address)) and **resolved once at
//! each chokepoint** like every other gesture's, never a second time here.

use crate::config_edit::branch::edit::EditOrigin;

/// Where one [`ApplyConfig`](crate::boundary::Action::ApplyConfig) lands (§9). The
/// destination decides the pipeline, so the gesture carries no mode flag: a
/// brazen draft is `bz`-validated, a litany-global one is provider-gated, and a
/// lineage file is committed by `litany config`.
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
    /// litany's global `models.yaml` (§9.2).
    LitanyModels,
    /// One `workflows/<name>.yaml` (§9.2). The name must be a safe single-file
    /// basename; anything else is refused before a byte is staged.
    LitanyWorkflow { name: String },
    /// yog's own `cadence.yaml` — the clock's periods (§7.2), on the same §9
    /// editor discipline because the file is a file.
    Cadence,
    /// One file on a per-workspace config lineage (§9.3): staged, then handed
    /// to `litany config`, whose `$EDITOR` callback re-enters this binary and
    /// copies it over the checkout. litany commits; yog never writes inside a
    /// workspace.
    Branch {
        workspace: String,
        /// The lineage's bare name — `litany config <ws> <lineage>`.
        lineage: String,
        /// Advance it, fork it off a source, or start a fresh orphan (§9.3).
        origin: EditOrigin,
        /// The checkout-relative path this text is.
        path: String,
    },
}

impl ConfigFile {
    /// **The destination's row of the one workspace table** (REMOTE §8, §8.2;
    /// bl-523f): the wall this config gesture is aimed at, or `None` for the
    /// three destinations that name no world — litany's globals and yog's own
    /// cadence file. The §9 family's own half of
    /// [`Action::workspace`](crate::boundary::Action), where it belongs: the
    /// destination decides, and only it can say.
    ///
    /// Borrowed rather than read, for that table's reason exactly: the §8.2
    /// rewrite is spent at the channel boundary over *whatever* field names the
    /// gesture's workspace, and this family's is nested one level down, inside
    /// `target`. Until this row existed a config act aimed at a workspace held
    /// on another box under a local rename resolved to no entry, fell through
    /// to the local engine, and edited the local wall's file.
    pub(crate) fn workspace_slot(&mut self) -> Option<&mut String> {
        match self {
            ConfigFile::Brazen { workspace } | ConfigFile::Branch { workspace, .. } => {
                Some(workspace)
            }
            ConfigFile::LitanyModels | ConfigFile::LitanyWorkflow { .. } | ConfigFile::Cadence => {
                None
            }
        }
    }
}
