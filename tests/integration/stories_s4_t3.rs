//! STORIES **S4-T3** join-rows: one fixture exercising the §3.5 row states in a
//! single project — two **Bound** balls under one workspace, a **ready** ball, a
//! **claimed-elsewhere** ball, and an **unassigned** workspace — and the balls
//! section groups **all** the bound balls under their claimant workspace (the
//! wave-1 multi-ball fix: no arbitrary single badge, no Delivered shadowing a
//! Bound). Carries the bl-9cb0 negative: focusing the unassigned workspace
//! focuses **no** ball, so the composer's ball row (§8.2) and the marks knob
//! (§16.3) — both `focused_join` consumers — withhold themselves.
//! DESIGN §3.5, §11, §16.3, §15 M6 Z4.

#![allow(clippy::unwrap_used)]

use crate::support::FakeBl;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// The §3.5 row states in one project: two balls bound to the local workspace
/// "cobalt", a ready (unclaimed) ball, and a ball claimed by a non-local name.
const LIST: &str = r#"[
    {"id":"bl-1","title":"One","claimant":"cobalt"},
    {"id":"bl-2","title":"Two","claimant":"cobalt"},
    {"id":"bl-3","title":"Ready"},
    {"id":"bl-4","title":"Boss","claimant":"boss"}
]"#;

/// STORIES **S4-T3** join-rows.
#[test]
fn s4_t3_balls_section_groups_all_bound_balls_under_their_workspace() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    // A clone whose percent-encoded basename decodes to /proj/a (FakeBl never
    // chdirs, so the decoded path need not exist).
    std::fs::create_dir_all(roots.balls_clones.join("%2Fproj%2Fa")).unwrap();
    // Two yog-named workspaces (§3.1, leaf = name): "cobalt" (bound) and "spare"
    // (unassigned — no ball claims it).
    for name in ["cobalt", "spare"] {
        std::fs::create_dir_all(
            roots
                .yog_data
                .join("workspaces")
                .join(name)
                .join("repo.git"),
        )
        .unwrap();
    }
    let live = HashMap::from([(PathBuf::from("/proj/a"), LIST.to_owned())]);
    let (mut m, _worker) = AppModel::boot(
        roots,
        None,
        Arc::new(SystemClock),
        Box::new(FakeBl {
            live,
            ..FakeBl::default()
        }),
        None,
    );

    // Both named workspaces are wall tabs (§11), in name order.
    let bar = m.tab_bar();
    let names: Vec<&str> = bar.tabs.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["cobalt", "spare"], "named workspaces tab the wall");

    // cobalt groups BOTH its bound balls — the multi-ball fix (§3.5); each Bound
    // needs no badge.
    let cobalt = m.ws_balls(&bar.tabs[0].ws);
    let ids: Vec<&str> = cobalt.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, vec!["bl-1", "bl-2"], "all bound balls, grouped");
    assert!(cobalt.iter().all(|b| b.badge.is_none()));

    // spare is the §3.5 unassigned-workspace row — full rendering, no ball column.
    assert!(
        m.ws_balls(&bar.tabs[1].ws).is_empty(),
        "unassigned workspace has no ball"
    );
    // The negative beat (bl-9cb0): focusing that workspace focuses NO ball, so
    // the composer's ball row (§8.2) and the per-project marks knob (§16.3) —
    // both `focused_join` consumers — render their empty state instead of a row
    // naming an empty ball and an empty project.
    m.focus_workspace(&bar.tabs[1].ws);
    assert!(
        m.focused_join().is_none(),
        "no ball claims spare ⇒ no ball row, no marks knob"
    );
    m.focus_workspace(&bar.tabs[0].ws);
    assert_eq!(
        m.focused_join().map(|r| r.ball_id.clone()),
        Some("bl-1".to_owned()),
        "a bound workspace still focuses its ball"
    );

    // The ready, unclaimed ball is startable; the claimed-elsewhere one is not
    // (§3.5) — one ▶ Start entry, bl-3.
    let startable = m.startable();
    assert_eq!(startable.len(), 1, "only the ready ball is startable");
}
