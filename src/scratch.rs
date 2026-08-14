//! **I3's scratch temp** (DESIGN §2 I3, §5.2): the one spelling of
//! `.<name>.yog-tmp-<pid>`, the predicate that recognizes one, and the startup
//! sweep that removes the stale ones.
//!
//! I3, verbatim: *"All yog file writes are temp-in-destination-directory +
//! `rename`. Never in-place truncation, never a temp on another filesystem
//! (EXDEV). Temp names are dotfiles (`.<name>.yog-tmp-<pid>`) so no substrate
//! reads them; leftovers older than 24 h are swept at startup."*
//!
//! The write half had three sites, each spelling the name itself; the sweep
//! half was never written at all (bl-e47c). They are **one fact** — a sweep
//! that did not spell the temp exactly as the writers do would delete nothing,
//! or delete something else — so the name lives here, the three writers ask
//! for it ([`temp_in`]), and the sweep recognizes what it produced
//! ([`is_temp`]).
//!
//! A leftover only happens when a process dies between the write and the
//! rename, and nothing reads one (that is what the dotfile buys). So this is
//! hygiene, and it is **best-effort and narrow**: only a file whose name this
//! module would itself have written, only directly inside a directory yog
//! writes temps into ([`dirs`]), only when it has been untouched for over
//! [`STALE_SECS`]. Never a directory, never a symlink, never a recursive walk.

use std::path::{Path, PathBuf};

use crate::config_edit::brazen::BrazenPaths;
use crate::config_edit::lernie_global::LernieGlobal;
use crate::xdg::Env;

/// A leftover older than this is swept (§2 I3, §5.2): 24 h, in seconds. One
/// home for the bound — the §9.3 staging sweep
/// ([`sweep_staging`](crate::config_edit::branch::edit::sweep_staging)) is the
/// same sentence's other half and reads it here.
pub const STALE_SECS: i64 = 24 * 60 * 60;

/// The `.yog-tmp-` infix every temp carries, between the destination's name
/// and the writing process's pid.
const MARK: &str = ".yog-tmp-";

/// I3's temp for a file named `name` in its destination's own directory `dir`:
/// `<dir>/.<name>.yog-tmp-<pid>`. A dotfile, so no substrate reads it; in the
/// destination's own directory, so the commit is a same-filesystem rename and
/// never EXDEV.
pub fn temp_in(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!(".{name}{MARK}{}", std::process::id()))
}

/// Whether `name` is one of [`temp_in`]'s: a dotfile whose tail is
/// `.yog-tmp-<digits>` after a non-empty destination name. Exact, because the
/// sweep deletes what this recognizes — an operator's own `.notes.yog-tmp-old`
/// is not ours and stays.
pub fn is_temp(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('.') else {
        return false;
    };
    rest.rsplit_once(MARK).is_some_and(|(dest, pid)| {
        !dest.is_empty() && !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit())
    })
}

/// Pure decision (clock-injected): which of `files` — `(path, mtime)` in unix
/// seconds — were last touched more than 24 h before `now_secs`. Exactly 24 h
/// is kept, as the §9.3 staging sweep's own boundary is.
pub fn stale(now_secs: i64, files: &[(PathBuf, i64)]) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|(_, mtime)| now_secs - mtime > STALE_SECS)
        .map(|(p, _)| p.clone())
        .collect()
}

/// Every I3 temp lying **directly** in `dir`, paired with its mtime (unix
/// seconds). Best-effort like the staging enumeration: a missing directory, a
/// dangling symlink or an un-stat-able entry contributes nothing, and nothing
/// but a regular file whose name [`is_temp`] is ever returned — `DirEntry`
/// metadata does not traverse symlinks, so a link named like a temp is not a
/// file here and is skipped.
fn temps_in(dir: &Path) -> Vec<(PathBuf, i64)> {
    use std::os::unix::fs::MetadataExt as _;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name();
        if !is_temp(&name.to_string_lossy()) {
            continue;
        }
        if let Ok(meta) = entry.metadata()
            && meta.is_file()
        {
            out.push((entry.path(), meta.mtime()));
        }
    }
    out
}

/// Sweep `dirs`: best-effort delete of every I3 temp in them untouched for
/// over 24 h (§2 I3, §5.2's startup sweep). Returns what was decided stale, in
/// the order the directories were given. The wall clock is the caller's — the
/// engine's injected [`Clock`](crate::ui_state::Clock) — keeping the decision
/// ([`stale`]) pure.
pub fn sweep(dirs: &[PathBuf], now_secs: i64) -> Vec<PathBuf> {
    let mut swept = Vec::new();
    for dir in dirs {
        for temp in stale(now_secs, &temps_in(dir)) {
            let _ = std::fs::remove_file(&temp);
            swept.push(temp);
        }
    }
    swept
}

/// Every directory yog writes an I3 temp into, folded from the composed world
/// (§16.2) — the sweep's whole territory, and the inverse of the three write
/// sites: `ui.json`'s state root (§4.1), the §9.2 lernie config root and its
/// `workflows/`, and per wall (§16.2) the three brazen destinations §9.1 and
/// [`bz_host::store`](crate::bz_host::store) write — `config.toml`'s directory,
/// the credentials dir and the model cache. A wall is discovered rather than
/// asked for: the roster is on disk, and a sweep at boot has no focus yet.
pub fn dirs(world: &Env) -> Vec<PathBuf> {
    let mut out = vec![
        world.yog_state_root(),
        world.lernie_config_root(),
        LernieGlobal::resolve(world).workflows_dir(),
    ];
    let walls = crate::world::wall::walls_dir(&crate::world::layout(world).root);
    for entry in std::fs::read_dir(walls).into_iter().flatten().flatten() {
        let paths = BrazenPaths::in_wall(&entry.path());
        out.extend(paths.config.parent().map(Path::to_path_buf));
        out.push(paths.credentials_dir);
        out.push(paths.models_cache_dir);
    }
    out
}

#[cfg(test)]
mod tests;
