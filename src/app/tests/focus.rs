//! Roster / attention-strip / seen-acknowledgement tests, and the M2
//! convergence milestone (§6, §11, §15 Y11).

use super::Harness;
use crate::keymap::InspectorTab;
use crate::nav::ws_key;
use crate::ui_state::SeenKind;
use crate::watch::Mark;

#[test]
fn focus_workspace_selects_no_agent_and_records_nothing() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    assert_eq!(model.focused_workspace(), Some(h.ws.clone()));
    assert!(model.focus().agent.is_none());
    assert_eq!(model.strip_total(), 1, "focusing a workspace acks nothing");
}

#[test]
fn focus_agent_acknowledges_the_stop_watermark() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let tip = model.tree(&h.ws).unwrap().agents[0].tip_oid.clone();
    let key = ws_key(&h.ws);
    assert!(!model.is_seen(SeenKind::Stopped, &key, "c-1", &tip));
    model.focus_agent(&h.ws, "c-1");
    assert_eq!(model.focus().agent.as_deref(), Some("c-1"));
    assert!(
        model.is_seen(SeenKind::Stopped, &key, "c-1", &tip),
        "recorded on focus"
    );
    assert_eq!(
        model.strip_total(),
        0,
        "the acked stop no longer draws attention"
    );
}

#[test]
fn focus_agent_on_a_missing_agent_is_a_no_op() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.focus_agent(&h.ws, "nope");
    assert_eq!(
        model.strip_total(),
        1,
        "no watermark recorded for a phantom"
    );
}

#[test]
fn jump_to_next_attention_focuses_and_acknowledges_it() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // Startup focus is the workspace (no agent), so the jump starts from the
    // front and lands on the attention-bearing agent, acknowledging it.
    model.jump_next_attention();
    assert_eq!(model.focus().agent.as_deref(), Some("c-1"));
    assert_eq!(model.strip_total(), 0, "jumping acknowledges");
    // A second jump, now focused with an agent, finds nothing → no change.
    model.jump_next_attention();
    assert_eq!(model.focus().agent.as_deref(), Some("c-1"));
}

#[test]
fn inspector_tab_selection_is_sticky_across_focus_changes() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    assert_eq!(
        model.inspector_tab(),
        InspectorTab::Transcript,
        "Transcript is the default tab"
    );
    model.select_tab(InspectorTab::Steps);
    assert_eq!(model.inspector_tab(), InspectorTab::Steps);
    // The selection survives both a workspace focus and an agent focus (§11).
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    assert_eq!(model.inspector_tab(), InspectorTab::Steps);
    model.focus_agent(&h.ws, "c-1");
    assert_eq!(
        model.inspector_tab(),
        InspectorTab::Steps,
        "sticky through the acknowledgement gesture"
    );
}

#[test]
fn toggle_pin_hoists_and_unhoists_the_workspace_tab() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // The pin toggle takes the §3.1 NAME (bl-7407) and resolves the `ui.json`
    // path key behind it — the durable key did not move, the door did.
    let key = crate::naming::leaf(&h.ws);
    // The foreign workspace starts in the overflow; pinning hoists it (§11).
    assert!(model.tab_bar().tabs.is_empty());
    model.toggle_pin(&key);
    let bar = model.tab_bar();
    assert_eq!(bar.tabs.len(), 1);
    assert!(bar.tabs[0].pinned);
    // It stays listed in the menu with ★ lit (bl-7e32): pinning changes where an
    // entry ALSO appears, never where it lives — which is what gives unpin a
    // visible carrier and demotes the tab menu's unpin to an accelerator (§11).
    assert_eq!(bar.overflow.len(), 1, "still its home");
    assert!(bar.overflow[0].pinned, "listed with the ★ lit");
    model.toggle_pin(&key);
    assert!(model.tab_bar().tabs.is_empty(), "unpinned again");
    let bar = model.tab_bar();
    assert_eq!(bar.overflow.len(), 1);
    assert!(!bar.overflow[0].pinned, "★ goes dark");
}

