//! **S10-T4 spine-paint** (bl-1802, the seat that replaced the rail gutter):
//! what the step spine puts on screen now that it runs *through* the chat —
//! one clickable rule per operable commit, the pin the click raises, the cards
//! and cohorts born at that commit, and the release on the second click.
//!
//! It is also S10's old rail-paint half and bl-929d's crossings half, merged:
//! there is one drawing of one fact. This file holds the fixture and the
//! **rule** — the commit and its gesture; [`cards`] holds what hangs under one.

use std::collections::HashSet;

use super::rows::{entry, tx};

mod cards;
use crate::rail::{ChildInput, Rail, build};
use crate::transcript::{AutoExpand, Block, Entry, EntryKind, Transcript, Usage, render};

fn delivered(name: &str) -> Entry {
    entry(
        name,
        EntryKind::Delivered {
            sender: "user".into(),
            epitaph: None,
            body: "hi".into(),
        },
    )
}

fn model(name: &str) -> Entry {
    entry(
        name,
        EntryKind::Model {
            model_id: "opus".into(),
            blocks: vec![Block::Text("ok".into())],
            usage: Usage::default(),
        },
    )
}

fn commit(oid: &str, at: i64) -> crate::git_tree::StepCommit {
    crate::git_tree::StepCommit {
        oid: oid.to_owned(),
        short_oid: oid.chars().take(8).collect(),
        timestamp_unix: at,
        subject: "step".to_owned(),
    }
}

fn step(seq: &str, oid: Option<&str>) -> crate::steps_view::StepSummary {
    crate::steps_view::StepSummary {
        seq: seq.to_owned(),
        framing: crate::git_tree::Framing::Complete,
        attempts: 1,
        tokens: crate::budgets::BudgetSpend::default(),
        commit: oid.map(str::to_owned),
        started_at: None,
        ended_at: None,
        auth_failed: crate::login::auth::AuthFailure::No,
        wound: crate::steps_view::Wound::None,
    }
}

fn child(name: &str, commits: Vec<crate::git_tree::StepCommit>) -> ChildInput {
    ChildInput {
        agent_id: format!("root-{name}"),
        name: name.to_owned(),
        state: crate::git_tree::AgentState::InFlight,
        streaming_text: Some(format!("{name} is thinking")),
        commits,
        tokens: 512,
        config_label: None,
    }
}

/// A two-turn chat: `001` delivered, `002` answered, `003` delivered, `004`
/// answered — two operable commits, one rule apiece.
fn chat() -> Transcript {
    tx(vec![
        delivered("001-user.md"),
        model("002-opus.json"),
        delivered("003-user.md"),
        model("004-opus.json"),
    ])
}

fn spine(oids: [Option<&str>; 2], children: Vec<ChildInput>) -> Rail {
    build(
        "storeroom",
        &[commit("0123456789abcdef", 10), commit("bbbb2222", 20)],
        &crate::steps_view::StepsView {
            steps: vec![step("001", oids[0]), step("002", oids[1])],
            orphan: crate::steps_view::Orphan::default(),
        },
        &chat(),
        &children,
    )
}

fn painted(rail: &Rail, selected: &mut Option<usize>) -> String {
    let mut folds = HashSet::new();
    let chat = chat();
    crate::paint_probe::paint(|ui| {
        let _ = render(
            ui,
            &chat,
            &super::reading(false, AutoExpand::default()),
            &mut folds,
            rail,
            selected,
        );
    })
}

/// Each rule wears the short oid of the commit the ensuing call read — never
/// the full id — and sits **above** the crossing it announces.
#[test]
fn every_rule_paints_its_read_state_commit_above_its_crossing() {
    let rail = spine([Some("0123456789abcdef"), Some("bbbb2222")], vec![]);
    let mut folds = HashSet::new();
    let chat = chat();
    let painted = crate::paint_probe::painted_settled(1024.0, 4096.0, |ui| {
        let _ = render(
            ui,
            &chat,
            &super::reading(false, AutoExpand::default()),
            &mut folds,
            &rail,
            &mut None,
        );
    });
    let id = painted
        .iter()
        .find(|(text, _)| text == "0123456")
        .expect("short oid painted");
    assert!(!painted.iter().any(|(text, _)| text.contains("0123456789")));
    let row = painted
        .iter()
        .find(|(text, _)| text.contains("user:"))
        .expect("the crossing's row");
    assert!(id.1.top() < row.1.top(), "rule above its crossing");
    assert!(painted.iter().any(|(text, _)| text == "bbbb222"));
}

/// Absence of a commit = no line, unchanged since bl-929d: the in-flight strip
/// owns that interval, and nothing is guessed in its place.
#[test]
fn a_step_with_no_commit_paints_no_rule() {
    let text = painted(&spine([None, None], vec![]), &mut None);
    assert!(text.contains("user:"), "{text}");
    assert!(!text.contains("0123456"), "{text}");
}

/// One gesture, both directions: clicking a rule pins that commit, clicking
/// the pinned one releases it. There is no second control to find in the chat.
#[test]
fn clicking_a_rule_pins_it_and_clicking_it_again_releases_it() {
    let rail = spine([Some("0123456789abcdef"), Some("bbbb2222")], vec![]);
    let mut selected = None;
    click(&rail, &mut selected, "bbbb222");
    assert_eq!(selected, Some(1));
    click(&rail, &mut selected, "bbbb222");
    assert_eq!(selected, None);
}

/// Two-frame pointer click on the galley whose text is `label`, discarding the
/// follow answer — the shared click idiom (the jsonview pattern).
fn click(rail: &Rail, selected: &mut Option<usize>, label: &str) {
    let _ = follow_click(rail, selected, label);
}

/// The same click, returning what the frame asked to open.
fn follow_click(rail: &Rail, selected: &mut Option<usize>, label: &str) -> Option<String> {
    let ctx = egui::Context::default();
    let chat = chat();
    let mut folds = HashSet::new();
    let run = |input: egui::RawInput, selected: &mut Option<usize>, folds: &mut HashSet<String>| {
        let mut follow = None;
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                follow = render(
                    ui,
                    &chat,
                    &super::reading(false, AutoExpand::default()),
                    folds,
                    rail,
                    selected,
                );
            });
        });
        follow
    };
    let screen = crate::paint_probe::screen;
    let _ = run(screen(), selected, &mut folds);
    let _ = run(screen(), selected, &mut folds);
    let pos = {
        let mut probe = folds.clone();
        let mut sel = *selected;
        let painted = crate::paint_probe::painted_settled(1024.0, 4096.0, |ui| {
            let _ = render(
                ui,
                &chat,
                &super::reading(false, AutoExpand::default()),
                &mut probe,
                rail,
                &mut sel,
            );
        });
        painted
            .iter()
            .find(|(text, _)| text == label)
            .map(|(_, rect)| rect.center())
            .expect("the label is on screen")
    };
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    };
    run(
        egui::RawInput {
            events: vec![egui::Event::PointerMoved(pos), button(true), button(false)],
            ..screen()
        },
        selected,
        &mut folds,
    )
}
