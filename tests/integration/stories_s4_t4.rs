//! STORIES **S4-T4** conversation-badges: four conversations in one workspace —
//! stamped with a bound ball, stamped with a delivered ball, stamped with an id
//! this machine's join does not know, and unstamped — derive badge hues green /
//! ash / **uncoloured id** / **none**; plus a ball the workspace claims with no
//! stamp anywhere, present in the workspace's bound rows and absent from every
//! conversation row (STORIES S4.5, DESIGN §3.2's two altitudes, §3.5).
//!
//! The stamp is truth and the colour is the join's when it has one: an unknown
//! id still renders (`ConvBall::id`), it just carries no `state` to colour by —
//! and a ball no conversation stamped is never given a fabricated row.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents, clone_dir};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use yog::projects::join::JoinState;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// Live: the bound ball a conversation stamps, plus the ball "cobalt" claimed
/// with **no stamp anywhere** — §3.2's honest limit (an agent picked it up
/// mid-conversation, and no fact records which conversation did).
const LIVE: &str = r#"[
    {"id":"bl-1","title":"Bound","claimant":"cobalt"},
    {"id":"bl-9","title":"Picked up mid-conversation","claimant":"cobalt"}
]"#;
/// Dead: the delivered ball, still naming its claimant — the only fact the ash
/// badge is re-derived from (§3.4).
const CLOSED: &str = r#"[{"id":"bl-2","title":"Delivered","claimant":"cobalt"}]"#;

/// STORIES **S4-T4** conversation-badges.
#[test]
fn s4_t4_conversation_badges_are_honest_about_what_the_join_knows() {
    let root = tempdir().unwrap();
    let project = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        litany_data: root.path().join("litany"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    clone_dir(&roots.balls_clones, project.path());
    let ws = roots.yog_data.join("workspaces").join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    build_agents(
        &ws,
        &[
            AgentFixture::stamped("c-001", "bl-1", "Bound"),
            AgentFixture::stamped("c-002", "bl-2", "Delivered"),
            AgentFixture::stamped("c-003", "bl-404", "Never heard of it"),
            AgentFixture::new("c-004", "just a prompt, no ball\n"),
        ],
    );

    let (mut m, mut deriver) = AppModel::boot(
        roots,
        Arc::new(SystemClock),
        Box::new(FakeBl {
            live: HashMap::from([(project.path().to_path_buf(), LIVE.to_owned())]),
            closed: HashMap::from([(project.path().to_path_buf(), CLOSED.to_owned())]),
            ..FakeBl::default()
        }),
        None,
    );
    let name = yog::naming::leaf(&ws);
    // The closed listing is **on demand** (§5.1 #4) — never on the fetch cadence,
    // so at boot the dead set is empty and the delivered badge has nothing to
    // colour by. A landed `bl` verb is its one trigger, which is also the only
    // way an operator ever sees a delivered badge: it appears *because* the ball
    // was just closed.
    m.after_bl_verb(project.path());
    for _ in 0..200 {
        deriver.step();
        if m.take() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let rows = crate::support::conversation_rows(&m, &name, 1000);
    assert_eq!(rows.len(), 4, "four conversations: {rows:?}");
    let by_root: HashMap<&str, &yog::nav::convs::ConvRow> =
        rows.iter().map(|r| (r.root_id.as_str(), r)).collect();
    let ball_of = |root: &str| by_root.get(root).and_then(|r| r.ball.clone());

    // 1. Bound ball ⇒ the id renders and the join colours it green.
    let bound = ball_of("c-001").expect("a stamped conversation carries its ball");
    assert_eq!(bound.id, "bl-1");
    assert_eq!(bound.state, Some(JoinState::Bound));

    // 2. Delivered ball ⇒ still stamped, now ash.
    let delivered = ball_of("c-002").expect("a delivered ball is still a stamp");
    assert_eq!(delivered.id, "bl-2");
    assert_eq!(delivered.state, Some(JoinState::Delivered));

    // 3. A stamped id the join does not know ⇒ the id renders UNCOLOURED. The
    //    stamp is source 1 and always true; the colour is the join's, and the
    //    join has nothing to say, so it says nothing rather than guessing.
    let unknown = ball_of("c-003").expect("an unknown id still renders — the stamp is truth");
    assert_eq!(unknown.id, "bl-404");
    assert_eq!(unknown.state, None, "no state ⇒ nothing to colour by");
    assert_eq!(unknown.title, None);
    assert_eq!(unknown.badge, None);

    // 4. An unstamped conversation carries NO badge at all (§3.2).
    assert_eq!(ball_of("c-004"), None, "bare conversations show no badge");

    // The claimed-but-unstamped ball: in the workspace's bound rows …
    let ws_ids: Vec<String> = crate::support::ws_balls(&m, &ws)
        .into_iter()
        .map(|b| b.id)
        .collect();
    assert!(
        ws_ids.contains(&"bl-9".to_owned()),
        "a ball the workspace claims is a workspace row: {ws_ids:?}"
    );
    // … and in NO conversation row, because no fact says which one picked it up.
    assert!(
        rows.iter()
            .all(|r| r.ball.as_ref().is_none_or(|b| b.id != "bl-9")),
        "no fact attributes bl-9 to a conversation, so no row invents one"
    );
}
