//! The derived conversation↔ball join (DESIGN §3.2/§3.3/§3.5, bl-de16): a
//! conversation root's `goal.md` stamp (`Ball <id>:`) parsed back and resolved
//! through the §3.5 claimant join. A hermetic world with one cloned project and a
//! real git fixture as the named workspace, so the conversations carry goal stamps.

use super::{AppModel, FakeBl};
use crate::app::Roots;
use crate::git_tree::tests::fixture::Fixture;
use crate::projects::join::JoinState;
use crate::test_support::FakeClock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

/// One live ball `bl-work`, claimant = the local workspace name `cobalt`, so it
/// binds (§3.2). A conversation stamping `bl-ghost` finds no match here — the
/// resolver's unresolved arm.
const LIST: &str = r#"[{"id":"bl-work","title":"Work","claimant":"cobalt"}]"#;

/// Build a model whose focused named workspace `cobalt` is a real git fixture with
/// three root conversations: `conv1` stamps the bound ball, `conv2` stamps an
/// unknown id, `bare` carries no stamp. Returns the model and the goal roots alive.
fn model() -> (tempfile::TempDir, Fixture, AppModel) {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: crate::test_support::no_world(),
    };
    // The cloned project whose percent-encoded basename decodes to /proj/a.
    std::fs::create_dir_all(roots.balls_clones.join("%2Fproj%2Fa")).unwrap();
    std::fs::create_dir_all(&roots.yog_state).unwrap();
    std::fs::create_dir_all(roots.yog_data.join("workspaces")).unwrap();
    // The named workspace `cobalt` is a real fixture, so it has conversations.
    let fx = Fixture::new();
    fx.build_agent(
        "conv1",
        "You are ochre-tern.\n\nBall bl-work: Work\n\ndo the work",
    );
    fx.build_agent(
        "conv2",
        "You are jade-vole.\n\nBall bl-ghost: Ghost\n\ngone",
    );
    fx.build_agent("bare", "You are slate-newt.\n\njust chatting, no ball");
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
        Some(ws),
        FakeClock::new().arc(),
        Box::new(bl),
        Some("me".to_string()),
    );
    (root, fx, m)
}

/// The `ConvRow` for a given conversation root.
fn ball_of(m: &AppModel, root: &str) -> Option<crate::nav::convs::ConvBall> {
    m.conversations(1000)
        .into_iter()
        .find(|r| r.root_id == root)
        .and_then(|r| r.ball)
}

#[test]
fn a_conversation_row_carries_its_goal_stamp_ball_coloured_by_the_join() {
    let (_root, _fx, m) = model();
    // conv1's goal stamps bl-work, which the §3.5 join binds to cobalt (Bound):
    // the badge resolves with its title and status — the resolver's matched arm.
    let bound = ball_of(&m, "conv1").expect("conv1's stamped ball");
    assert_eq!(bound.id, "bl-work");
    assert_eq!(bound.state, Some(JoinState::Bound));
    assert_eq!(bound.title.as_deref(), Some("Work"));
    assert_eq!(bound.badge, None, "a Bound ball needs no badge");
    // conv2 stamps bl-ghost, which no live/closed ball matches here: the id still
    // renders from source 1, but with no join facts — the unresolved arm.
    let ghost = ball_of(&m, "conv2").expect("conv2's stamped ball renders from source 1");
    assert_eq!(ghost.id, "bl-ghost");
    assert_eq!(ghost.state, None);
    assert_eq!(ghost.title, None);
    // A conversation with no stamp carries no ball.
    assert!(ball_of(&m, "bare").is_none(), "no stamp, no ball");
}

/// The conversation mint's occupied set (§3.3): the stamped names of the target
/// workspace's live roots, read off the tree the §11 list already derives — one
/// parse, no second disk walk, and no cross-workspace enumeration.
#[test]
fn the_conversation_mint_reads_its_occupied_set_off_the_stamped_roots() {
    let (root, _fx, m) = model();
    let ws = m
        .focused_workspace()
        .expect("cobalt is focused")
        .to_path_buf();
    let mut names = m.conversation_names(&ws);
    names.sort();
    assert_eq!(names, ["jade-vole", "ochre-tern", "slate-newt"]);
    // A workspace yog has no tree for — never swept, or the `Target::Mint` one
    // that does not exist yet — contributes nothing: the general path, empty.
    assert!(
        m.conversation_names(&root.path().join("nowhere"))
            .is_empty(),
        "no tree, no occupied names"
    );
}

#[test]
fn conversation_ball_and_groups_expose_the_same_derived_join() {
    let (_root, _fx, m) = model();
    // The header accessor mirrors the row derivation.
    let header = m.conversation_ball("conv1").expect("conv1 header ball");
    assert_eq!(header.id, "bl-work");
    assert_eq!(header.state, Some(JoinState::Bound));
    assert!(
        m.conversation_ball("bare").is_none(),
        "no stamp, no header ball"
    );
    assert!(
        m.conversation_ball("nonexistent").is_none(),
        "an unknown root has no ball"
    );
    // The grouped view heads each ball over its conversations, unassociated last.
    let groups = m.conversation_groups(1000, &std::collections::HashSet::new());
    let work = groups
        .iter()
        .find(|g| g.ball.as_ref().map(|b| b.id.as_str()) == Some("bl-work"))
        .expect("a group headed by bl-work");
    assert!(work.convs.iter().any(|c| c.root_id == "conv1"));
    let bare_group = groups
        .iter()
        .find(|g| g.ball.is_none())
        .expect("the trailing unassociated group");
    assert!(bare_group.convs.iter().any(|c| c.root_id == "bare"));
    assert_eq!(
        groups.last().map(|g| g.ball.is_none()),
        Some(true),
        "unassociated conversations group last"
    );
}
