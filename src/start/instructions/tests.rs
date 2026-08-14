//! §3.7's walk: authority root, precedence order, and every admission rule.

use super::*;
use std::fs;
use tempfile::{TempDir, tempdir};

/// A directory tree rooted at a fresh tempdir, with `.git` where asked.
fn tree(dirs: &[&str]) -> TempDir {
    let dir = tempdir().unwrap();
    for d in dirs {
        fs::create_dir_all(dir.path().join(d)).unwrap();
    }
    dir
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|n| (*n).to_owned()).collect()
}

/// The default policy: this suite's own convention, and only it.
fn agents() -> Vec<String> {
    names(&["AGENTS.md"])
}

#[test]
fn the_walk_runs_root_first_and_ranks_precedence_into_the_destination() {
    let t = tree(&[".git", "crates/foo"]);
    let root = t.path();
    fs::write(root.join("AGENTS.md"), "top").unwrap();
    fs::write(root.join("crates/foo/AGENTS.md"), "leaf").unwrap();
    let out = specs(&root.join("crates/foo"), &agents());
    assert_eq!(
        out,
        vec![
            format!(
                "instructions/00/AGENTS.md={}",
                root.join("AGENTS.md").display()
            ),
            format!(
                "instructions/01/crates/foo/AGENTS.md={}",
                root.join("crates/foo/AGENTS.md").display()
            ),
        ],
        "outermost first: the most specific instructions arrive last"
    );
}

#[test]
fn a_worktrees_gitdir_pointer_file_is_an_authority_root_too() {
    let t = tree(&["wt"]);
    let wt = t.path().join("wt");
    // A `work/<id>` worktree's `.git` is a file, not a directory.
    fs::write(wt.join(".git"), "gitdir: /elsewhere\n").unwrap();
    fs::write(wt.join("AGENTS.md"), "worktree rules").unwrap();
    // The tempdir above it also declares instructions; they are unreachable.
    fs::write(t.path().join("AGENTS.md"), "ambient").unwrap();
    let out = specs(&wt, &agents());
    assert_eq!(
        out,
        vec![format!(
            "instructions/00/AGENTS.md={}",
            wt.join("AGENTS.md").display()
        )]
    );
}

#[test]
fn nothing_above_the_authority_root_is_reachable() {
    let t = tree(&["repo/.git/objects", "repo/src"]);
    // The parent of the repo root declares instructions — an untrusted parent.
    fs::write(t.path().join("AGENTS.md"), "ambient").unwrap();
    fs::write(t.path().join("repo/AGENTS.md"), "project").unwrap();
    let out = specs(&t.path().join("repo/src"), &agents());
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].ends_with(&t.path().join("repo/AGENTS.md").display().to_string()));
}

/// The ascent's other terminus. It cannot be staged inside a tempdir — the
/// walk leaves it by design, and an ancestor of `$TMPDIR` may itself be a
/// repository (this machine's `/tmp` is one) — so it is asserted where the
/// filesystem itself ends the ascent.
#[test]
fn a_binding_with_no_repository_above_it_is_its_own_authority_root() {
    let root = Path::new("/");
    assert_eq!(authority_root(root), root.to_path_buf());
}

#[test]
fn each_configured_name_rides_in_its_declared_order_within_one_level() {
    let t = tree(&[".git"]);
    fs::write(t.path().join("AGENTS.md"), "a").unwrap();
    fs::write(t.path().join("HOUSE.md"), "h").unwrap();
    let out = specs(t.path(), &names(&["HOUSE.md", "AGENTS.md"]));
    assert!(out[0].starts_with("instructions/00/HOUSE.md="), "{out:?}");
    assert!(out[1].starts_with("instructions/01/AGENTS.md="), "{out:?}");
}

#[test]
fn a_symlink_is_skipped_because_the_freeze_is_byte_exact() {
    let t = tree(&[".git"]);
    fs::write(t.path().join("real.md"), "rules").unwrap();
    std::os::unix::fs::symlink("real.md", t.path().join("AGENTS.md")).unwrap();
    assert!(specs(t.path(), &agents()).is_empty());
}

#[test]
fn a_directory_wearing_an_instruction_name_is_not_a_document() {
    let t = tree(&[".git", "AGENTS.md"]);
    assert!(specs(t.path(), &agents()).is_empty());
}

#[test]
fn an_oversize_document_is_skipped_whole_never_truncated() {
    let t = tree(&[".git"]);
    fs::write(
        t.path().join("AGENTS.md"),
        vec![b'x'; MAX_BYTES as usize + 1],
    )
    .unwrap();
    assert!(
        specs(t.path(), &agents()).is_empty(),
        "half a rule reads exactly like a whole rule"
    );
}

#[test]
fn a_document_at_the_cap_still_rides() {
    let t = tree(&[".git"]);
    fs::write(t.path().join("AGENTS.md"), vec![b'x'; MAX_BYTES as usize]).unwrap();
    assert_eq!(specs(t.path(), &agents()).len(), 1);
}

#[test]
fn the_document_count_is_bounded() {
    let t = tree(&[".git"]);
    let list: Vec<String> = (0..MAX_DOCS + 4).map(|i| format!("R{i}.md")).collect();
    for name in &list {
        fs::write(t.path().join(name), "x").unwrap();
    }
    assert_eq!(specs(t.path(), &list).len(), MAX_DOCS);
}

#[test]
fn a_destination_that_cannot_be_spelled_is_skipped_rather_than_mangled() {
    let t = tree(&[".git", "a=b"]);
    fs::write(t.path().join("a=b/AGENTS.md"), "x").unwrap();
    assert!(
        specs(&t.path().join("a=b"), &agents()).is_empty(),
        "`=` is --pin's own separator, split at the first occurrence"
    );
}

#[test]
fn a_non_utf8_path_is_skipped() {
    use std::os::unix::ffi::OsStrExt;
    let t = tree(&[".git"]);
    let odd = t.path().join(std::ffi::OsStr::from_bytes(b"sub\xff"));
    fs::create_dir_all(&odd).unwrap();
    fs::write(odd.join("AGENTS.md"), "x").unwrap();
    assert!(specs(&odd, &agents()).is_empty());
}

#[test]
fn an_empty_policy_and_an_absent_document_both_discover_nothing() {
    let t = tree(&[".git"]);
    assert!(specs(t.path(), &agents()).is_empty(), "no file");
    fs::write(t.path().join("AGENTS.md"), "x").unwrap();
    assert!(specs(t.path(), &[]).is_empty(), "no names");
}