#[test]
fn set_collapsed_overrides_a_section() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // The balls section's persisted fold (§4.1/§11): default open, override
    // sticks through the ui.json write path.
    assert!(!model.is_collapsed("balls"));
    model.set_collapsed("balls", true);
    assert!(model.is_collapsed("balls"));
    model.set_collapsed("balls", false);
    assert!(!model.is_collapsed("balls"));
}

/// **The Z9 proof on the live fixture shape** (§6/§11): a conversation whose
/// latest step failed on auth (the stench-pug `kind:auth` response) classifies
/// Stopped, stirs the strip while unseen, and keys the Login affordance;
/// acknowledging clears the *signal*, never the settled failure the badge and
/// banner keep rendering.
#[test]
fn an_auth_failed_conversation_stirs_the_strip_and_flags_login() {
    let h = Harness::new();
    h.write_response(
        "c-1",
        b"{\"type\":\"error\",\"kind\":\"auth\",\"message\":\"no credential for this provider: run `bz --login --provider <id>`\",\"provider_detail\":null}\n{\"type\":\"end\"}\n",
    );
    let (_c, mut model) = h.model();
    assert_eq!(
        model.strip_total(),
        1,
        "the dead conversation stirs (§6 rule 2)"
    );
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    let rows = model.conversations(10);
    assert_eq!(rows[0].state, crate::git_tree::AgentState::Stopped);
    assert_eq!(rows[0].attention, 1);
    let steps_of = || crate::steps_view::build(&h.ws, "c-1", crate::git_tree::AgentState::Stopped);
    assert!(
        crate::login::auth::latest_step_auth_failed(&steps_of()).offered(),
        "the center's Login banner keys on the same response"
    );
    // Acknowledge (§6): the signal clears; the settled failure still renders.
    model.focus_agent(&h.ws, "c-1");
    assert_eq!(model.strip_total(), 0, "acknowledged");
    let rows = model.conversations(10);
    assert_eq!(rows[0].state, crate::git_tree::AgentState::Stopped);
    assert!(crate::login::auth::latest_step_auth_failed(&steps_of()).offered());
}

/// bl-b54e at the model boundary: a gesture is on disk when it returns, with
/// no flush call and no tick in between — so a SIGTERM here loses nothing.
#[test]
fn a_gesture_is_on_disk_when_it_returns() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let key = crate::nav::ws_key(&h.ws);
    model.toggle_pin(&crate::naming::leaf(&h.ws));
    let back = std::fs::read_to_string(h.roots.ui_json()).unwrap();
    assert!(back.contains(&key), "the pin is already durable: {back}");
}

/// **M2 milestone (§15 Y11):** two instances against one fixture set converge
/// on the seen watermark through `ui.json` — instance A's acknowledgement,
/// written to the file, is adopted by instance B via the fs-event path, so B's
/// attention for that agent clears. No windows; fakes + tempdirs only.
#[test]
fn m2_two_instances_converge_on_seen_through_ui_json() {
    let h = Harness::new();
    let (_ca, mut a) = h.model();
    let (_cb, mut b) = h.model();
    assert_eq!(a.strip_total(), 1, "both see the unseen stop");
    assert_eq!(b.strip_total(), 1);

    // A acknowledges by focusing the stopped agent — write-through to ui.json.
    a.focus_agent(&h.ws, "c-1");
    assert_eq!(a.strip_total(), 0, "A's own view clears");
    assert_eq!(b.strip_total(), 1, "B has not adopted yet");

    // B's yog-state watcher fires: B adopts A's ui.json wholesale.
    b.dirty_handle()
        .mark_all([(h.roots.yog_state.clone(), Mark::Watch)]);
    b.tick();
    assert_eq!(b.strip_total(), 0, "B converged on the acknowledgement");
    let tip = b.tree(&h.ws).unwrap().agents[0].tip_oid.clone();
    let key = ws_key(&h.ws);
    assert!(
        b.is_seen(SeenKind::Stopped, &key, "c-1", &tip),
        "the concrete watermark converged, not just the count",
    );
}
