//! **S10-T5 pinned-tree**: the Files tab read as of a commit — the listing out
//! of `ls-tree`, one file's bytes out of `git show`, and both declining
//! honestly when the commit names nothing.

use crate::files_view::{FilesView, Preview};
use crate::git_tree::tests::fixture::Fixture;
use crate::rail::tree::parse_row;
use crate::rail::{files_at, preview_at};

const CONV: &str = "20260427T120000Z-aaaa";
const GOAL: &str = "walk the rail";

/// The listing is that commit's blobs, sorted, with their sizes as of then —
/// and `messages/` stays the Transcript tab's, exactly as the live walk has it.
#[test]
fn the_pinned_listing_is_that_commits_tree() {
    let fx = Fixture::new();
    fx.build_agent(CONV, GOAL);
    let FilesView::Present { entries, truncated } = files_at(&fx.path, &format!("agents/{CONV}"))
    else {
        panic!("a real commit lists");
    };
    assert!(!truncated);
    let paths: Vec<&str> = entries.iter().map(|e| e.rel_path.as_str()).collect();
    assert!(paths.contains(&"goal.md"), "{paths:?}");
    assert!(paths.contains(&"soul.md"), "{paths:?}");
    assert!(paths.contains(&"summary/001.md"), "{paths:?}");
    assert!(
        !paths.iter().any(|p| p.starts_with("messages")),
        "messages/ is the Transcript tab's: {paths:?}"
    );
    let goal = entries
        .iter()
        .find(|e| e.rel_path == "goal.md")
        .expect("goal.md listed");
    assert_eq!(goal.size, GOAL.len() as u64);
    assert!(!goal.is_dir, "ls-tree -r names blobs");
}

/// A commit git does not have lists nothing — the same "nothing to list here"
/// value a torn-down worktree already produces, said once.
#[test]
fn a_commit_that_is_not_there_lists_nothing() {
    let fx = Fixture::new();
    assert_eq!(files_at(&fx.path, "deadbeef"), FilesView::AbsentWorktree);
}

/// One file's bytes as of the commit, classified by the same vocabulary the
/// live preview uses.
#[test]
fn the_pinned_preview_is_that_commits_bytes() {
    let fx = Fixture::new();
    fx.build_agent(CONV, GOAL);
    let at = format!("agents/{CONV}");
    assert_eq!(
        preview_at(&fx.path, &at, "goal.md"),
        Preview::Text(GOAL.to_owned())
    );
    // A path the tree does not carry is a race against the listing it came
    // from, not a fact: an empty read, never an error row.
    assert_eq!(
        preview_at(&fx.path, &at, "nowhere.md"),
        Preview::Text(String::new())
    );
}

/// Binary and over-cap bytes read exactly as the live preview reads them — one
/// vocabulary for "what this file is", asked of a tree instead of a worktree.
#[test]
fn the_pinned_preview_classifies_binary_and_over_cap_bytes() {
    let fx = Fixture::new();
    fx.build_agent(CONV, GOAL);
    fx.commit_other("blob.bin", "a\0b");
    fx.commit_other("long.txt", &"x".repeat(crate::files_view::PREVIEW_CAP + 5));
    assert_eq!(
        preview_at(&fx.path, "config/default", "blob.bin"),
        Preview::Binary { size: 3 }
    );
    let Preview::Truncated { text, size } = preview_at(&fx.path, "config/default", "long.txt")
    else {
        panic!("over the cap truncates");
    };
    assert_eq!(text.len(), crate::files_view::PREVIEW_CAP);
    assert_eq!(size as usize, crate::files_view::PREVIEW_CAP + 5);
}

/// A row `ls-tree` writes without a numeric blob size contributes nothing:
/// this listing is blobs, and a size it cannot state it does not invent.
#[test]
fn a_row_without_a_blob_size_contributes_nothing() {
    assert!(parse_row("040000 tree abc123       -\tsummary").is_none());
    assert!(parse_row("no tab in this line").is_none());
}
