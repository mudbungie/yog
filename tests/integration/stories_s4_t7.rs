//! STORIES **S4-T7** tab-strip: the tab strip is the sphere wall — pinned tabs
//! hoist **in pin order** ahead of name order, each tab's badge is its own
//! workspace's attention rollup, foreign and replay workspaces fall to the ⋯
//! overflow (real, but not regimes), and the overflow button carries their
//! aggregate (STORIES S4.7, DESIGN §3.1, §11).
//!
//! Driven off `AppModel::tab_bar`, which is where the three inputs meet: the
//! §3.1 workspace classification (which root a directory sits under), the §6
//! per-workspace rollup, and the `ui.json` pin order.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents};
use std::sync::Arc;
use tempfile::tempdir;
use yog::nav::tabs::Kind;
use yog::nav::ws_key;
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

/// STORIES **S4-T7** tab-strip.
#[test]
fn s4_t7_pins_hoist_kinds_overflow_and_every_badge_is_its_own_rollup() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
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
    // A FOREIGN workspace (lernie's auto-id territory) and a REPLAY one — real
    // workspaces, but not spheres the operator named, so neither tabs the wall.
    let foreign = roots.lernie_data.join("workspaces").join("ws-7f3a");
    std::fs::create_dir_all(&foreign).unwrap();
    build_agents(&foreign, &[stirring("f-001")]);
    let replay = roots.lernie_data.join("replays").join("rp-1");
    std::fs::create_dir_all(&replay).unwrap();
    build_agents(&replay, &[stirring("r-001")]);

    let (mut m, _worker) = AppModel::boot(
        roots,
        None,
        Arc::new(SystemClock),
        Box::new(FakeBl::default()),
        None,
    );

    // --- Unpinned: the wall is the named workspaces in NAME order.
    let bar = m.tab_bar();
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
        m.strip_total(),
        5,
        "2 alpha + 1 delta + 1 foreign + 1 replay"
    );

    // --- Pinned: hoisted in PIN order, ahead of the name-ordered remainder.
    m.toggle_pin(&ws_key(&named("delta")));
    m.toggle_pin(&ws_key(&named("charlie")));
    let bar = m.tab_bar();
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
}
