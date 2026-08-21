//! Which file yog itself is (bl-f558): the judgement over one reading, and the
//! process-lifetime memo built on it.
//!
//! The *end to end* proof — a real executable whose pathname is atomically
//! replaced under a live process, followed by a Start's shim write and an actual
//! consult — cannot be made in-process (replacing this test binary's own inode
//! would poison every other test in it), so it lives in
//! `tests/self_exe_replacement.rs`. What is pinned here is the rule that test
//! depends on.

use super::*;
use crate::cli_outbound::self_exe::{self_exe, usable};
use tempfile::tempdir;

/// The whole judgement is `stat`, never spelling. A `<path> (deleted)` reading
/// — Linux's `/proc/self/exe` annotation once yog's own inode is unlinked — is
/// rejected because nothing is there, and the very same name IS accepted when a
/// real executable happens to carry it, which is why no suffix is stripped.
#[test]
fn a_reading_is_usable_only_when_the_file_is_actually_there() {
    let dir = tempdir().unwrap();
    let real = dir.path().join("yog");
    fs::write(&real, "#!/bin/sh\nexit 0\n").unwrap();

    assert_eq!(usable(Some(real.clone())), Some(real.clone()));

    // What procfs hands back after `mv -f new yog`: an annotation, not a path.
    let annotated = dir.path().join("yog (deleted)");
    assert_eq!(usable(Some(annotated.clone())), None);

    // Create that exact name and it resolves — a strip would have mangled it.
    fs::write(&annotated, "#!/bin/sh\nexit 0\n").unwrap();
    assert_eq!(usable(Some(annotated.clone())), Some(annotated));

    // A yog born from a file since deleted outright, and a platform whose
    // `current_exe()` failed at all, are the same nothing.
    assert_eq!(usable(Some(dir.path().join("gone"))), None);
    assert_eq!(usable(None), None);

    // A directory is not an executable either.
    assert_eq!(usable(Some(dir.path().to_path_buf())), None);
}

/// The memo: one reading per process, so two asks separated by anything at all
/// answer the same file.
#[test]
fn the_process_reads_its_own_executable_once() {
    let first = self_exe();
    assert_eq!(first, self_exe());
    let exe = first.expect("this test binary is on disk");
    assert!(exe.is_file(), "{exe:?}");
}
