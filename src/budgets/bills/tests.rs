//! The one steps-tree walk: scope selection, the per-step model read, and the
//! forgiving degradations each named in [`super`]'s doc.

use super::{Scope, StepBill, bills, total};
use crate::budgets::BudgetSpend;
use std::path::Path;
use tempfile::tempdir;

const ROOT: &str = "20260717T120000Z-root";
const KID: &str = "20260717T120000Z-root-20260717T120100Z-kid0";

fn usage(input: u64) -> String {
    format!(r#"{{"type":"usage","input_tokens":{input},"output_tokens":1}}"#)
}

/// Write one step: its Usage line, and a `request.json` naming `model` when
/// one is given.
fn write_step(ws: &Path, conv: &str, seq: &str, input: u64, model: Option<&str>) {
    let step = ws.join("steps").join(conv).join(seq);
    std::fs::create_dir_all(&step).unwrap();
    std::fs::write(step.join("response.json"), usage(input)).unwrap();
    if let Some(model) = model {
        std::fs::write(
            step.join("request.json"),
            format!(r#"{{"model":"{model}","messages":[]}}"#),
        )
        .unwrap();
    }
}

fn models(bills: &[StepBill]) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = bills.iter().map(|b| b.model.clone()).collect();
    out.sort();
    out
}

#[test]
fn tree_scope_takes_the_root_and_its_descent_only() {
    let dir = tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 10, Some("opus"));
    write_step(dir.path(), KID, "001", 100, Some("sonnet"));
    // A sibling root, and a conv-id that merely *prefixes* the root without
    // the descent hyphen — neither is in this tree.
    write_step(dir.path(), "20260717T120000Z-othr", "001", 999, None);
    write_step(dir.path(), &format!("{ROOT}x"), "001", 999, None);

    let bills = bills(dir.path(), &Scope::Tree(ROOT.to_owned()));
    assert_eq!(bills.len(), 2);
    assert_eq!(total(&bills).input_tokens, 110);
    assert_eq!(
        models(&bills),
        vec![Some("opus".to_owned()), Some("sonnet".to_owned())]
    );
}

#[test]
fn workspace_scope_takes_every_conversation() {
    let dir = tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 10, Some("opus"));
    write_step(dir.path(), "20260717T120000Z-othr", "001", 5, Some("opus"));

    let bills = bills(dir.path(), &Scope::Workspace);
    assert_eq!(bills.len(), 2);
    assert_eq!(total(&bills).input_tokens, 15);
}

#[test]
fn missing_steps_tree_yields_no_bills() {
    let dir = tempdir().unwrap();
    assert!(bills(dir.path(), &Scope::Workspace).is_empty());
    assert_eq!(total(&[]), BudgetSpend::default());
}

#[test]
fn unreadable_or_shapeless_records_degrade_to_none_never_panic() {
    let dir = tempdir().unwrap();
    let steps = dir.path().join("steps");
    // A step with no request.json at all.
    write_step(dir.path(), ROOT, "001", 10, None);
    // request.json that is not JSON.
    let two = steps.join(ROOT).join("002");
    std::fs::create_dir_all(&two).unwrap();
    std::fs::write(two.join("request.json"), b"{ not json").unwrap();
    // request.json with no `model`, and one whose `model` is not a string.
    let three = steps.join(ROOT).join("003");
    std::fs::create_dir_all(&three).unwrap();
    std::fs::write(three.join("request.json"), br#"{"messages":[]}"#).unwrap();
    let four = steps.join(ROOT).join("004");
    std::fs::create_dir_all(&four).unwrap();
    std::fs::write(four.join("request.json"), br#"{"model":42}"#).unwrap();
    // A non-step entry, and a conv-id entry that is a file, not a dir.
    std::fs::create_dir_all(steps.join(ROOT).join("tools")).unwrap();
    std::fs::write(steps.join(format!("{ROOT}-stray")), b"x").unwrap();

    let bills = bills(dir.path(), &Scope::Tree(ROOT.to_owned()));
    assert_eq!(bills.len(), 4);
    assert!(bills.iter().all(|b| b.model.is_none()));
    // Only the one step that has a response.json contributes tokens.
    assert_eq!(total(&bills).input_tokens, 10);
}

/// VISION V1.5's per-agent scope: exactly `steps/<id>`, and none of its
/// descent — so a fork's shared prefix cost stays with the ancestor and the
/// card's figure never double-counts.
#[test]
fn the_agent_scope_counts_one_id_and_never_its_descent() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_step(ws, ROOT, "001", 10, None);
    write_step(ws, KID, "001", 7, None);
    let own = total(&bills(ws, &Scope::Agent(ROOT.into()))).total_tokens();
    let tree = total(&bills(ws, &Scope::Tree(ROOT.into()))).total_tokens();
    assert_eq!(own, 11, "the agent's own steps only");
    assert_eq!(tree, 19, "the subtree still rolls up");
    assert_eq!(
        total(&bills(
            ws,
            &Scope::Agent(format!("{ROOT}-20260717T120100Z-kid"))
        ))
        .total_tokens(),
        0,
        "a byte prefix is not an id"
    );
}

/// The `meta.json` span rides the one walk (§3.9, bl-40ab), and every way it
/// cannot be read contributes zero rather than a fabricated duration: no meta
/// at all, unparseable bytes, a settled record with no timestamps, a still-open
/// step, and an end before its start.
#[test]
fn the_wall_span_rides_the_walk_and_degrades_to_zero() {
    let dir = tempdir().unwrap();
    let meta = |conv: &str, seq: &str, body: &str| {
        write_step(dir.path(), conv, seq, 1, None);
        let step = dir.path().join("steps").join(conv).join(seq);
        std::fs::write(step.join("meta.json"), body).unwrap();
    };
    // A settled step: 61 seconds, across a minute boundary.
    meta(
        ROOT,
        "001",
        r#"{"started_at":"2026-04-22T06:54:32Z","ended_at":"2026-04-22T06:55:33Z"}"#,
    );
    // Unparseable, no timestamps, an unfinished step, and time running backwards.
    meta(ROOT, "002", "not json");
    meta(ROOT, "003", r#"{"commit":"abc"}"#);
    meta(ROOT, "004", r#"{"started_at":"2026-04-22T06:54:32Z"}"#);
    meta(
        ROOT,
        "005",
        r#"{"started_at":"2026-04-22T06:55:33Z","ended_at":"2026-04-22T06:54:32Z"}"#,
    );
    // And a step with no `meta.json` at all.
    write_step(dir.path(), ROOT, "006", 1, None);

    let bills = bills(dir.path(), &Scope::Tree(ROOT.to_owned()));
    assert_eq!(bills.len(), 6);
    assert_eq!(super::wall(&bills), 61, "only the settled span counts");
}
