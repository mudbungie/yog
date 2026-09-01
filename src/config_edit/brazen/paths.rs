//! brazen's on-disk layout **inside one workspace's wall** (DESIGN §5.1 rows
//! 19/22/23, §16.2 as amended), and the two reads that need nothing else.
//!
//! Split off [`super`] at §12's pre-split band on a seam that module's own
//! prose already drew: [`model_cache_at`] is documented there as *free of the
//! editor* — the §9.4 pick asks it holding no draft and no Apply pipeline. A
//! layout plus the questions answerable from a layout alone is one subject; the
//! staged-validation editor is another. (The credential-presence read that
//! stood beside it went in bl-dba3: it was a second derivation of brazen's own
//! `credential` column, blind to every spelling but `stored`.)

use super::FileIo;
use crate::xdg::Env;
use std::path::{Path, PathBuf};

/// brazen's three locations **inside one workspace's wall** (§5.1 rows
/// 19/22/23, §16.2 as amended): the layout is yog's own, so it is the same
/// three leaves on every §10 target and there is no per-OS branch left. This
/// struct is that layout's single home — [`crate::bz_host`] hands the very
/// same paths to the linked brazen's seams, so the file yog's editor writes
/// and the file `bz` reads cannot be two different files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrazenPaths {
    pub config: PathBuf,
    pub credentials_dir: PathBuf,
    pub models_cache_dir: PathBuf,
}

impl BrazenPaths {
    /// The three locations under an explicit wall root.
    pub fn in_wall(wall: &Path) -> Self {
        let brazen = wall.join("brazen");
        Self {
            config: brazen.join("config.toml"),
            credentials_dir: brazen.join("credentials"),
            models_cache_dir: brazen.join("models"),
        }
    }

    /// The three locations of the wall `env` names ([`Env::wall`]), or `None`
    /// outside any wall — a seat in no workspace has no providers to read, and
    /// that emptiness is rendered as a guard rather than filled from the
    /// machine's own brazen state (§16.2: nothing is ambient but the roster).
    pub fn of(env: &Env) -> Option<Self> {
        env.wall().map(|w| Self::in_wall(&w))
    }
}

/// The raw model-cache document `bz --list-models` wholesale-wrote for
/// `provider` under `dir` (§5.1 row 23), or `None` where it never ran there.
/// Read-only and forgiving — no parse, no schema coupling.
///
/// Free of the editor: the §9.4
/// pick reads this very file to seed the context window it declares (bl-848f)
/// and holds no `config.toml` draft to ask through. One naming of
/// `<provider>.json`, so the file the picker reads and the file the §9.5 pane
/// shows can never be two different files.
pub fn model_cache_at(
    dir: &Path,
    provider: &str,
    io: &dyn FileIo,
) -> std::io::Result<Option<String>> {
    Ok(io
        .read(&dir.join(format!("{provider}.json")))?
        .map(|b| String::from_utf8_lossy(&b).into_owned()))
}
