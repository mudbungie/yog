//! The workspace tab-bar view-model's tests (DESIGN §11), split from
//! [`super`] at the 300-line cap — same move as `menu/tests.rs`.

use super::*;

/// One answered row (`Query::Workspaces`), by the facts a tab is built from.
fn row(name: &str, kind: WorkspaceKind, attention: usize, pinned: Option<usize>) -> WsRow {
    WsRow {
        workspace: name.to_owned(),
        kind,
        attention,
        agents: 0,
        running: false,
        pinned,
    }
}

fn item(name: &str, kind: WorkspaceKind, attention: usize) -> WsRow {
    row(name, kind, attention, None)
}

fn named(name: &str, attention: usize) -> WsRow {
    item(
        name,
        WorkspaceKind::Named {
            name: name.to_owned(),
        },
        attention,
    )
}

#[test]
fn named_workspaces_tab_in_name_order_and_carry_facts() {
    let rows = [named("zeta-pug", 2), named("alba-koi", 0)];
    let bar = build(&rows, Some("zeta-pug"));
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["alba-koi", "zeta-pug"]);
    assert!(bar.overflow.is_empty());
    let zeta = &bar.tabs[1];
    assert!(zeta.selected && !zeta.pinned && zeta.kind == Kind::Named);
    assert_eq!(zeta.attention, 2);
    assert!(!bar.tabs[0].selected);
    // The strip beside the bar is the same answer summed (bl-296f).
    assert_eq!(strip_total(&rows), 2);
}

#[test]
fn foreign_and_replay_go_to_overflow_with_aggregate_attention() {
    let rows = [
        named("alba-koi", 0),
        item("20260101T-aa", WorkspaceKind::Foreign, 1),
        item("20260102T-bb", WorkspaceKind::Replay, 2),
    ];
    let bar = build(&rows, None);
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
    // The strip counts every workspace, folded away or not — the ⚑ total is
    // the world's, where the ⋯ badge is only what the menu still hides.
    assert_eq!(strip_total(&rows), 3);
}

#[test]
fn pinning_hoists_any_kind_into_the_tabs_in_pin_order() {
    let rows = [
        named("alba-koi", 0),
        row("20260101T-aa", WorkspaceKind::Foreign, 0, Some(1)),
        row("20260102T-bb", WorkspaceKind::Replay, 0, Some(0)),
    ];
    let bar = build(&rows, None);
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
    let rows = [
        item("20260101T-aa", WorkspaceKind::Foreign, 1),
        row("20260102T-bb", WorkspaceKind::Replay, 2, Some(0)),
    ];
    let bar = build(&rows, None);
    assert_eq!(bar.overflow.len(), 2, "pinned entries stay listed");
    assert_eq!(
        bar.overflow_attention(),
        1,
        "the pinned entry's own tab badge already shows its 2"
    );
}

#[test]
fn an_empty_answer_is_an_empty_bar_and_a_quiet_strip() {
    // The honest resting state of a frame the engine has not answered — no
    // tabs, no overflow, nothing stirring — reached by the general path with
    // no rows rather than by a branch (REMOTE §9.7's collapsed-pane rule).
    let bar = build(&[], Some("alba-koi"));
    assert!(bar.tabs.is_empty() && bar.overflow.is_empty());
    assert_eq!(strip_total(&[]), 0);
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
    let rows = [
        named("alba-koi", 0),
        item("20260101T-aa", WorkspaceKind::Foreign, 0),
        row("20260102T-bb", WorkspaceKind::Replay, 0, Some(0)),
    ];
    let bar = build(&rows, None);
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
