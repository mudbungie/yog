//! The §8.5 **search** query end-to-end, and its GUI ⇄ headless parity.
//!
//! One world on disk; three seats ask it the same question — the typed line the
//! composer reads, the JSON envelope a deposit carries, and the in-RAM variant
//! the click-glue would construct — and all three must produce the *same
//! gesture* and the *same bytes back*. That is what "one dispatch surface, two
//! serializations, never two implementations" (VISION §8) means for a read.

#![allow(clippy::unwrap_used)]

use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use yog::app::Snapshot;
use yog::boundary::reply::{Reply, encode};
use yog::boundary::{Gesture, Query, answer, codec, consume, deposit, line};
use yog::projects::balls::Ball;
use yog::ui_state::UiState;

fn ball(id: &str, title: &str, body: &str) -> Ball {
    Ball {
        id: id.to_owned(),
        title: title.to_owned(),
        body: body.to_owned(),
        claimant: None,
        blockers: vec![],
        parent: None,
        priority: 3,
        tags: vec![],
        created: None,
        updated: None,
        root_commit: None,
    }
}

/// A world with one project's balls and one workspace whose conversation has a
/// goal and a committed transcript on disk.
fn world(ws: &Path) -> Snapshot {
    let agent = "20260427T120000Z-aaaa";
    let dir = ws.join("agents").join(agent);
    std::fs::create_dir_all(dir.join("messages")).unwrap();
    std::fs::write(dir.join("goal.md"), b"raise the kraken").unwrap();
    std::fs::write(dir.join("messages").join("001-user.md"), b"it stirs").unwrap();
    Snapshot {
        workspaces: vec![],
        projects: vec![],
        trees: HashMap::new(),
        bills: HashMap::new(),
        balls_by_project: HashMap::from([(
            ws.join("proj"),
            vec![ball("bl-1f2a", "wake the kraken", "body")],
        )]),
        closed_by_project: HashMap::from([(
            ws.join("proj"),
            vec![ball("bl-dead", "the kraken, delivered", "closed body")],
        )]),
        join_rows: vec![],
        ops: vec![],
        growth: vec![],
        ui_bytes: None,
        derived_at_unix: 0,
        cadence: yog::app::Cadence::default(),
        fleet: std::collections::BTreeMap::new(),
    }
}

fn ui() -> UiState {
    UiState::open(std::path::PathBuf::from("/nonexistent/ui.json"))
}

/// A `Deps` around `snap` — no binary this suite runs ever needs to spawn.
fn deps(snap: Snapshot, state: &Path) -> yog::boundary::dispatch::Deps {
    yog::boundary::dispatch::Deps {
        litany: yog::cli_outbound::Cli::new("/no/litany"),
        bl: yog::cli_outbound::Cli::new("/no/bl"),
        state_root: state.to_path_buf(),
        yog_binary: state.join("yog"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
        home: state.join("home"),
        yog_data_root: state.join("data"),
        balls_state_root: state.join("balls"),
        snapshot: Arc::new(snap),
        caller: yog::boundary::dispatch::Caller::default(),
    }
}

/// The three spellings are one gesture, and one gesture is one answer.
#[test]
fn the_line_the_envelope_and_the_variant_ask_one_search_and_get_one_answer() {
    let dir = tempdir().unwrap();
    let state = tempdir().unwrap();
    let snap = world(dir.path());
    let d = deps(snap, state.path());
    let variant = Gesture::Ask(Query::Search {
        text: "kraken".to_owned(),
    });

    let typed = line::parse("/search kraken", &line::Context::default()).unwrap();
    assert_eq!(typed, variant, "the line reads the click-glue's variant");
    assert_eq!(
        line::spell(&variant),
        "/search kraken",
        "and writes it back"
    );

    let envelope = json!({ "op": "search", "text": "kraken" });
    assert_eq!(codec::decode(&envelope).unwrap(), variant);
    assert_eq!(codec::encode(&variant), envelope);

    let Gesture::Ask(query) = variant else {
        panic!("a search is a query");
    };
    let reply = answer::answer(&query, &d, &ui(), 100).unwrap();
    let Reply::Search(found) = &reply else {
        panic!("search answers search");
    };
    // Live ball (title), closed ball (title), and the ball ids' project — the
    // whole corpus in one answer.
    let ids: Vec<String> = found.hits.iter().map(yog::search::label).collect();
    assert!(ids.iter().any(|l| l.starts_with("ball bl-1f2a")), "{ids:?}");
    assert!(ids.iter().any(|l| l.starts_with("ball bl-dead")), "{ids:?}");
    assert_eq!(encode(&reply)["kind"], "search");
}

/// The deposit path answers it too — parity is not optional (§8.5) — and the
/// reply file carries the same rows the in-process answer does.
#[test]
fn a_deposited_search_converges_to_the_same_rows() {
    let dir = tempdir().unwrap();
    let state = tempdir().unwrap();
    let snap = world(dir.path());
    let deps = deps(snap, state.path());
    deposit::deposit(
        state.path(),
        "g-find",
        &json!({ "op": "search", "text": "kraken" }),
    )
    .unwrap();
    assert_eq!(consume::consume(&deps, &mut ui(), "T9", 100), 1);

    let reply = deposit::read_reply(state.path(), "g-find").unwrap();
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["kind"], "search");
    let in_process = encode(
        &answer::answer(
            &Query::Search {
                text: "kraken".to_owned(),
            },
            &deps,
            &ui(),
            100,
        )
        .unwrap(),
    );
    assert_eq!(reply, in_process, "one derivation, two serializations");
    assert_eq!(
        reply["unreadable"].as_array().unwrap().len(),
        0,
        "nothing in this world is unreadable"
    );
}

/// A search naming nothing is not a refusal: it is the general path with no
/// input, which is how a seat clears its last answer.
#[test]
fn an_empty_search_answers_empty_at_every_seat() {
    let dir = tempdir().unwrap();
    let state = tempdir().unwrap();
    let snap = world(dir.path());
    let d = deps(snap, state.path());
    let typed = line::parse("/search", &line::Context::default()).unwrap();
    assert_eq!(
        typed,
        Gesture::Ask(Query::Search {
            text: String::new()
        })
    );
    let reply = answer::answer(
        &Query::Search {
            text: String::new(),
        },
        &d,
        &ui(),
        100,
    )
    .unwrap();
    assert_eq!(encode(&reply)["rows"].as_array().unwrap().len(), 0);
}
