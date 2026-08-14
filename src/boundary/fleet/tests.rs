//! Arming writes one entry and nothing else; disarming removes it and nothing
//! else; neither spawns anything.

use super::*;
use crate::boundary::dispatch::Deps;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::fleet::Verb;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;

fn deps(state_root: &Path) -> Deps {
    Deps {
        lernie: Cli::new("/no/such/lernie"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::xdg::Env::from_env(),
        snapshot: Arc::new({
            let mut snap = snapshot(Path::new("/names/alba"), "alba", Vec::new(), Vec::new());
            // The set an arm's project name resolves over (REMOTE §8).
            snap.projects = vec![PathBuf::from("/dev/yog")];
            snap
        }),
        mint_seed: 7,
    }
}

fn settings(state_root: &Path) -> String {
    std::fs::read_to_string(state_root.join(crate::app::cadence::CADENCE_YAML)).unwrap_or_default()
}

fn arm_verb(ws: &Path, cap: usize) -> Verb {
    Verb::Arm {
        workspace: crate::naming::leaf(ws),
        project: crate::naming::leaf(&(PathBuf::from("/dev/yog"))),
        cap,
    }
}

#[test]
fn arming_writes_the_entry_and_logs_itself() {
    let root = tempdir().expect("tempdir");
    let ws = Path::new("/names/alba");
    let deps = deps(&root.path().join("state"));
    let armed = dispatch(&deps, "1", ws, &arm_verb(ws, 3));
    assert_eq!(armed.expect("armed"), Reply::Armed { armed: true });
    let policy = arming::policy(&settings(&deps.state_root), &crate::nav::ws_key(ws))
        .expect("the entry is the arming");
    assert_eq!(policy.cap, 3);
    assert_eq!(policy.project, PathBuf::from("/dev/yog"));
    assert_eq!(
        policy.lease, None,
        "an arm reaps nothing until the operator writes a lease"
    );
    assert!(
        opslog::tail(&deps.state_root, 10)
            .iter()
            .any(|e| e.argv.contains(&ARM_STEP.to_owned())),
        "the config write is a step on the trail like any other"
    );
}

#[test]
fn disarming_removes_the_entry_and_logs_its_own_step() {
    let root = tempdir().expect("tempdir");
    let ws = Path::new("/names/alba");
    let deps = deps(&root.path().join("state"));
    dispatch(&deps, "1", ws, &arm_verb(ws, 2)).expect("armed");
    let off = dispatch(
        &deps,
        "2",
        ws,
        &Verb::Disarm {
            workspace: crate::naming::leaf(ws),
        },
    );
    assert_eq!(off.expect("disarmed"), Reply::Armed { armed: false });
    assert_eq!(
        arming::policy(&settings(&deps.state_root), &crate::nav::ws_key(ws)),
        None
    );
    assert!(
        opslog::tail(&deps.state_root, 10)
            .iter()
            .any(|e| e.argv.contains(&DISARM_STEP.to_owned()))
    );
}

#[test]
fn arming_leaves_the_clocks_own_entry_byte_for_byte() {
    let root = tempdir().expect("tempdir");
    let state = root.path().join("state");
    std::fs::create_dir_all(&state).expect("mkdir");
    std::fs::write(
        state.join(crate::app::cadence::CADENCE_YAML),
        crate::app::cadence::TEMPLATE,
    )
    .expect("seed");
    let deps = deps(&state);
    dispatch(
        &deps,
        "1",
        Path::new("/names/alba"),
        &arm_verb(Path::new("/names/alba"), 1),
    )
    .expect("armed");
    let text = settings(&state);
    assert_eq!(
        crate::app::cadence::parse(&text),
        crate::app::Cadence::default(),
        "the clock's periods survive an arm untouched"
    );
}

#[test]
fn an_inline_block_refuses_rather_than_rewriting_the_file() {
    let root = tempdir().expect("tempdir");
    let state = root.path().join("state");
    std::fs::create_dir_all(&state).expect("mkdir");
    let before = "fleet: {}\n";
    std::fs::write(state.join(crate::app::cadence::CADENCE_YAML), before).expect("seed");
    let deps = deps(&state);
    let refused = dispatch(
        &deps,
        "1",
        Path::new("/names/alba"),
        &arm_verb(Path::new("/names/alba"), 1),
    );
    assert!(
        refused.is_err(),
        "an inline block is a refusal, not a rewrite"
    );
    assert_eq!(settings(&state), before, "and the file is untouched");
}

/// Reachable from **the** chokepoint, not only from its own door: the gesture
/// runs through `boundary::dispatch` — the one match both frontends enter — so
/// the click, the line and the deposit all reach the same body.
#[test]
fn arming_runs_through_the_one_chokepoint() {
    let root = tempdir().expect("tempdir");
    let ws = Path::new("/names/alba");
    let deps = deps(&root.path().join("state"));
    let mut ui = crate::ui_state::UiState::open(root.path().join("ui.json"));
    let reply = crate::boundary::dispatch::dispatch(
        &deps,
        &mut ui,
        "1",
        &crate::boundary::Action::Fleet(arm_verb(ws, 2)),
    );
    assert_eq!(reply.expect("armed"), Reply::Armed { armed: true });
    assert_eq!(
        arming::policy(&settings(&deps.state_root), &crate::nav::ws_key(ws))
            .expect("armed")
            .cap,
        2
    );
}
