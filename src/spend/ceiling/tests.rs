//! The ceiling's policy half: the forgiving read, the three ways a birth
//! flies, the refusal's two figures, and bl-a80a's world-scoped comparison.

use super::{CEILING_HEAD, Ceiling};
use crate::spend::Prices;
use serde_json::json;
use std::path::{Path, PathBuf};

const CONV: &str = "20260803T120000Z-root";

/// $1/Mtok in — so a million input tokens is exactly $1.
fn table() -> Prices {
    Prices::from_json(&json!({ "anthropic": { "opus": { "input": 1 } } }))
}

/// A workspace whose one step spent `input` tokens on the priced model.
fn spent(dir: &Path, input: u64) {
    let step = dir.join("steps").join(CONV).join("001");
    std::fs::create_dir_all(&step).unwrap();
    std::fs::write(
        step.join("response.json"),
        format!(r#"{{"type":"usage","input_tokens":{input}}}"#),
    )
    .unwrap();
    std::fs::write(step.join("request.json"), r#"{"model":"opus"}"#).unwrap();
}

/// A one-workspace world that has spent `input` tokens, plus its roster.
fn world(dir: &Path, input: u64) -> Vec<PathBuf> {
    spent(dir, input);
    vec![dir.to_path_buf()]
}

#[test]
fn an_absent_or_malformed_key_is_no_gate() {
    for value in [None, Some(json!("lots")), Some(json!(-1))] {
        let ceiling = Ceiling::from_json(value.as_ref());
        assert_eq!(ceiling, Ceiling::default());
        let dir = tempfile::tempdir().unwrap();
        let roster = world(dir.path(), 9_000_000);
        assert!(ceiling.refusal(&roster, &table()).is_none());
    }
}

#[test]
fn an_unpriced_world_gates_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let roster = world(dir.path(), 9_000_000);
    let ceiling = Ceiling::from_json(Some(&json!(1)));
    assert!(ceiling.refusal(&roster, &Prices::default()).is_none());
}

#[test]
fn under_the_ceiling_flies() {
    let dir = tempfile::tempdir().unwrap();
    let roster = world(dir.path(), 2_000_000);
    let ceiling = Ceiling::from_json(Some(&json!(2.5)));
    assert!(ceiling.refusal(&roster, &table()).is_none());
}

#[test]
fn at_the_ceiling_refuses_and_names_both_figures() {
    let dir = tempfile::tempdir().unwrap();
    let roster = world(dir.path(), 3_000_000);
    let refusal = Ceiling::from_json(Some(&json!(2.5)))
        .refusal(&roster, &table())
        .unwrap();
    assert!(refusal.contains("$3.00"), "{refusal}");
    assert!(refusal.contains("$2.50"), "{refusal}");
    assert!(refusal.contains("parks at its next tool call"), "{refusal}");
    // The fixed head the release selects marks by (bl-53d1).
    assert!(refusal.starts_with(CEILING_HEAD), "{refusal}");
}

/// **bl-a80a, the whole point.** Two workspaces, each half the ceiling and
/// each individually clear of it, refuse together: the allowance is the
/// world's and arming a second project cannot multiply it.
#[test]
fn two_workspaces_under_the_number_are_over_it_together() {
    let (one, two) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let mut roster = world(one.path(), 2_000_000);
    roster.extend(world(two.path(), 2_000_000));
    let ceiling = Ceiling::from_json(Some(&json!(2.5)));
    assert!(
        ceiling.refusal(&roster[..1], &table()).is_none(),
        "$2 alone is under a $2.50 ceiling"
    );
    let refusal = ceiling.refusal(&roster, &table()).unwrap();
    assert!(refusal.contains("$4.00"), "{refusal}");
    assert!(refusal.contains("every workspace"), "{refusal}");
}

/// The gate every seat consults *before* it pays for the walk (bl-4b48):
/// both halves must be present, and it is the structural statement of
/// "an unbounded or unpriced world costs one read and no `steps/` walk".
#[test]
fn armed_needs_both_a_number_and_a_table() {
    let set = Ceiling::from_json(Some(&json!(1)));
    assert!(set.armed(&table()));
    assert!(
        !set.armed(&Prices::default()),
        "a number it cannot denominate"
    );
    assert!(
        !Ceiling::default().armed(&table()),
        "a table with no number to bind at"
    );
}

/// An empty roster is the general path with no inputs, not a bootstrap
/// case: a world with no workspace has spent nothing, and only a `0`
/// ceiling refuses that.
#[test]
fn zero_is_the_hard_stop_and_an_empty_world_spends_nothing() {
    assert!(
        Ceiling::from_json(Some(&json!(0)))
            .refusal(&[], &table())
            .is_some(),
        "a ceiling of 0 refuses a birth into an unspent world"
    );
    assert!(
        Ceiling::from_json(Some(&json!(0.01)))
            .refusal(&[], &table())
            .is_none()
    );
}
