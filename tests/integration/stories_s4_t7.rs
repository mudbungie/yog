//! STORIES **S4-T7** tab-strip: the tab strip is the sphere wall — pinned tabs
//! hoist **in pin order** ahead of name order, each tab's badge is its own
//! workspace's attention rollup, foreign and replay workspaces fall to the ⋯
//! overflow (real, but not regimes), and the overflow button carries their
//! aggregate (STORIES S4.7, DESIGN §3.1, §11).
//!
//! Driven the way the top bar drives it (REMOTE §9.7, bl-296f): the
//! `Query::Workspaces` answer — §3.1 classification, §6 per-workspace rollup
//! and §4.1 pin *rank*, all three meeting at the boundary — folded by
//! `nav::tabs::build`, which is the whole of what the seat does.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents};
use std::sync::Arc;
use tempfile::tempdir;
use yog::nav::tabs::Kind;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// A workspace whose single agent rests **unacknowledged** — one unit of §6
/// attention (rule 2, bl-2194: a clean rest stirs too).
fn stirring(id: &str) -> AgentFixture {
    AgentFixture::new(id, "work\n").settled(true)
}

/// A workspace whose single agent rests **abandoned** — the will-not-retry
/// assertion suppresses rule 2, so it contributes nothing to any rollup.
fn quiet(id: &str) -> AgentFixture {
    AgentFixture::new(id, "done\n")
        .settled(true)
        .mark("abandoned")
}

/// Fire the §4.1 pin act and take the listing it answers with (bl-b986) — the
/// receipt IS the chrome the pin ranks, re-derived after the write.
fn pin(m: &AppModel, workspace: &str, pinned: bool) -> yog::boundary::reply::Workspaces {
    let reply = crate::support::act(
        m,
        &yog::boundary::Action::Pin {
            workspace: workspace.to_owned(),
            pinned,
        },
    );
    match reply {
        Ok(yog::boundary::reply::Reply::Workspaces(rows)) => rows,
        other => unreachable!("a pin lands and answers the chrome it ranked: {other:?}"),
    }
}

