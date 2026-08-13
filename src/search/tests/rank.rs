//! The order and the bounds (§8.5): the three tiers, the offset within one,
//! ASCII folding that keeps the offset honest, the empty query, and the cap.

use super::*;

#[test]
fn ranking_is_name_then_summary_then_text_then_offset_then_address() {
    let ws = PathBuf::from("/w/void");
    let snap = world(
        &ws,
        vec![],
        vec![
            ball("bl-body", "unrelated", "a long line then void here"),
            ball("bl-title", "the void", "nothing"),
            ball("void-id", "unrelated", "nothing"),
        ],
        vec![],
    );
    let found = run(&snap, "void", &always());
    let ids: Vec<String> = found
        .hits
        .iter()
        .filter_map(|h| match &h.at {
            Address::Ball { id, .. } => Some(id.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ids, ["void-id", "bl-title", "bl-body"]);
    let fields: Vec<Field> = found.hits.iter().map(|h| h.field).collect();
    assert_eq!(
        fields,
        [Field::Name, Field::Name, Field::Summary, Field::Text],
        "the workspace name matches too, and sorts by offset within its tier"
    );
}

#[test]
fn matching_is_ascii_case_insensitive_and_the_offset_indexes_the_real_bytes() {
    let ws = PathBuf::from("/w/x");
    let snap = world(
        &ws,
        vec![],
        vec![ball("b", "t", "Nyarlathotep waits")],
        vec![],
    );
    let found = run(&snap, "  NYARLATHOTEP  ", &always());
    assert_eq!(found.hits.len(), 1);
    assert_eq!(found.hits[0].offset, 0);
    assert_eq!(found.hits[0].excerpt, "Nyarlathotep waits");
}

#[test]
fn an_empty_query_matches_nothing_and_clears_rather_than_refusing() {
    let ws = PathBuf::from("/w/x");
    let snap = world(&ws, vec![], vec![ball("b", "t", "body")], vec![]);
    assert_eq!(run(&snap, "   ", &always()), Found::default());
}

#[test]
fn the_bound_is_deterministic_and_caps_the_answer() {
    let ws = PathBuf::from("/w/x");
    let balls: Vec<Ball> = (0..MAX + 10)
        .map(|n| ball(&format!("bl-{n:03}"), "t", "body"))
        .collect();
    let snap = world(&ws, vec![], balls, vec![]);
    let found = run(&snap, "bl-", &always());
    assert_eq!(found.hits.len(), MAX);
    assert_eq!(
        found,
        run(&snap, "bl-", &always()),
        "same world, same answer"
    );
}

/// The answer's own "nothing to show" (bl-1ca2): both halves count, because
/// an unreadable corner of the world is something the operator must be shown
/// even when no hit survived. The §11 Search tab is offered on exactly this
/// predicate, so a `false` here is a tab that does not appear.
#[test]
fn an_answer_is_empty_only_when_both_halves_are() {
    assert!(Found::default().is_empty());
    let ws = PathBuf::from("/w/x");
    let snap = world(&ws, vec![], vec![ball("b", "gate", "body")], vec![]);
    assert!(
        !run(&snap, "gate", &always()).is_empty(),
        "a hit is content"
    );
    assert!(
        run(&snap, "nothing-matches-this", &always()).is_empty(),
        "no hit and no unreadable source is nothing to show"
    );
    let unreadable = Found {
        hits: vec![],
        unreadable: vec!["/w/x — permission denied".to_owned()],
    };
    assert!(
        !unreadable.is_empty(),
        "a gap in the corpus is content of its own"
    );
}
