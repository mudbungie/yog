//! The **wall-rooted** brazen seams (DESIGN §16.2 as amended by the blast-radius ruling): yog's
//! own [`CredStore`] and [`ModelCache`] impls, rooted at an explicit directory
//! instead of brazen's ambient per-OS fold.
//!
//! **Why yog implements these at all.** brazen's shipped shim
//! (`XdgCredStore` / `XdgModelCache`) resolves its directories from the
//! **process** environment, so no `Env` a caller folds can move them: yog's
//! in-process `bz` calls would read the operator's machine-wide credentials
//! whatever wall was asked about, and the one var that would move them
//! (`$XDG_DATA_HOME`) is the world's anchor, which must not be overridden
//! (§14 — overriding it recurses). brazen's own exposure note names the exit
//! verbatim: *"a host wanting isolation can supply its own seam impls instead
//! and never touch these."* These are those, and rooting them at a path
//! rather than an env fold is what makes the in-process seat and the spawned
//! `yog bz` seat resolve the same wall through the same
//! [`BrazenPaths`](crate::config_edit::brazen::BrazenPaths).
//!
//! **What is not duplicated.** The documents are brazen's own serde (`Cred`,
//! `CachedModels`) and the foreign-credential parse is brazen's pure
//! [`parse_ambient`] — the wall moves *where the file is*, never what is in
//! it, so there is no second schema to drift.

use std::fs;
use std::io::{self, Write as _};
use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

use brazen::{
    AmbientFormat, AmbientSpec, CachedModels, Cred, CredStore, ModelCache, parse_ambient,
};

use crate::xdg::Env;

/// One 0600 JSON file per provider under the wall's `brazen/credentials`
/// (§5.1 #22). `get` is `None` on any miss — a corrupt or absent file is the
/// no-creds path, never an error, exactly as brazen's own store rules it.
pub(crate) struct WallCredStore {
    dir: PathBuf,
    /// The snapshot ambient discovery reads (§5.5): an `ApiKeyEnv` spec names a
    /// variable whose value is the key, and a `ClaudeCode` spec names a `~/`
    /// path. Injected rather than read live, so the module rule holds here too.
    env: Env,
}

impl WallCredStore {
    pub(crate) fn new(dir: PathBuf, env: Env) -> Self {
        Self { dir, env }
    }

    fn path(&self, provider: &str) -> PathBuf {
        self.dir.join(format!("{provider}.json"))
    }

    /// Expand a leading `~/` against the snapshot's `$HOME` (brazen auth §5.5);
    /// anything else passes through verbatim.
    fn expand_home(&self, path: &str) -> PathBuf {
        match path.strip_prefix("~/") {
            Some(rest) => self.env.home_dir().join(rest),
            None => PathBuf::from(path),
        }
    }
}

impl CredStore for WallCredStore {
    fn get(&self, provider: &str) -> Option<Cred> {
        serde_json::from_slice(&fs::read(self.path(provider)).ok()?).ok()
    }

    fn put(&self, provider: &str, cred: &Cred) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(cred)?;
        fs::create_dir_all(&self.dir)?;
        fs::set_permissions(&self.dir, fs::Permissions::from_mode(0o700))?;
        write_atomic(&self.path(provider), &bytes, 0o600)
    }

    fn discover(&self, spec: &AmbientSpec) -> Option<Cred> {
        let bytes = match spec.format {
            AmbientFormat::ApiKeyEnv => self.env.var(&spec.path)?.into_bytes(),
            AmbientFormat::ClaudeCode => fs::read(self.expand_home(&spec.path)).ok()?,
        };
        parse_ambient(spec.format, &bytes)
    }
}

/// One JSON file per provider under the wall's `brazen/models` (§5.1 #23).
/// Forgiving on read and best-effort on write — a regenerable cache, so a
/// failed write warns nowhere and self-heals on the next `bz --list-models`.
pub(crate) struct WallModelCache {
    dir: PathBuf,
}

impl WallModelCache {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, provider: &str) -> PathBuf {
        self.dir.join(format!("{provider}.json"))
    }

    fn write(&self, provider: &str, cached: &CachedModels) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(cached)?;
        fs::create_dir_all(&self.dir)?;
        write_atomic(&self.path(provider), &bytes, 0o600)
    }
}

impl ModelCache for WallModelCache {
    fn get(&self, provider: &str) -> Option<CachedModels> {
        serde_json::from_slice(&fs::read(self.path(provider)).ok()?).ok()
    }

    fn put(&self, provider: &str, cached: &CachedModels) {
        drop(self.write(provider, cached));
    }
}

/// I3's write, at owner-only mode: a dot-temp **in the destination's own
/// directory** created at `mode` (never a create-then-chmod window), synced,
/// then renamed — so a concurrent reader sees the whole old or the whole new
/// file and never a partial one, and the secret is never briefly world-readable.
fn write_atomic(dest: &Path, bytes: &[u8], mode: u32) -> io::Result<()> {
    let name = dest
        .file_name()
        .map_or_else(|| "cred".to_owned(), |n| n.to_string_lossy().into_owned());
    let dir = dest.parent().unwrap_or_else(|| Path::new("."));
    let tmp = dir.join(format!(".{name}.yog-tmp-{}", std::process::id()));
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(&tmp)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    fs::rename(&tmp, dest)
}

#[cfg(test)]
mod tests;
