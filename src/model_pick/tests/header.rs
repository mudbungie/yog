//! The conversation's model row and its §9.4 drift clause (bl-9786, re-shaped
//! by bl-cd2a), plus the §11 birth-config block's row (bl-824e).

use super::TEMPLATE_PROVIDERS;
use crate::model_pick::{
    ConfigPoint, NEW_CONVERSATION_EXIT, RETARGET_EXIT, RETARGET_HOVER, birth_row, conversation_row,
    row_role,
};

/// The governing commit as the fixtures see it: lernie's own template, forked
/// at `1a2b3c4d`.
fn governing() -> ConfigPoint {
    point("1a2b3c4dfeed", "1a2b3c4d", TEMPLATE_PROVIDERS)
}

fn point(oid: &str, short_oid: &str, providers_yaml: &str) -> ConfigPoint {
    ConfigPoint {
        oid: oid.to_string(),
        short_oid: short_oid.to_string(),
        providers_yaml: providers_yaml.to_string(),
    }
}

/// The template with the worker moved to another model — what an advanced
/// workspace default looks like.
fn advanced(model: &str) -> ConfigPoint {
    point(
        "9f8e7d6cbeef",
        "9f8e7d6c",
        &TEMPLATE_PROVIDERS.replace("model: gpt-5.4\n", &format!("model: {model}\n")),
    )
}

/// The row IS the selection (bl-cd2a): the two dropdowns carry the pair and the
/// line carries nothing else — no `model ·` prefix, no `frozen at <oid>`. The
/// freeze rides the hover instead, where it costs the line nothing.
#[test]
fn the_row_is_the_pair_and_the_freeze_rides_the_hover() {
    let gov = governing();
    let row = conversation_row(&gov, &gov, "worker");
    assert_eq!(row.provider, "codex");
    assert_eq!(row.model, "gpt-5.4");
    assert!(!row.drifted());
    assert!(row.hover.contains("1a2b3c4d"));
    assert!(row.hover.contains("next conversation"));
    assert!(row.hover.contains("frozen"));
}

/// What the dropdowns show is what they WRITE — the workspace default, not the
/// freeze. Showing the frozen pair would report the operator's own write back
/// as a no-op: the tip moves and the freeze cannot.
#[test]
fn the_dropdowns_show_the_tip_the_pick_advances_not_the_frozen_pair() {
    let row = conversation_row(&governing(), &advanced("gpt-5.6-sol"), "worker");
    assert_eq!(row.model, "gpt-5.6-sol");
}

/// The reported bug (bl-9786) keeps its answer: an advanced default does not
/// silently replace the freeze on screen — the clause names what this
/// conversation is actually frozen on, and only then.
#[test]
fn an_advanced_workspace_default_names_the_frozen_pair_beside_it() {
    let row = conversation_row(&governing(), &advanced("gpt-5.6-sol"), "worker");
    assert_eq!(
        row.drift.as_deref(),
        Some("this conversation is frozen on codex · gpt-5.4 at 1a2b3c4d")
    );
    assert!(row.drifted());
}

/// Drift is the OIDS differing, not the models: a config commit that moved
/// something else still parted this conversation from the default, and saying
/// so keeps the row honest instead of special-casing the coincidence.
#[test]
fn a_moved_default_carrying_the_same_model_still_drifts() {
    let row = conversation_row(&governing(), &advanced("gpt-5.4"), "worker");
    assert!(row.drifted());
    assert!(
        row.drift
            .unwrap_or_default()
            .contains("gpt-5.4 at 1a2b3c4d")
    );
}

/// The same commit on both sides is no drift — the common case, and the one the
/// operator asked to be nothing but the pair.
#[test]
fn a_conversation_on_the_current_default_carries_no_clause() {
    let gov = governing();
    assert!(conversation_row(&gov, &gov, "worker").drift.is_none());
}

/// A role the file does not declare is absent, and absence is a value: the row
/// still paints, so the dropdowns are still reachable to fix it with.
#[test]
fn a_row_for_an_undeclared_role_paints_the_absence() {
    let row = conversation_row(&governing(), &governing(), "nonesuch");
    assert_eq!(row.provider, "(none)");
    assert_eq!(row.model, "(none)");
    let empty = point("bbb", "abc", "");
    let row = conversation_row(&empty, &empty, "worker");
    assert_eq!(row.model, "(none)");
}

/// The exit says what it does, and what it does is start over — a NEW
/// conversation, never adoption by this one.
#[test]
fn the_exit_label_promises_a_new_conversation_not_adoption() {
    assert!(NEW_CONVERSATION_EXIT.contains("new conversation"));
    assert!(NEW_CONVERSATION_EXIT.contains("current config"));
}

/// The exit beside it (bl-2d19) is the one that keeps the conversation: it
/// names the *move*, not a new start, and its hover carries the three facts an
/// operator needs before spending it — the substrate verb, when it lands, and
/// that nothing is discarded — plus the §8.5 line that fires it without a
/// mouse (§11 discoverability rule 3).
#[test]
fn the_retarget_exit_moves_this_conversation_and_keeps_it() {
    assert!(RETARGET_EXIT.contains("move this conversation"));
    assert!(RETARGET_EXIT.contains("current config"));
    assert!(!RETARGET_EXIT.contains("new conversation"));
    assert!(RETARGET_HOVER.contains("lernie retarget"));
    assert!(RETARGET_HOVER.contains("next step"));
    assert!(RETARGET_HOVER.contains("keeps every message"));
    assert!(RETARGET_HOVER.contains("/retarget"));
}

/// The birth row is the same pair on the same two dropdowns, minus a clause a
/// conversation that does not exist yet cannot have.
#[test]
fn the_birth_row_names_the_pair_and_the_branch_head_it_forks() {
    let row = birth_row(&governing(), "worker", "default");
    assert_eq!(
        (row.provider.as_str(), row.model.as_str()),
        ("codex", "gpt-5.4")
    );
    assert!(row.drift.is_none());
    assert!(row.hover.contains("forks the head of config/default"));
    assert!(row.hover.contains("1a2b3c4d"));
    // The whole reason the pick is a workspace write, said where it is asked.
    assert!(row.hover.contains("no per-conversation config"));
}

/// A tip with no worker role renders like the conversation row does: absence is
/// a value, and the dropdowns are still there to answer it with.
#[test]
fn a_birth_tip_with_no_worker_role_still_paints_the_row() {
    let none = point(
        "aaa",
        "abc",
        "roles:\n  compactor:\n    provider: p\n    model: m\n",
    );
    assert_eq!(birth_row(&none, "worker", "default").provider, "(none)");
}

/// A bare row is the role that talks to you; the picker's role strip is the one
/// thing that re-scopes it, so the fallback lives with the row, not in a
/// frontend.
#[test]
fn the_bare_row_reports_the_worker_role() {
    assert_eq!(row_role(None), "worker");
    assert_eq!(row_role(Some("compactor")), "compactor");
}
