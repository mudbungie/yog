//! Arming writes one entry and seeds one file; disarming removes the entry and
//! nothing else; flagging writes one row and touches nothing.

use super::*;
use crate::boundary::dispatch::Deps;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::monitor::{Verb, arming};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

fn deps(state_root: &std::path::Path) -> Deps {
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::xdg::Env::from_env(),
        snapshot: Arc::new(snapshot(
            std::path::Path::new("/names/alba"),
            "alba",
            Vec::new(),
            Vec::new(),
        )),
        caller: crate::boundary::dispatch::Caller::default(),
    }
}

/// An id-shaped agent needle (ARCH §2.3's compact stamp), which the §8.5
/// conversation resolution passes through with no enumeration (bl-49bc).
const AGENT: &str = "20260101T000000Z-c1";

fn settings(state_root: &std::path::Path) -> String {
    std::fs::read_to_string(state_root.join(crate::app::cadence::CADENCE_YAML)).unwrap_or_default()
}

#[test]
fn arming_writes_the_entry_seeds_the_policy_and_logs_itself() {
    let root = tempdir().expect("tempdir");
    let ws = std::path::Path::new("/names/alba");
    let deps = deps(&root.path().join("state"));
    // Through the family's one door, exactly as the chokepoint calls it.
    let armed = dispatch(
        &deps,
        "1",
        ws,
        "",
        &Verb::Arm {
            workspace: crate::naming::leaf(ws),
            model: "haiku".to_owned(),
        },
    );
    assert_eq!(armed.expect("armed"), Reply::Armed { armed: true });
    let text = settings(&deps.state_root);
    let watch = arming::watch(&text, &crate::nav::ws_key(ws)).expect("armed");
    assert_eq!(watch.model, "haiku");
    let policy = std::fs::read_to_string(deps.state_root.join(&watch.prompt)).expect("seeded");
    assert_eq!(policy, arming::TEMPLATE);
    assert!(
        opslog::tail(&deps.state_root, 10)
            .iter()
            .any(|e| e.argv.contains(&ARM_STEP.to_owned())),
        "the config write is on the trail like every other step"
    );
}

#[test]
fn re_arming_never_overwrites_the_operators_tuned_policy() {
    let root = tempdir().expect("tempdir");
    let ws = std::path::Path::new("/names/alba");
    let deps = deps(&root.path().join("state"));
    arm(&deps, "1", ws, Some("haiku")).expect("armed");
    std::fs::write(deps.state_root.join(arming::PROMPT_FILE), "my rules").expect("tuned");
    arm(&deps, "2", ws, Some("cheaper")).expect("re-armed");
    assert_eq!(
        std::fs::read_to_string(deps.state_root.join(arming::PROMPT_FILE)).expect("kept"),
        "my rules"
    );
}

#[test]
fn disarming_removes_the_entry_and_leaves_the_clock_alone() {
    let root = tempdir().expect("tempdir");
    let ws = std::path::Path::new("/names/alba");
    let deps = deps(&root.path().join("state"));
    std::fs::create_dir_all(&deps.state_root).expect("state root");
    std::fs::write(
        deps.state_root.join(crate::app::cadence::CADENCE_YAML),
        crate::app::cadence::TEMPLATE,
    )
    .expect("clock");
    arm(&deps, "1", ws, Some("haiku")).expect("armed");
    let off = dispatch(
        &deps,
        "2",
        ws,
        "",
        &Verb::Disarm {
            workspace: crate::naming::leaf(ws),
        },
    );
    assert_eq!(off.expect("disarmed"), Reply::Armed { armed: false });
    let text = settings(&deps.state_root);
    assert_eq!(arming::watch(&text, &crate::nav::ws_key(ws)), None);
    assert_eq!(
        crate::app::cadence::parse(&text),
        crate::app::Cadence::default(),
        "the clock's own settings survive"
    );
}

#[test]
fn an_unrewritable_settings_file_refuses_rather_than_half_arming() {
    let root = tempdir().expect("tempdir");
    let deps = deps(&root.path().join("state"));
    std::fs::create_dir_all(&deps.state_root).expect("state root");
    std::fs::write(
        deps.state_root.join(crate::app::cadence::CADENCE_YAML),
        "monitor: {}\n",
    )
    .expect("inline");
    let why = arm(
        &deps,
        "1",
        std::path::Path::new("/names/alba"),
        Some("haiku"),
    )
    .expect_err("refused");
    assert!(why.contains("inline"), "{why}");
}

#[test]
fn flagging_writes_one_row_and_does_nothing_else() {
    let root = tempdir().expect("tempdir");
    let deps = deps(&root.path().join("state"));
    let ws = std::path::Path::new("/names/alba");
    let raised = dispatch(
        &deps,
        "1",
        ws,
        AGENT,
        &Verb::Flag {
            workspace: crate::naming::leaf(ws),
            agent: AGENT.to_owned(),
            reason: "this looks wrong".to_owned(),
        },
    );
    assert_eq!(raised.expect("flagged"), Reply::Flagged);
    // And through the boundary's own chokepoint, which is the door every seat
    // actually uses — the family must be reachable from there, not only from
    // its own module.
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let through = crate::boundary::dispatch::dispatch(
        &deps,
        &mut ui,
        "2",
        &crate::boundary::Action::Monitor(Verb::Flag {
            workspace: crate::naming::leaf(ws),
            // Id-shaped, because the chokepoint resolves the conversation now
            // (bl-49bc): a needle that reads as an id passes through untouched.
            agent: AGENT.to_owned(),
            reason: "again".to_owned(),
        }),
    );
    assert_eq!(through.expect("flagged"), Reply::Flagged);
    let tail = opslog::tail(&deps.state_root, 10);
    assert_eq!(tail.len(), 2, "two rows, nothing else");
    assert_eq!(
        tail[0].argv.first().map(String::as_str),
        Some(crate::monitor::flag::YOG_FLAG)
    );
    assert_eq!(tail[0].stdout, "this looks wrong");
    assert!(
        !settings(&deps.state_root).contains("monitor"),
        "flagging arms nothing"
    );
}
