//! **Where a laid state's pieces go** — the path arithmetic, and nothing that
//! touches disk.
//!
//! Its own file on a real seam, the one `test_support::world` was cut on
//! (bl-fcd5): everything in [`super::lay`] and [`super::disk`] performs an
//! *effect*, while this composes a *set of paths* — every one derived by the
//! production folds an engine reads them back with, so a fixture can never
//! write where nothing reads.

use std::path::{Path, PathBuf};

/// Where a laid state's pieces are — computed once so the writer and the
/// [`super::Laid`] the caller is handed cannot disagree about one path.
/// The `root` this is folded from is the caller's own — the `XDG_DATA_HOME` an
/// engine is booted with — and is deliberately **not** a field: it is the
/// caller's value, and a copy here would be one fact with two homes.
pub struct Places {
    /// `<root>/yog` — the anchor every fold below hangs off.
    pub data: PathBuf,
    /// `<root>/yog/world/litany` — `LITANY_HOME`, holding the seed marker.
    pub litany: PathBuf,
    /// `<root>/yog/workspaces` — yog's flat **names** root (§3.1). A fixture
    /// lays here and not under the litany home: a leaf here is a `Named`
    /// workspace, the only kind that carries a claimant and can bind a ball,
    /// and a leaf under litany's own `workspaces/` enumerates as `Foreign`.
    pub names: PathBuf,
    /// `<root>/yog/world/state/yog` — the yog state root, holding the client
    /// registry and `cadence.yaml`.
    pub state: PathBuf,
    /// `<root>/yog/world/walls` — the per-workspace brazen layer (§16.2).
    pub walls: PathBuf,
    /// `<root>/yog/wire` — the key material, beside the world and not in it.
    pub wire: PathBuf,
}

impl Places {
    /// Fold every place out of one root, by the production folds.
    pub fn under(root: &Path) -> Places {
        let data = root.join("yog");
        let layout = crate::world::layout_under(&data);
        Places {
            litany: layout.litany.clone(),
            state: layout.state.join("yog"),
            names: crate::binding::names_root(&data),
            walls: crate::world::wall::walls_dir(&layout.root),
            wire: data.join(crate::wire::material::DIR),
            data,
        }
    }

    /// One **named** workspace's directory (§3.1: the leaf IS the name).
    pub fn workspace(&self, name: &str) -> PathBuf {
        self.names.join(name)
    }
}
