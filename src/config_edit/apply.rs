//! `yog --editor-apply` shim: the `$EDITOR` lernie execs (DESIGN §9.3 Y21).
//!
//! **TASK-0 FINDING** — lernie's exact `$EDITOR` invocation shape
//! (`lernie/src/bin/lernie/cli.rs:20-27`, `edit_in_editor`, source-read
//! 2026-07-17). `lernie config` hands the authoring checkout to `$EDITOR` as
//! ```text
//!     sh -c 'exec {EDITOR} "$1"' sh <checkout-dir>
//! ```
//! so `$EDITOR` is **word-split** by `sh` and receives exactly ONE positional
//! argument — the checkout **directory** (`<workspace>/.config-author`,
//! `template/authoring/mod.rs:121,132`), never per-file — quoted `"$1"` so a
//! spaced path stays one argv element. The process cwd is inherited (unset),
//! so the shim must take the checkout from **argv**, never cwd. Crucially,
//! lernie has already refreshed `descriptions/**` INTO that checkout before
//! calling `$EDITOR` (`authoring/mod.rs:131`) and commits the whole checkout
//! after — so the shim copies **only the staged files** over it and never
//! deletes, or lernie's fresh `descriptions/**` (and any unedited config)
//! would be clobbered.
//!
//! Contract: argv is `<yog> --editor-apply <checkout>`; env `YOG_EDIT_SRC` is
//! the staging dir. Every regular file under the staging dir is copied into
//! the checkout at the same relative path (parent dirs created); every other
//! checkout path is left untouched. Exit 0 on success; non-zero + a stderr
//! diagnostic on any failure — which aborts lernie's commit cleanly.
//!
//! **Symlink / special-file hygiene.** Staging holds the plain files yog's UI
//! wrote. Only regular files and directories are mirrored; a symlink or
//! special file (fifo, socket, device) is **skipped** — never followed into a
//! config commit, so nothing outside staging can be smuggled in and no link
//! can escape the checkout.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The argv flag selecting shim mode; also the tail of the composed `$EDITOR`
/// value ([`editor_env_value`](super::branch::edit::editor_env_value)). The
/// one authoritative home for the string both sides must agree on.
pub const EDITOR_APPLY_FLAG: &str = "--editor-apply";

/// Shim entry mapped to a process exit code (the value `main` exits with).
/// `edit_src` is `YOG_EDIT_SRC` (env), `checkout` the argv the shim received.
/// A missing input or any copy error is exit 1 with a `yog --editor-apply:`
/// diagnostic on stderr — the non-zero exit aborts lernie's commit.
pub fn run_shim(edit_src: Option<String>, checkout: Option<String>) -> i32 {
    match shim(edit_src, checkout) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("yog {EDITOR_APPLY_FLAG}: {e}");
            1
        }
    }
}

/// The fallible core of [`run_shim`]: validate both inputs are present, then
/// copy the staged files over the checkout.
fn shim(edit_src: Option<String>, checkout: Option<String>) -> io::Result<()> {
    let src = edit_src
        .filter(|s| !s.is_empty())
        .ok_or_else(|| io::Error::other("YOG_EDIT_SRC unset — nothing to apply"))?;
    let dst = checkout
        .filter(|s| !s.is_empty())
        .ok_or_else(|| io::Error::other("no checkout path in argv"))?;
    copy_staged(Path::new(&src), Path::new(&dst))?;
    Ok(())
}

/// Copy every regular file under `staging` into `checkout` at the same
/// relative path, creating parent dirs; nothing in `checkout` is ever deleted
/// (the "only drafted files" rule — lernie's freshly-refreshed
/// `descriptions/**` must survive). Nested dirs are mirrored recursively;
/// symlinks and special files are skipped (see the module hygiene note).
/// Returns the checkout-relative paths written, sorted, for assertions.
pub fn copy_staged(staging: &Path, checkout: &Path) -> io::Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    copy_dir(staging, checkout, Path::new(""), &mut written)?;
    written.sort();
    Ok(written)
}

fn copy_dir(src: &Path, dst_root: &Path, rel: &Path, written: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let child_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir(&entry.path(), dst_root, &child_rel, written)?;
        } else if file_type.is_file() {
            let dst = dst_root.join(&child_rel);
            // A path joined under `dst_root` always has a parent (at worst
            // `dst_root` itself); the fallback keeps the copy panic-free.
            fs::create_dir_all(dst.parent().unwrap_or(dst_root))?;
            fs::copy(entry.path(), &dst)?;
            written.push(child_rel);
        }
        // symlinks & special files: skipped (module hygiene note).
    }
    Ok(())
}

#[cfg(test)]
mod tests;
