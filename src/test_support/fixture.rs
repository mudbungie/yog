//! The suite's half of the executable-file discipline — a thin panic over
//! [`crate::git_env::write_exec`], which is the whole of it (bl-fd28, made
//! production's home by bl-e6c9). A fixture wants no `Result`; everything else
//! about the hazard, the measurement and the shape of the answer is stated
//! once, where the write happens.
//!
//! The two mode helpers below are the OTHER fixture and no part of that
//! hazard: they take the owner's write bit off a file and put it back, so a
//! test can stage "this cannot be recorded". They live here because the
//! mode-bit vocabulary lives here, and neither takes a mode — an exec bit is
//! not reachable through them.

use std::path::Path;

/// Write `body` to `path` and mark it `0755` — [`crate::git_env::write_exec`]
/// with the error turned into a panic. The one lawful way for a test in this
/// crate to create an executable file; `rules/no-hand-chmod.yml` refuses the
/// hand-rolled spelling.
pub(crate) fn write_exec(path: &Path, body: &str) {
    crate::git_env::write_exec(path, body)
        .unwrap_or_else(|e| panic!("write the executable fixture {}: {e}", path.display()));
}

/// Take the owner's write bit off an existing file — the "this cannot be
/// recorded" fixture, and no part of the executable-bit hazard above. It lives
/// here because the mode-bit vocabulary lives here (see the rule), and it takes
/// no mode: an exec bit is not reachable through it.
pub(crate) fn read_only(path: &Path) {
    chmod(path, 0o444);
}

/// Undo [`read_only`] — restore an ordinary `0644`, so a `TempDir` can clean up
/// after a test that made one of its files unwritable.
pub(crate) fn writable(path: &Path) {
    chmod(path, 0o644);
}

fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
}
