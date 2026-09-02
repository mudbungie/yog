//! The conversation's model row and its §9.4 apart clause (bl-9786, re-shaped
//! by bl-cd2a, inverted by bl-e654), plus the §11 birth-config block's row
//! (bl-824e).

use super::TEMPLATE_PROVIDERS;
use crate::model_pick::{
    ConfigPoint, RETARGET_EXIT, RETARGET_HOVER, birth_row, conversation_row, row_role,
};

/// The commit a conversation resolves, as the fixtures see it: litany's own
/// template, standing at `1a2b3c4d`.
fn resolved() -> ConfigPoint {
    point("1a2b3c4dfeed", "1a2b3c4d", TEMPLATE_PROVIDERS)
}

fn point(oid: &str, short_oid: &str, providers_yaml: &str) -> ConfigPoint {
    ConfigPoint {
        oid: oid.to_string(),
        short_oid: short_oid.to_string(),
        providers_yaml: providers_yaml.to_string(),
    }
}

/// The template with the worker moved to another model — what this workspace's
/// lineage tip looks like once a pick has advanced it.
fn advanced(model: &str) -> ConfigPoint {
    point(
        "9f8e7d6cbeef",
        "9f8e7d6c",
        &TEMPLATE_PROVIDERS.replace("model: gpt-5.4\n", &format!("model: {model}\n")),
    )
}

/// The row IS the selection (bl-cd2a): the two dropdowns carry the pair and the
/// line carries nothing else — no `model ·` prefix, no oid. The commit the
/// conversation resolves rides the hover instead, where it costs the line
/// nothing.
#[test]
fn the_row_is_the_pair_and_the_resolved_commit_rides_the_hover() {
    let point = resolved();
    let row = conversation_row(&point, &point, "worker");
    assert_eq!(row.provider, "codex");
    assert_eq!(row.model, "gpt-5.4");
    assert!(!row.is_apart());
    assert!(row.hover.contains("1a2b3c4d"));
    assert!(row.hover.contains("at its next step"));
    assert!(
        !row.hover.contains("frozen"),
        "the hover may not promise a freeze the engine stopped keeping: {}",
        row.hover
    );
}

/// **The inversion, at the surface** (bl-e654). The dropdowns show the lineage
/// tip, and under follow-the-tip that is also what the conversation on screen
/// runs from its next step — so a pick that advanced the lineage is reported
/// as the conversation's own pair, with no clause and nothing to press.
#[test]
fn a_conversation_following_the_lineage_wears_the_advanced_pair_with_no_clause() {
    let tip = advanced("gpt-5.6-sol");
    let row = conversation_row(&tip, &tip, "worker");
    assert_eq!(row.model, "gpt-5.6-sol");
    assert!(!row.is_apart());
    assert!(row.apart.is_none());
}

/// bl-9786's lesson outlives its doctrine: the clause still fires the moment
/// the two commits part, because the surprise arrives at the moment of the
/// READ. What it now means is that this conversation resolves something else —
/// held on a divergence, or following another lineage — which is exactly the
/// state `retarget` settles.
#[test]
fn a_conversation_resolving_another_commit_names_its_own_pair_beside_it() {
    let row = conversation_row(&resolved(), &advanced("gpt-5.6-sol"), "worker");
    assert_eq!(
        row.apart.as_deref(),
        Some(
            "this conversation resolves codex · gpt-5.4 at 1a2b3c4d, not this workspace's lineage"
        )
    );
    assert!(row.is_apart());
}

/// Apartness is the OIDS differing, not the models: a commit that moved
/// something else is still not this workspace's lineage tip, and saying so
/// keeps the row honest instead of special-casing the coincidence.
#[test]
fn a_lineage_tip_carrying_the_same_model_is_still_apart() {
    let row = conversation_row(&resolved(), &advanced("gpt-5.4"), "worker");
    assert!(row.is_apart());
    assert!(
        row.apart
            .unwrap_or_default()
            .contains("gpt-5.4 at 1a2b3c4d")
    );
}

/// The same commit on both sides carries no clause — the common case, and the
/// one the operator asked to be nothing but the pair.
#[test]
fn a_conversation_on_this_workspaces_lineage_carries_no_clause() {
    let point = resolved();
    assert!(conversation_row(&point, &point, "worker").apart.is_none());
}

/// A role the file does not declare is absent, and absence is a value: the row
/// still paints, so the dropdowns are still reachable to fix it with.
#[test]
fn a_row_for_an_undeclared_role_paints_the_absence() {
    let row = conversation_row(&resolved(), &resolved(), "nonesuch");
    assert_eq!(row.provider, "(none)");
    assert_eq!(row.model, "(none)");
    let empty = point("bbb", "abc", "");
    let row = conversation_row(&empty, &empty, "worker");
    assert_eq!(row.model, "(none)");
}

/// **The clause has ONE exit** (bl-e654). Its old peer offered to start over,
/// which was the escape from a freeze; there is no freeze, so it is gone and
/// the label must not promise one. What is left names the *settlement*, and its
/// hover carries the four facts an operator needs before spending it — that
/// they do not need it for an ordinary edit, the substrate verb, when it lands,
/// and that nothing is discarded — plus the §8.5 line that fires it without a
/// mouse (§11 discoverability rule 3).
#[test]
fn the_one_exit_settles_this_conversation_and_keeps_it() {
    assert!(RETARGET_EXIT.contains("settle this conversation"));
    assert!(RETARGET_EXIT.contains("config lineage"));
    assert!(!RETARGET_EXIT.contains("new conversation"));
    assert!(RETARGET_HOVER.contains("litany retarget"));
    assert!(RETARGET_HOVER.contains("reaches it on its own"));
    assert!(RETARGET_HOVER.contains("next step"));
    assert!(RETARGET_HOVER.contains("keeps every message"));
    assert!(RETARGET_HOVER.contains("/retarget"));
}

/// The birth row is the same pair on the same two dropdowns, minus a clause a
/// conversation that does not exist yet cannot have.
#[test]
fn the_birth_row_names_the_pair_and_the_branch_head_it_forks() {
    let row = birth_row(&resolved(), "worker", "default");
    assert_eq!(
        (row.provider.as_str(), row.model.as_str()),
        ("codex", "gpt-5.4")
    );
    assert!(row.apart.is_none());
    assert!(row.hover.contains("forks the head of config/default"));
    assert!(row.hover.contains("follows that lineage"));
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
