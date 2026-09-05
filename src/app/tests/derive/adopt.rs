//! **What a pass adopts off the yog-state root** — the three announcement-driven
//! reads, split off [`super`] at the cap on the seam they share: each is a file
//! the operator hand-edits, re-read when that root says so and never on a tick
//! of its own, and each is severable — deleting the file deletes the policy
//! rather than reaching a code path.

use super::super::Harness;
use crate::watch::Mark;
use std::time::Duration;

/// The clock's one setting, end to end (bl-3381): a `cadence.yaml` present at
/// boot tunes the first pass; a change re-tunes on its own announcement and
/// rides the published snapshot; deleting the file is the reset to defaults.
#[test]
fn cadence_yaml_tunes_the_clock_at_boot_on_change_and_resets_on_delete() {
    let h = Harness::new();
    let file = h.roots.yog_state.join(crate::app::cadence::CADENCE_YAML);
    std::fs::write(&file, "cadence:\n  watcher:\n    cheap_sweep_ms: 5000\n").unwrap();
    let (_c, mut model) = h.model();
    assert_eq!(
        model.cadence().cheap_sweep,
        Duration::from_secs(5),
        "boot adopted the file before the first schedule decision"
    );
    std::fs::write(&file, "cadence:\n  watcher:\n    cheap_sweep_ms: 9000\n").unwrap();
    model
        .dirty_handle()
        .mark_all([(h.roots.yog_state.clone(), Mark::Watch)]);
    assert!(model.tick(), "a re-tune publishes");
    assert_eq!(model.cadence().cheap_sweep, Duration::from_secs(9));
    std::fs::remove_file(&file).unwrap();
    model
        .dirty_handle()
        .mark_all([(h.roots.yog_state.clone(), Mark::Watch)]);
    assert!(model.tick(), "the reset publishes too");
    assert_eq!(
        model.cadence(),
        crate::app::Cadence::default(),
        "deleting the file is the reset — severability, not an error"
    );
}

#[test]
fn adopting_our_own_ui_json_echo_is_suppressed() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // Pin the real workspace (write-through to ui.json), then let the state
    // root fire.
    // `ui.json` keys pins by PATH (§4.1) — durable state whose re-keying would
    // be its own migration — while a tab is addressed by its §3.1 name.
    let key = crate::naming::leaf(&h.ws);
    model.ui.set_pinned(vec![crate::nav::ws_key(&h.ws)]);
    model
        .dirty_handle()
        .mark_all([(h.roots.yog_state.clone(), Mark::Watch)]);
    model.tick(); // adopt_ui reads our own bytes → is_echo true → skip adopt
    let bar = model.tab_bar(Some(&key));
    assert!(
        bar.tabs.first().is_some_and(|t| t.pinned),
        "the pin survives the echo: {bar:?}"
    );
}

/// The armed loops the published snapshot carries — the same `board::build` the
/// §8.5 `Query::Board` answers and the §11 fold now paints over the wire
/// (bl-adcb retired the model's own `board()` with that migration). Spelled here
/// because the subject of the test below is the **worker's** side: what the
/// derivation adopted, read through the derivation that reads it.
fn fleet(model: &crate::AppModel) -> Vec<crate::fleet::Facts> {
    crate::board::build(&model.snap, &model.ui, model.now_unix()).fleet
}

/// The §4.3 fleet arming rides the clock's own file and the clock's own
/// announcement (bl-66fb): an entry published on the snapshot the board reads,
/// deleted the same way. The **burden check from the worker's side** — with no
/// entry, the published snapshot carries no loop at all.
#[test]
fn a_fleet_entry_is_adopted_and_deleted_on_the_clocks_own_announcement() {
    let h = Harness::new();
    let file = h.roots.yog_state.join(crate::app::cadence::CADENCE_YAML);
    let (_c, mut model) = h.model();
    assert!(
        fleet(&model).is_empty(),
        "unarmed: the board is today's balls section"
    );
    std::fs::write(
        &file,
        "fleet:\n  /ws/a:\n    project: /dev/yog\n    cap: 3\n",
    )
    .unwrap();
    model
        .dirty_handle()
        .mark_all([(h.roots.yog_state.clone(), Mark::Watch)]);
    assert!(model.tick(), "an arming publishes");
    let armed = fleet(&model);
    assert_eq!(armed.len(), 1);
    assert_eq!(armed[0].cap, 3);
    assert_eq!(armed[0].project, std::path::PathBuf::from("/dev/yog"));
    std::fs::remove_file(&file).unwrap();
    model
        .dirty_handle()
        .mark_all([(h.roots.yog_state.clone(), Mark::Watch)]);
    assert!(model.tick(), "the disarming publishes too");
    assert!(
        fleet(&model).is_empty(),
        "deleting the entry deletes the loop, not a code path"
    );
}

/// **Another instance's `ui.json` is adopted wholesale** (§4.1, I5) — the other
/// side of the echo suppression above, and the whole of I0's convergence: two
/// yogs over one document agree because each takes what the worker read, and
/// only its own bytes are skipped.
#[test]
fn an_external_ui_json_write_is_adopted() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let key = crate::naming::leaf(&h.ws);
    assert!(
        model
            .ws_listing()
            .rows
            .iter()
            .all(|row| row.pinned.is_none()),
        "nothing is pinned yet"
    );
    // A write this instance did not make: another yog's, or an operator's
    // editor. `ui.json` keys pins by path (§4.1), which is why the fixture
    // writes the key rather than the name.
    crate::ui_state::UiState::open(model.ui_json_path())
        .set_pinned(vec![crate::nav::ws_key(&h.ws)]);
    model
        .dirty_handle()
        .mark_all([(h.roots.yog_state.clone(), Mark::Watch)]);
    assert!(model.tick(), "the re-read publishes");
    let bar = model.tab_bar(Some(&key));
    assert!(
        bar.tabs.first().is_some_and(|t| t.pinned),
        "the other instance's pin is this one's too: {bar:?}"
    );
}
