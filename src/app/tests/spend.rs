//! The §3.5 spend queries on [`AppModel`]: the price table read off `ui.json`,
//! the per-conversation figure, and the two attribution altitudes of a per-ball
//! figure (§3.2's stamped conversation vs. the accepted workspace fallback).
//!
//! A hermetic world with one cloned project and a real git fixture as the named
//! workspace `cobalt`, so the conversations carry goal stamps — the same shape
//! `balls/tests/convball.rs` proved the join on.

use crate::app::balls::tests::FakeBl;
use crate::app::{AppModel, Roots};
use crate::git_tree::tests::fixture::Fixture;
use crate::spend::Attribution;
use crate::test_support::FakeClock;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

/// Two live balls claimed by the local workspace `cobalt`, so both bind (§3.2).
/// Only `bl-stamped` is named by any conversation's goal.
const LIST: &str = r#"[{"id":"bl-stamped","title":"Stamped","claimant":"cobalt"},
                       {"id":"bl-picked","title":"Picked up mid-conversation","claimant":"cobalt"}]"#;

/// $1/Mtok in, $2/Mtok out for `opus`; `haiku` is deliberately absent so the
/// unpriced-tokens path is exercised end to end.
const PRICES: &str = r#"{"v":1,"prices":{"opus":{"input":1,"output":2}}}"#;

