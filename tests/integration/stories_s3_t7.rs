//! STORIES **S3-T7** close-stamps-and-delivers: Close on a bound ball spawns
//! `bl close <id> --as <the ball's bound workspace name>` with cwd = the project
//! (§8.2's claimant rider — the operator's `$USER` never appears), and with the
//! ball then absent from the live set the §3.5 join re-derives it **delivered**
//! under that same workspace, turning the conversation's badge ash
//! (STORIES S3.5, DESIGN §3.2, §3.4, §3.5, §8.2).
//!
//! One model is driven across the delivery rather than two being compared: the
//! fake `bl` runner's live/closed sets flip together (`support::FakeBl`), which
//! is exactly the disk change `bl close` makes, so `after_bl_verb` → re-fetch →
//! re-join is the sequence under test and not a second fixture.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, Recorder, build_agents, canon, clone_dir};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::tempdir;
use yog::actions::verbs;
use yog::cli_outbound::Cli;
use yog::projects::join::{self, JoinState};
use yog::theme;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// The ball, claimed by the local workspace "cobalt" — a §3.5 **Bound** row.
const LIVE: &str = r#"[{"id":"bl-7","title":"Wire it","claimant":"cobalt"}]"#;
/// The same ball once delivered: absent from live, present in the dead set with
/// its claimant intact — the only fact the Delivered row is re-derived from.
const CLOSED: &str = r#"[{"id":"bl-7","title":"Wire it","claimant":"cobalt"}]"#;

/// STORIES **S3-T7** close-stamps-and-delivers.
#[test]
fn s3_t7_close_stamps_the_bound_workspace_and_the_row_re_derives_delivered() {
    let root = tempdir().unwrap();
    let (bin, project) = (tempdir().unwrap(), tempdir().unwrap());
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    clone_dir(&roots.balls_clones, project.path());
    // The workspace "cobalt" with one conversation whose goal stamps bl-7 (§3.3).
    let ws = roots.yog_data.join("workspaces").join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    build_agents(&ws, &[AgentFixture::stamped("c-001", "bl-7", "Wire it")]);

    let delivered = Arc::new(AtomicBool::new(false));
    let (mut m, mut deriver) = AppModel::boot(
        roots,
        None,
        Arc::new(SystemClock),
        Box::new(FakeBl {
            live: HashMap::from([(project.path().to_path_buf(), LIVE.to_owned())]),
            closed: HashMap::from([(project.path().to_path_buf(), CLOSED.to_owned())]),
            delivered: Arc::clone(&delivered),
        }),
        None,
    );
    m.focus_workspace(&ws);

    // --- Before: the row is Bound and the conversation's badge is the bound hue.
    let row = m.focused_join().expect("cobalt binds bl-7").clone();
    assert_eq!(row.state, JoinState::Bound);
    let before = m.conversation_ball("c-001").expect("the goal stamps bl-7");
    assert_eq!(before.id, "bl-7");
    assert_eq!(before.state, Some(JoinState::Bound));
    assert_eq!(
        theme::ball_hue(JoinState::Bound),
        theme::HYDRA,
        "a bound ball's badge is green"
    );

    // --- Close: one verb, stamped with the ball's BOUND WORKSPACE name (§8.2).
    // The name is derived, never typed: `owner_name` reads the claimant off the
    // row, which for a Bound row IS the workspace's name (§3.2's equality).
    let owner = join::owner_name(&row);
    assert_eq!(owner, "cobalt", "the claimant delivers its own ball");
    let bl = Recorder::new(bin.path(), "bl").on("close", "", 0);
    let out = verbs::close(
        &Cli::new(bl.path()),
        &root.path().join("state"),
        "T0",
        project.path(),
        &row.ball_id,
        &owner,
    )
    .unwrap();
    assert_eq!(out.exit, 0);
    let inv = bl.invocations();
    assert_eq!(inv.len(), 1);
    assert_eq!(inv[0].argv, ["close", "bl-7", "--as", "cobalt"]);
    assert_eq!(
        inv[0].cwd,
        canon(project.path()),
        "cwd = the project (§8.2)"
    );

    // --- After: the ball leaves the live set, so the join re-derives it from the
    // closed listing's claimant alone — nothing was stored to remember it (§3.4).
    delivered.store(true, Ordering::SeqCst);
    m.after_bl_verb(project.path());
    for _ in 0..200 {
        deriver.step();
        m.refresh();
        if m.focused_join().map(|r| r.state) == Some(JoinState::Delivered) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let after_row = m
        .focused_join()
        .expect("the delivered row groups under cobalt");
    assert_eq!(after_row.state, JoinState::Delivered);
    assert_eq!(after_row.ball_id, "bl-7");
    assert_eq!(
        after_row.workspace.as_deref(),
        Some(ws.as_path()),
        "grouped under the same workspace"
    );
    assert_eq!(
        join::badge(JoinState::Delivered, None).as_deref(),
        Some("delivered")
    );

    // The conversation's badge turns ash — same stamp, new join (§3.5).
    let after = m
        .conversation_ball("c-001")
        .expect("the stamp is unchanged");
    assert_eq!(
        after.id, "bl-7",
        "the stamp is the fact; the colour is the join"
    );
    assert_eq!(after.state, Some(JoinState::Delivered));
    assert_eq!(
        theme::ball_hue(JoinState::Delivered),
        theme::ASH,
        "a delivered ball's badge is ash"
    );
}
