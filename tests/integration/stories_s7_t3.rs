//! STORIES **S7-T3** budget-fold: usage across a root and two children folds to
//! the subtree total the header shows; the limit line renders as raw text,
//! never parsed (STORIES S7.4, DESIGN §11).
//!
//! "Budget is a fold over the subtree's usage events" — a *query* over the
//! `response.json` lines on disk, never a counter yog keeps, so a retried step
//! is billed exactly because its segment is there to read.
//!
//! **The limit half needed locating.** yog owns no YAML parser and will not
//! become a second authority on one, so `workflow.yaml` — budget lines and all
//! — is rendered as raw text. The mechanism is `config_edit::form::schema_for`
//! returning `None` for that file name, `None` being the documented raw-text
//! fallback; there is no budget-limit view-model anywhere in yog, which is the
//! promise kept rather than a gap.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, build_agents, write_step};
use tempfile::tempdir;
use yog::budgets::{self, Scope};
use yog::config_edit::form::schema_for;
use yog::spend::{self, Attribution, Prices};

/// One usage segment worth `input`/`output` tokens, settled.
fn usage(input: u64, output: u64) -> String {
    format!(
        "{{\"type\":\"usage\",\"input_tokens\":{input},\"output_tokens\":{output}}}\n{{\"type\":\"finish\"}}\n{{\"type\":\"end\"}}\n"
    )
}

/// STORIES **S7-T3** budget-fold.
#[test]
fn s7_t3_the_subtree_folds_and_the_limit_is_never_parsed() {
    let root = tempdir().unwrap();
    let ws = root.path().join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    // A root and two children (§2.3: a child's id is its parent's plus two
    // tokens), plus an unrelated conversation that must NOT be folded in.
    let (r, c1, c2, other) = ("c-001", "c-001-a-002", "c-001-b-003", "z-009");
    build_agents(
        &ws,
        &[
            AgentFixture::new(r, "root\n"),
            AgentFixture::new(c1, "child one\n"),
            AgentFixture::new(c2, "child two\n"),
            AgentFixture::new(other, "stranger\n"),
        ],
    );
    write_step(&ws, r, "001", "response.json", &usage(100, 10));
    write_step(&ws, r, "001", "request.json", r#"{"model":"opus"}"#);
    // The root retried: a second segment, billed too.
    write_step(&ws, r, "002", "response.json", &usage(50, 5));
    write_step(&ws, c1, "001", "response.json", &usage(200, 20));
    write_step(&ws, c2, "001", "response.json", &usage(300, 30));
    write_step(&ws, other, "001", "response.json", &usage(999, 999));

    // --- The fold over the subtree. `Scope::Tree` is the hyphenated descent,
    // so the stranger is excluded by the id grammar, not by a stored parent.
    let bills = budgets::bills(&ws, &Scope::Tree(r.to_owned()));
    let total = budgets::total(&bills);
    assert_eq!(total.input_tokens, 100 + 50 + 200 + 300);
    assert_eq!(total.output_tokens, 10 + 5 + 20 + 30);
    assert_eq!(total.total_tokens(), 715, "the header's one number");
    assert!(
        !bills.iter().any(|b| b.conv == other),
        "an unrelated conversation is not in this subtree"
    );

    // A single agent's scope is the root's own steps alone — the difference
    // between the two scopes is exactly the children's usage.
    let alone = budgets::total(&budgets::bills(&ws, &Scope::Agent(r.to_owned())));
    assert_eq!(alone.total_tokens(), 165, "the root's two segments");
    // The whole workspace includes the stranger.
    let all = budgets::total(&budgets::bills(&ws, &Scope::Workspace));
    assert_eq!(all.total_tokens(), 715 + 1998);

    // --- The same fold through the header's own seat.
    let figure = spend::of_conversation(&bills, r, &Prices::default());
    assert_eq!(figure.tokens.total_tokens(), 715);
    assert_eq!(figure.attribution, Attribution::Conversations(1));
    // With no price table there is no cost — the severability gate. yog states
    // tokens, which it can read, and refuses to invent money it cannot.
    assert!(
        figure.cost.is_none(),
        "an empty price table yields no dollar figure, not a zero one"
    );

    // The model rides off `request.json`; a step that never named one is
    // unattributed rather than guessed at.
    let priced = bills.iter().find(|b| b.model.is_some()).unwrap();
    assert_eq!(priced.model.as_deref(), Some("opus"));
    assert!(
        bills.iter().any(|b| b.model.is_none()),
        "a step with no request.json names no model"
    );

    // --- The limit line: raw text, never parsed. `workflow.yaml` has no schema,
    // and no-schema IS the raw-text fallback — so the budget limits inside it
    // reach the operator as the file's own words.
    assert!(
        schema_for("workflow.yaml").is_none(),
        "yog owns no workflow.yaml parser and must not grow one"
    );
    // The files that DO have a schema are the ones yog genuinely owns a form
    // for, which is what makes the absence above a rule rather than an oversight.
    assert!(schema_for("models.yaml").is_some());
    assert!(schema_for("providers.yaml").is_some());
    assert!(schema_for("anything-else.yaml").is_none());
}