/// STORIES **S4-T7** tab-strip.
#[test]
fn s4_t7_pins_hoist_kinds_overflow_and_every_badge_is_its_own_rollup() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        litany_data: root.path().join("litany"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    // Three NAMED workspaces (§3.1: the leaf is the name), laid in an order that
    // is neither the name order nor the pin order, so neither can pass by luck.
    let names_root = roots.yog_data.join("workspaces");
    let named = |name: &str| names_root.join(name);
    for (name, agents) in [
        ("delta", vec![stirring("d-001")]),
        ("alpha", vec![stirring("a-001"), stirring("a-002")]),
        ("charlie", vec![quiet("c-001")]),
    ] {
        let ws = named(name);
        std::fs::create_dir_all(&ws).unwrap();
        build_agents(&ws, &agents);
    }
    // A FOREIGN workspace (litany's auto-id territory) and a REPLAY one — real
    // workspaces, but not spheres the operator named, so neither tabs the wall.
    let foreign = roots.litany_data.join("workspaces").join("ws-7f3a");
    std::fs::create_dir_all(&foreign).unwrap();
    build_agents(&foreign, &[stirring("f-001")]);
    let replay = roots.litany_data.join("replays").join("rp-1");
    std::fs::create_dir_all(&replay).unwrap();
    build_agents(&replay, &[stirring("r-001")]);

    let (m, _worker) = AppModel::boot(
        roots.clone(),
        Arc::new(SystemClock),
        Box::new(FakeBl::default()),
        None,
    );

    // --- Unpinned: the wall is the named workspaces in NAME order.
    let bar = crate::support::tab_bar(&m, None);
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        ["alpha", "charlie", "delta"],
        "name order by default"
    );
    assert!(
        bar.tabs.iter().all(|t| t.kind == Kind::Named),
        "only named workspaces tab the wall"
    );

    // Each badge is its OWN workspace's rollup — never the strip's total, and
    // never a shared number: alpha stirs twice, delta once, charlie not at all.
    let badge = |name: &str| bar.tabs.iter().find(|t| t.name == name).unwrap().attention;
    assert_eq!(badge("alpha"), 2);
    assert_eq!(badge("delta"), 1);
    assert_eq!(badge("charlie"), 0, "an abandoned rest stirs nothing");

    // --- The overflow: foreign and replay, with their aggregate on the button.
    let kinds: Vec<Kind> = bar.overflow.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        [Kind::Foreign, Kind::Replay],
        "foreign and replay are real, but not regimes"
    );
    assert_eq!(
        bar.overflow_attention(),
        2,
        "the ⋯ button carries what the wall is not showing"
    );
    // The strip total spans EVERY workspace, overflow included — the wall hides
    // rows, never facts (§6).
    assert_eq!(
        crate::support::strip_total(&m),
        5,
        "2 alpha + 1 delta + 1 foreign + 1 replay"
    );

    // --- Pinned: hoisted in PIN order, ahead of the name-ordered remainder.
    // A pin is durable operator state in `ui.json` (§4.1), and since bl-b986 it
    // is written by a **boundary act** rather than by a hand-edited document —
    // which is the whole of what that ball was: every seat had read the rank
    // since bl-296f and nothing had written one since bl-7942 took the window's
    // tab strip. Two acts, in pin order; the second instance below then reads
    // the same document, which is how one seat's pin reaches another (I0:
    // restart is re-read).
    for (rank, name) in ["delta", "charlie"].into_iter().enumerate() {
        let seen = pin(&m, name, true);
        // The receipt is the listing the pin ranks, re-derived after the write.
        assert_eq!(
            seen.rows
                .iter()
                .find(|r| r.workspace == name)
                .and_then(|r| r.pinned),
            Some(rank),
            "{name} carries the rank the act just gave it"
        );
    }
    // The set is idempotent and says what it means (the ruling, bl-b986):
    // unpinning one that was never pinned leaves the float alone, so two seats
    // sending the same instruction agree where two toggles would cancel.
    pin(&m, "alpha", false);
    drop(m);
    let (m, _worker) = AppModel::boot(
        roots,
        Arc::new(SystemClock),
        Box::new(FakeBl::default()),
        None,
    );
    let bar = crate::support::tab_bar(&m, None);
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        ["delta", "charlie", "alpha"],
        "pin order first (delta was pinned first), then name order"
    );
    assert!(bar.tabs[0].pinned && bar.tabs[1].pinned && !bar.tabs[2].pinned);
    // Pinning hoists one out; it does not change what its badge counts.
    let badge = |name: &str| bar.tabs.iter().find(|t| t.name == name).unwrap().attention;
    assert_eq!(badge("delta"), 1);
    assert_eq!(badge("alpha"), 2);

    a_set_is_idempotent_and_says_which_way(&m);
}

/// The set's own two remaining moods, on the instance that READ the document
/// rather than the one that wrote it (bl-b986). Re-pinning something already
/// pinned moves it to the end of the float rather than saying it twice — the
/// order IS the assertion — and an unpin takes one back out. Both are the same
/// instruction said again, which is what makes the act safe for two seats to
/// send at once, and what a toggle could never be.
fn a_set_is_idempotent_and_says_which_way(m: &AppModel) {
    pin(m, "delta", true);
    let bar = crate::support::tab_bar(m, None);
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["charlie", "delta", "alpha"], "re-pinned last");
    pin(m, "charlie", false);
    let bar = crate::support::tab_bar(m, None);
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["delta", "alpha", "charlie"], "unpinned falls back");
    let pinned: Vec<bool> = bar.tabs.iter().map(|t| t.pinned).collect();
    assert_eq!(pinned, [true, false, false], "only delta is still floated");
}
