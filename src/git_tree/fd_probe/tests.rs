//! Unit tests for the `/proc/*/fd` writer scan (fd_probe).

use super::*;
use std::fs;
use tempfile::tempdir;

/// Lay out a fake `/proc/<pid>/fd/<n>` symlink to `target` plus a
/// matching `fdinfo/<n>` with `flags`.
fn fake_proc(root: &Path, pid: &str, fd: &str, target: &Path, flags: &str) {
    let fd_dir = root.join(pid).join("fd");
    fs::create_dir_all(&fd_dir).unwrap();
    std::os::unix::fs::symlink(target, fd_dir.join(fd)).unwrap();
    let fdinfo = root.join(pid).join("fdinfo");
    fs::create_dir_all(&fdinfo).unwrap();
    fs::write(fdinfo.join(fd), format!("pos:\t0\nflags:\t{flags}\n")).unwrap();
}

fn target_file(dir: &Path) -> PathBuf {
    let p = dir.join("response.json");
    fs::write(&p, b"x").unwrap();
    p
}

#[test]
fn detects_a_writer_holding_the_path() {
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let proc = dir.path().join("proc");
    // O_WRONLY (octal ...01) → writer.
    fake_proc(&proc, "42", "3", &target, "0100001");
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Held
    );
}

#[test]
fn ignores_a_reader_only_handle() {
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let proc = dir.path().join("proc");
    // O_RDONLY (trailing 0) → not a writer (e.g. the UI's own tail).
    fake_proc(&proc, "42", "3", &target, "0100000");
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Free
    );
}

#[test]
fn no_writer_when_no_fd_matches() {
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let other = dir.path().join("other.json");
    fs::write(&other, b"y").unwrap();
    let proc = dir.path().join("proc");
    fake_proc(&proc, "42", "3", &other, "0100001");
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Free
    );
}

#[test]
fn missing_target_is_no_writer() {
    let dir = tempdir().unwrap();
    let proc = dir.path().join("proc");
    fs::create_dir_all(&proc).unwrap();
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&dir.path().join("gone.json")),
        Probe::Free
    );
}

#[test]
fn missing_proc_root_is_no_writer() {
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    assert_eq!(
        ProcFsProbe::with_root(dir.path().join("no-proc")).writer_state(&target),
        Probe::Free
    );
}

#[test]
fn numeric_pid_without_an_fd_dir_is_skipped() {
    // A pid we cannot introspect — no readable `fd/` subdir (other
    // uid, raced teardown) — does not match: `read_dir` fails and the
    // scan moves on.
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let proc = dir.path().join("proc");
    fs::create_dir_all(proc.join("55")).unwrap(); // pid 55, no `fd/`
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Free
    );
}

#[test]
fn non_numeric_proc_entries_and_missing_fdinfo_are_skipped() {
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let proc = dir.path().join("proc");
    // A non-pid dir is skipped.
    fs::create_dir_all(proc.join("acpi")).unwrap();
    // A pid whose fd matches but whose fdinfo is absent → not a
    // writer (flags unknown, treated conservatively).
    let fd_dir = proc.join("7").join("fd");
    fs::create_dir_all(&fd_dir).unwrap();
    std::os::unix::fs::symlink(&target, fd_dir.join("4")).unwrap();
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Free
    );
}

#[test]
fn fdinfo_without_flags_line_is_not_writable() {
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let proc = dir.path().join("proc");
    let fd_dir = proc.join("9").join("fd");
    fs::create_dir_all(&fd_dir).unwrap();
    std::os::unix::fs::symlink(&target, fd_dir.join("5")).unwrap();
    let fdinfo = proc.join("9").join("fdinfo");
    fs::create_dir_all(&fdinfo).unwrap();
    fs::write(fdinfo.join("5"), "pos:\t0\n").unwrap(); // no flags line
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Free
    );
}

#[test]
fn non_numeric_fd_entry_is_skipped() {
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let proc = dir.path().join("proc");
    let fd_dir = proc.join("11").join("fd");
    fs::create_dir_all(&fd_dir).unwrap();
    // A non-numeric fd name resolving to target is skipped.
    std::os::unix::fs::symlink(&target, fd_dir.join("notanum")).unwrap();
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Free
    );
}

#[test]
fn non_symlink_fd_entry_is_skipped() {
    // A regular file where an fd symlink is expected → `read_link`
    // fails and the entry is skipped (matches a racing teardown).
    let dir = tempdir().unwrap();
    let target = target_file(dir.path());
    let proc = dir.path().join("proc");
    let fd_dir = proc.join("13").join("fd");
    fs::create_dir_all(&fd_dir).unwrap();
    fs::write(fd_dir.join("3"), b"not a symlink").unwrap();
    assert_eq!(
        ProcFsProbe::with_root(proc).writer_state(&target),
        Probe::Free
    );
}
