//! Tests for the files_view VM. The walk and preview are pure over injected
//! paths; the defensive read/stat error arms (unreachable through `build` on a
//! sane tree) are covered by calling the private helpers directly, the same way
//! the codebase unit-tests `toggle_path` / `flatten`.

use std::path::{Path, PathBuf};

use super::*;
use tempfile::{TempDir, tempdir};

/// A workspace tempdir with `agents/<id>/` created and populated by `fill`.
fn worktree(id: &str, fill: impl FnOnce(&Path)) -> (TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let ws = dir.path().to_path_buf();
    let root = agent_worktree(&ws, id);
    std::fs::create_dir_all(&root).unwrap();
    fill(&root);
    (dir, ws)
}

fn present(view: FilesView) -> (Vec<FileEntry>, bool) {
    match view {
        FilesView::Present { entries, truncated } => (entries, truncated),
        FilesView::AbsentWorktree => panic!("expected Present"),
    }
}

fn rels(entries: &[FileEntry]) -> Vec<&str> {
    entries.iter().map(|e| e.rel_path.as_str()).collect()
}

#[test]
fn missing_worktree_is_absent_not_error() {
    let dir = tempdir().unwrap();
    assert_eq!(build(dir.path(), "ghost"), FilesView::AbsentWorktree);
}

#[test]
fn empty_worktree_is_present_with_no_entries() {
    let (_d, ws) = worktree("a1", |_| {});
    assert_eq!(
        build(&ws, "a1"),
        FilesView::Present {
            entries: Vec::new(),
            truncated: false,
        }
    );
}

#[test]
fn listing_is_sorted_dfs_excluding_git_and_messages_at_root_only() {
    let (_d, ws) = worktree("a1", |root| {
        std::fs::write(root.join("goal.md"), "hi").unwrap();
        std::fs::write(root.join("soul.md"), "you").unwrap();
        // Excluded at the root: the Transcript tab's domain and git plumbing.
        std::fs::write(root.join(".git"), "gitdir: ...").unwrap();
        std::fs::create_dir_all(root.join("messages")).unwrap();
        std::fs::write(root.join("messages/001.md"), "x").unwrap();
        // A work-product subtree; a *nested* `messages` dir is kept (only the
        // root name is reserved).
        std::fs::create_dir_all(root.join("work/messages")).unwrap();
        std::fs::write(root.join("work/messages/deep.md"), "d").unwrap();
        std::fs::write(root.join("work/product.txt"), "prod").unwrap();
    });
    let (entries, truncated) = present(build(&ws, "a1"));
    assert!(!truncated);
    assert_eq!(
        rels(&entries),
        vec![
            "goal.md",
            "soul.md",
            "work",
            "work/messages",
            "work/messages/deep.md",
            "work/product.txt",
        ]
    );
    // dir/file flags and file sizes are carried verbatim.
    let by = |p: &str| entries.iter().find(|e| e.rel_path == p).unwrap();
    assert!(by("work").is_dir && by("work/messages").is_dir);
    assert!(!by("goal.md").is_dir);
    assert_eq!(by("goal.md").size, 2);
    assert_eq!(by("work").size, 0);
}

#[test]
fn entry_cap_stops_the_walk_and_marks_truncation() {
    let (_d, ws) = worktree("a1", |root| {
        for i in 0..=MAX_ENTRIES {
            std::fs::write(root.join(format!("f{i:04}")), "x").unwrap();
        }
    });
    let (entries, truncated) = present(build(&ws, "a1"));
    assert_eq!(entries.len(), MAX_ENTRIES);
    assert!(truncated);
}

#[test]
fn depth_cap_lists_to_the_floor_and_marks_the_hidden_remainder() {
    // a/b/c/d/e/f are depths 1..6; f (depth 6) is a dir holding g.txt (depth 7),
    // which is beyond the floor and hidden.
    let (_d, ws) = worktree("a1", |root| {
        std::fs::create_dir_all(root.join("a/b/c/d/e/f")).unwrap();
        std::fs::write(root.join("a/b/c/d/e/f/g.txt"), "deep").unwrap();
    });
    let (entries, truncated) = present(build(&ws, "a1"));
    assert!(truncated, "a non-empty dir at the depth floor truncates");
    let paths = rels(&entries);
    assert!(paths.contains(&"a/b/c/d/e/f"));
    assert!(!paths.contains(&"a/b/c/d/e/f/g.txt"));
}

#[test]
fn empty_dir_at_the_depth_floor_is_not_truncation() {
    // The depth-6 entry is an empty dir: it is listed, nothing is hidden below.
    let (_d, ws) = worktree("a1", |root| {
        std::fs::create_dir_all(root.join("a/b/c/d/e/empty")).unwrap();
    });
    let (entries, truncated) = present(build(&ws, "a1"));
    assert!(!truncated);
    assert!(rels(&entries).contains(&"a/b/c/d/e/empty"));
}

#[test]
fn entry_meta_classifies_dir_file_and_the_unstattable() {
    let (_d, ws) = worktree("a1", |root| {
        std::fs::write(root.join("f"), "abcde").unwrap();
    });
    let root = agent_worktree(&ws, "a1");
    assert_eq!(entry_meta(&root), (true, 0));
    assert_eq!(entry_meta(&root.join("f")), (false, 5));
    assert_eq!(entry_meta(&root.join("gone")), (false, 0));
}

#[test]
fn dir_helpers_tolerate_a_missing_directory() {
    let missing = Path::new("/no/such/files_view/dir");
    assert!(sorted_children(missing, false).is_empty());
    assert!(!has_children(missing));
}

#[test]
fn preview_reads_text_within_the_cap() {
    let (_d, ws) = worktree("a1", |root| {
        std::fs::write(root.join("f.md"), "hello world").unwrap();
    });
    assert_eq!(
        preview(&agent_worktree(&ws, "a1").join("f.md")),
        Preview::Text("hello world".into())
    );
}

#[test]
fn preview_sniffs_a_nul_byte_as_binary() {
    let (_d, ws) = worktree("a1", |root| {
        std::fs::write(root.join("bin"), b"ab\0cd").unwrap();
    });
    assert_eq!(
        preview(&agent_worktree(&ws, "a1").join("bin")),
        Preview::Binary { size: 5 }
    );
}

#[test]
fn preview_truncates_past_the_cap_keeping_the_true_size() {
    let big = PREVIEW_CAP + 4096;
    let (_d, ws) = worktree("a1", |root| {
        std::fs::write(root.join("big.txt"), vec![b'a'; big]).unwrap();
    });
    match preview(&agent_worktree(&ws, "a1").join("big.txt")) {
        Preview::Truncated { text, size } => {
            assert_eq!(text.len(), PREVIEW_CAP);
            assert_eq!(size, big as u64);
        }
        other => panic!("expected Truncated, got {other:?}"),
    }
}

#[test]
fn preview_of_a_vanished_file_is_opaque_binary() {
    let dir = tempdir().unwrap();
    assert_eq!(
        preview(&dir.path().join("gone")),
        Preview::Binary { size: 0 }
    );
}
