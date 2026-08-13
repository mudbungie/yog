//! The workspace tab-bar view-model's tests (DESIGN §11), split from
//! [`super`] at the 300-line cap — same move as `menu/tests.rs`.

use super::*;
use std::path::Path;

fn item(path: &str, kind: WorkspaceKind, attention: usize) -> Item {
    Item {
        ws: Workspace {
            path: PathBuf::from(path),
            kind,
        },
        attention,
    }
}

fn named(path: &str, name: &str, attention: usize) -> Item {
    item(
        path,
        WorkspaceKind::Named {
            name: name.to_owned(),
        },
        attention,
    )
}

#[test]
fn named_workspaces_tab_in_name_order_and_carry_facts() {
    let items = [
        named("/y/zeta-pug", "zeta-pug", 2),
        named("/y/alba-koi", "alba-koi", 0),
    ];
    let bar = build(&items, &[], Some(Path::new("/y/zeta-pug")));
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["alba-koi", "zeta-pug"]);
    assert!(bar.overflow.is_empty());
    let zeta = &bar.tabs[1];
    assert!(zeta.selected && !zeta.pinned && zeta.kind == Kind::Named);
    assert_eq!(zeta.attention, 2);
    assert!(!bar.tabs[0].selected);
}

#[test]
fn foreign_and_replay_go_to_overflow_with_aggregate_attention() {
    let items = [
        named("/y/alba-koi", "alba-koi", 0),
        item("/l/workspaces/20260101T-aa", WorkspaceKind::Foreign, 1),
        item("/l/replays/20260102T-bb", WorkspaceKind::Replay, 2),
    ];
    let bar = build(&items, &[], None);
    assert_eq!(bar.tabs.len(), 1);
    let names: Vec<&str> = bar.overflow.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["20260101T-aa", "20260102T-bb"]);
    // The kind rides every entry: it is what the §11 mark is said from and
    // what the §3.6 delete seat reads (neither of these two is yog's to
    // delete — only a Named workspace is).
    assert_eq!(bar.overflow[0].kind, Kind::Foreign, "foreign, not a replay");
    assert_eq!(bar.overflow[1].kind, Kind::Replay, "the read-only one");
    assert_eq!(bar.tabs[0].kind, Kind::Named, "yog's own workspace");
    assert_eq!(bar.overflow_attention(), 3, "menu badge sums entries");
}

#[test]
fn pinning_hoists_any_kind_into_the_tabs_in_pin_order() {
    let items = [
        named("/y/alba-koi", "alba-koi", 0),
        item("/l/workspaces/20260101T-aa", WorkspaceKind::Foreign, 0),
        item("/l/replays/20260102T-bb", WorkspaceKind::Replay, 0),
    ];
    let pins = [
        "/l/replays/20260102T-bb".to_owned(),
        "/l/workspaces/20260101T-aa".to_owned(),
    ];
    let bar = build(&items, &pins, None);
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        ["20260102T-bb", "20260101T-aa", "alba-koi"],
        "pins first in pin order, then named"
    );
    assert!(bar.tabs[0].pinned && bar.tabs[0].kind == Kind::Replay);
    // The §11 sole-carrier fix (bl-7e32): a hoisted entry keeps its overflow
    // row with ★ lit, so unpin has a visible carrier and the tab's context
    // menu is only an accelerator.
    let menu: Vec<&str> = bar.overflow.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(menu, ["20260101T-aa", "20260102T-bb"], "still listed");
    assert!(
        bar.overflow.iter().all(|t| t.pinned),
        "listed with the ★ lit"
    );
}

#[test]
fn the_overflow_badge_counts_only_what_is_still_folded_away() {
    let items = [
        item("/l/workspaces/20260101T-aa", WorkspaceKind::Foreign, 1),
        item("/l/replays/20260102T-bb", WorkspaceKind::Replay, 2),
    ];
    let bar = build(&items, &["/l/replays/20260102T-bb".to_owned()], None);
    assert_eq!(bar.overflow.len(), 2, "pinned entries stay listed");
    assert_eq!(
        bar.overflow_attention(),
        1,
        "the pinned entry's own tab badge already shows its 2"
    );
}

#[test]
fn every_marked_kind_says_itself_in_words_and_says_it_uniquely() {
    let mut marks = std::collections::HashSet::new();
    for kind in [Kind::Named, Kind::Foreign, Kind::Replay] {
        let Some(mark) = kind.mark() else { continue };
        // The §11 glyph doctrine: a mark made of glyphs alone puts the whole
        // load back on the glyph, and a mark shared with another kind says
        // nothing about *which* kind — so every mark carries its own words.
        assert!(
            mark.chars().any(|c| c.is_ascii_alphabetic()),
            "glyph-only mark {mark:?} for {kind:?}"
        );
        assert!(marks.insert(mark), "duplicate mark {mark:?} for {kind:?}");
    }
    assert_eq!(marks.len(), 2, "the ordinary named regime wears no mark");
    assert_eq!(Kind::Named.mark(), None, "an unmarked tab is a workspace");
    // Delete the glyph and the replay is still named — the doctrine's test.
    assert!(Kind::Replay.mark().unwrap_or_default().contains("replay"));
}

#[test]
fn a_hoisted_entry_and_its_overflow_row_wear_one_mark() {
    let items = [
        named("/y/alba-koi", "alba-koi", 0),
        item("/l/workspaces/20260101T-aa", WorkspaceKind::Foreign, 0),
        item("/l/replays/20260102T-bb", WorkspaceKind::Replay, 0),
    ];
    let pins = ["/l/replays/20260102T-bb".to_owned()];
    let bar = build(&items, &pins, None);
    assert_eq!(bar.tabs[0].kind_suffix(), " · ⏮ replay");
    assert_eq!(
        bar.tabs[0].kind_suffix(),
        bar.overflow[1].kind_suffix(),
        "pinning moves where an entry appears, never what it says"
    );
    assert_eq!(bar.overflow[0].kind_suffix(), " · foreign");
    assert!(
        bar.tabs[1].kind_suffix().is_empty(),
        "a named workspace is the unmarked ordinary regime"
    );
}

#[test]
fn a_stale_pin_key_matches_nothing_and_is_dropped() {
    let items = [named("/y/alba-koi", "alba-koi", 0)];
    let bar = build(&items, &["/gone".to_owned()], None);
    assert_eq!(bar.tabs.len(), 1);
    assert!(!bar.tabs[0].pinned);
}