fn write_step(ws: &Path, conv: &str, seq: &str, input: u64, output: u64, model: &str) {
    let step = ws.join("steps").join(conv).join(seq);
    std::fs::create_dir_all(&step).unwrap();
    std::fs::write(
        step.join("response.json"),
        format!(r#"{{"type":"usage","input_tokens":{input},"output_tokens":{output}}}"#),
    )
    .unwrap();
    std::fs::write(
        step.join("request.json"),
        format!(r#"{{"model":"{model}"}}"#),
    )
    .unwrap();
}

/// A model over `cobalt`, which holds three roots: `conv1` stamps `bl-stamped`,
/// `conv1-20260717T120100Z-kid0` is its descent child (also stamped, to prove the
/// dedupe), and `other` stamps nothing. `ui_prices` seeds `ui.json`.
fn model(ui_prices: Option<&str>) -> (tempfile::TempDir, Fixture, PathBuf, AppModel) {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        // A world whose lernie config root is this temp dir's own `lernie`
        // leaf — the §9.2 global `models.yaml` below has to be somewhere the
        // worker's `adopt_windows` will actually read it (§5.1 #35).
        world: crate::test_support::world_under(root.path()),
    };
    std::fs::create_dir_all(roots.balls_clones.join("%2Fproj%2Fa")).unwrap();
    std::fs::create_dir_all(&roots.lernie_data).unwrap();
    // `opus` declares a window, `haiku` deliberately does not — the same
    // split the price table above uses, one field over.
    std::fs::write(
        roots.lernie_data.join("models.yaml"),
        "models:\n  opus:\n    provider: anthropic\n    context_window: 2000000\n  \
         haiku:\n    provider: anthropic\n",
    )
    .unwrap();
    std::fs::create_dir_all(&roots.yog_state).unwrap();
    std::fs::create_dir_all(roots.yog_data.join("workspaces")).unwrap();
    if let Some(doc) = ui_prices {
        std::fs::write(roots.ui_json(), doc).unwrap();
    }
    let fx = Fixture::new();
    fx.build_agent(
        "conv1",
        "You are ochre-tern.\n\nBall bl-stamped: Stamped\n\ngo",
    );
    fx.build_agent(
        "conv1-20260717T120100Z-kid0",
        "You are jade-vole.\n\nBall bl-stamped: Stamped\n\nsub",
    );
    fx.build_agent("other", "You are slate-newt.\n\nno ball here");
    // conv1's own step, its child's step (whole-tree, ARCH §6), and a step in
    // the unrelated conversation that only a workspace-wide sum can reach.
    write_step(&fx.path, "conv1", "001", 1_000_000, 500_000, "opus");
    write_step(
        &fx.path,
        "conv1-20260717T120100Z-kid0",
        "001",
        0,
        0,
        "haiku",
    );
    write_step(&fx.path, "other", "001", 2_000_000, 0, "opus");
    let ws = roots.yog_data.join("workspaces").join("cobalt");
    std::os::unix::fs::symlink(&fx.path, &ws).unwrap();
    let bl = FakeBl {
        live: Arc::new(Mutex::new(HashMap::from([(
            PathBuf::from("/proj/a"),
            LIST.to_string(),
        )]))),
        closed: Arc::new(Mutex::new(HashMap::new())),
        fail: Arc::new(Mutex::new(HashSet::new())),
    };
    let (m, _deriver) = AppModel::boot(
        roots,
        Some(ws.clone()),
        FakeClock::new().arc(),
        Box::new(bl),
        Some("me".to_string()),
    );
    (root, fx, ws, m)
}

#[test]
fn no_price_table_leaves_every_figure_tokens_only() {
    let (_root, _fx, ws, m) = model(None);
    assert!(m.prices().is_empty());
    let figure = m.conversation_spend(&ws, "conv1");
    assert_eq!(figure.tokens.total_tokens(), 1_500_000);
    assert!(
        figure.cost.is_none(),
        "deleting the table deletes the column"
    );
}

#[test]
fn a_conversation_figure_prices_its_whole_descent() {
    let (_root, _fx, ws, m) = model(Some(PRICES));
    let figure = m.conversation_spend(&ws, "conv1");
    assert_eq!(figure.attribution, Attribution::Conversations(1));
    // 1 Mtok in at $1 + 0.5 Mtok out at $2 = $2.00; the child's haiku step is
    // in the tree but unpriced, so it adds nothing but zero tokens.
    assert_eq!(figure.cost.unwrap().usd(), "$2.00");
}

#[test]
fn a_stamped_ball_attributes_to_one_conversation_child_stamps_deduped() {
    let (_root, _fx, ws, m) = model(Some(PRICES));
    let figure = m.ball_spend(&ws, "bl-stamped");
    // Both the root and its descent child stamp the ball; the child resolves to
    // its root, so the tree is billed once — not twice.
    assert_eq!(figure.attribution, Attribution::Conversations(1));
    assert_eq!(figure.tokens.total_tokens(), 1_500_000);
    assert_eq!(figure.cost.unwrap().micro_usd, 2_000_000);
    assert!(figure.attribution.note().is_none());
}

#[test]
fn a_ball_no_conversation_stamps_falls_back_to_the_workspace() {
    let (_root, _fx, ws, m) = model(Some(PRICES));
    let figure = m.ball_spend(&ws, "bl-picked");
    // The §3.5 ruling: a ball claimed mid-conversation records no conversation
    // link, so the honest figure is the whole workspace's — labelled as such,
    // never a fabricated linkage.
    assert_eq!(figure.attribution, Attribution::Workspace);
    assert_eq!(figure.tokens.total_tokens(), 3_500_000);
    assert_eq!(figure.attribution.note().unwrap().label, "workspace-wide");
}

#[test]
fn an_unknown_workspace_attributes_nothing_rather_than_guessing() {
    let (_root, _fx, _ws, m) = model(Some(PRICES));
    let figure = m.ball_spend(Path::new("/nowhere"), "bl-stamped");
    assert_eq!(figure.attribution, Attribution::Workspace);
    assert_eq!(figure.tokens.total_tokens(), 0);
}

/// The §5.1 #35 figure over the very same snapshot the spend figures read, and
/// what makes it a different question: the conversation's whole-tree spend is
/// 1.5 Mtok, while its context holds the 1 Mtok prompt its root's latest step
/// actually sent.
#[test]
fn a_conversation_context_is_its_root_s_prompt_not_its_descent_s_spend() {
    let (_root, _fx, ws, m) = model(None);
    assert_eq!(
        m.conversation_spend(&ws, "conv1").tokens.total_tokens(),
        1_500_000
    );
    let full = m
        .conversation_context(&ws, "conv1")
        .expect("a measured context");
    assert_eq!(full.model, "opus");
    assert_eq!(full.prompt_tokens, 1_000_000);
    assert_eq!(full.window, 2_000_000);
    assert_eq!(full.percent(), 50);
    // The child runs its own context on a model nothing declares a window for,
    // so its conversation renders no figure rather than a guessed one.
    assert_eq!(
        m.conversation_context(&ws, "conv1-20260717T120100Z-kid0"),
        None
    );
}
