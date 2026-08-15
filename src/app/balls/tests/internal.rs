//! Nested-delivery clone filtering (§5.1 #1) and start-flow input plumbing
//! (§8.1), against the shared cloned-project world in [`super`].

use super::{empty_model, model, model_focused, set_list, world};
use crate::binding::workspace_path;
use crate::names::DEFAULT_NAME;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload};
use std::fs;

#[test]
fn internal_clones_never_reach_the_project_surface() {
    // bl-e3e7: the "internal clones" checkbox is gone — the derivation no
    // longer reads any knob out of `ui.json`, and a nested-delivery clone is
    // filtered unconditionally, on the first pass and on every sweep after it.
    let w = world();
    // A nested-delivery clone: its decoded path lies under the state root's
    // bl-delivery tree, so `projects::enumerate` flags it internal (§5.1 #1).
    let internal = w
        .roots
        .balls_clones
        .parent()
        .unwrap()
        .join("plugins/bl-delivery/home/u/p/bl-x");
    let enc = internal.to_string_lossy().replace('/', "%2F");
    fs::create_dir_all(w.roots.balls_clones.join(enc)).unwrap();

    let (_c, mut m) = model(&w);
    assert_eq!(
        m.project_paths(),
        vec![w.project.clone()],
        "the internal clone is filtered out of the project surface"
    );
    m.tick();
    assert_eq!(m.project_paths(), vec![w.project.clone()], "and stays out");
}

#[test]
fn start_inputs_expose_the_ball_payload_and_roots() {
    let w = world();
    set_list(
        &w,
        r#"[{"id":"bl-rdy","title":"Go","body":"do it"},{"id":"bl-me","claimant":"me"}]"#,
    );
    let (_c, m) = model(&w);
    // startable: only the ready, unclaimed ball (bound + claimed-elsewhere excluded).
    let cards = m.startable();
    assert_eq!(cards.len(), 1);
    let Payload::Ball {
        project,
        ball: BallSpec::Existing { id, body, join, .. },
    } = &cards[0].payload
    else {
        panic!("existing ball payload expected");
    };
    assert_eq!(id, "bl-rdy");
    assert_eq!(body, "do it");
    assert_eq!(*join, JoinState::ReadyStartable);
    assert_eq!(project, &crate::naming::leaf(&w.project));
    assert_eq!(cards[0].home.as_path(), w.roots.home.as_path());
    assert_eq!(
        cards[0].balls_state_root.as_path(),
        w.roots.balls_clones.parent().unwrap()
    );
    // A new-ball input bundles a New spec at the same target + roots.
    let nb = m.new_ball_inputs(&w.project, "T", "B");
    assert!(matches!(
        nb.payload,
        Payload::Ball {
            ball: BallSpec::New { .. },
            ..
        }
    ));
    assert_eq!(
        nb.workspace, cards[0].workspace,
        "one resolved target per instance"
    );
    assert_eq!(m.yog_data_root(), w.roots.yog_data.as_path());
    assert_eq!(m.project_paths(), vec![w.project.clone()]);
}

#[test]
fn bare_target_is_the_focused_workspace_named_or_foreign() {
    // Focused on a yog-named workspace → the bare rung prompts into it (§3.4).
    let w = world();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    assert_eq!(m.start_bare_inputs().workspace, w.ws_cobalt);
    // A foreign workspace is a real lernie workspace, so §3.4's "prompt into the
    // focused workspace" applies — its own path, never a redirect into yog's
    // names root (the addendum-3 fix). Really on disk, because the focus is a
    // §3.1 name now and a name resolves against the enumeration (bl-7407) — a
    // directory the walk never saw was never a state this could be in.
    let foreign = w.roots.lernie_data.join("workspaces/foreign");
    std::fs::create_dir_all(foreign.join("repo.git")).unwrap();
    let (_c2, mf) = model_focused(&w, &foreign);
    assert_eq!(mf.start_bare_inputs().workspace, foreign);
}

#[test]
fn bare_target_is_the_default_name_without_a_focus() {
    // The empty world — no workspaces to focus — takes §3.1's default name, the
    // one state that does. Nothing is minted and nothing is asked.
    let (_root, m) = empty_model();
    let inputs = m.start_bare_inputs();
    assert_eq!(
        inputs.workspace,
        workspace_path(m.yog_data_root(), DEFAULT_NAME)
    );
    assert!(inputs.workspace.ends_with("workspaces/home"));
}

#[test]
fn new_workspace_inputs_take_the_typed_name_over_any_focus() {
    // New workspace (§3.4/§11): the deliberate sphere-wall verb raises the
    // operator's own name under the names root, regardless of what is focused.
    let w = world();
    let (_c, mut m) = model(&w);
    m.focus_workspace(&crate::naming::leaf(&w.ws_cobalt));
    let inputs = m.new_workspace_inputs("ops");
    assert_eq!(
        inputs.workspace,
        workspace_path(&w.roots.yog_data, "ops"),
        "the raise names its own workspace, not the focused one"
    );
    // §3.1's validation is the gate that stands in front of it: `ops` is lawful
    // here, an existing leaf and the reserved literal are not.
    assert_eq!(m.validate_workspace_name("  ops  "), Ok("ops".to_owned()));
    assert!(m.validate_workspace_name("Ops!").is_err());
    assert!(m.validate_workspace_name("unknown").is_err());
    assert!(
        m.validate_workspace_name("cobalt").is_err(),
        "an existing leaf under any of the three roots is refused (§3.1)"
    );
}

#[test]
fn start_path_inputs_carries_the_path_payload_at_the_focused_target() {
    // STORIES S2 path rung: the optional work directory, on the focused target.
    let w = world();
    let (_c, mut m) = model(&w);
    m.focus_workspace(&crate::naming::leaf(&w.ws_cobalt));
    let inputs = m.start_path_inputs(std::path::Path::new("/work/here"));
    assert_eq!(inputs.workspace, w.ws_cobalt);
    let Payload::Path { dir } = inputs.payload else {
        panic!("path payload");
    };
    assert_eq!(dir, std::path::PathBuf::from("/work/here"));
}

#[test]
fn empty_project_hint_only_with_zero_projects() {
    // STORIES S3-T5: zero projects → the `yog exec bl prime` hint; any project
    // present (the world here has one clone) → None.
    let w = world();
    let (_c, m) = model(&w);
    assert!(m.empty_project_hint().is_none(), "a project is present");
    let (_root, empty) = empty_model();
    // bl-b491: the command is its own line, verbatim and unadorned — the prose
    // that would have pushed it past the roster's truncation lives on the
    // `lead` line, and nothing is appended to `command`.
    assert_eq!(
        empty.empty_project_hint(),
        Some(crate::app::balls::EmptyHint {
            lead: "No projects yet — add one with:".to_owned(),
            command: "yog exec bl prime".to_owned(),
        })
    );
}

#[test]
fn move_targets_are_the_named_workspaces_minus_the_one_holding_the_ball() {
    // §8.2 Move: one destination rule for the composer's `move to:` buttons,
    // the §11 ball-row menu's submenu and the board row's — and never a move to
    // where the ball already is. **Off the landed enumeration** since bl-b4b5,
    // so the destinations and the tab bar are one answer; a foreign (lernie
    // auto-id) workspace carries no yog identity and is excluded, which is the
    // non-Named arm.
    let w = world();
    fs::create_dir_all(w.roots.lernie_data.join("workspaces/adhoc/repo.git")).unwrap();
    let (_c, m) = model(&w);
    let rows = crate::test_support::chrome::ws_rows(&m);
    assert_eq!(
        crate::nav::tabs::move_targets(&rows, "cobalt"),
        vec!["spare".to_owned()],
        "foreign 'adhoc' excluded, and never where it already is",
    );
    let mut all = crate::nav::tabs::move_targets(&rows, "nobody");
    all.sort();
    assert_eq!(all, vec!["cobalt".to_owned(), "spare".to_owned()]);
    // The §3.6 scope gate off the same rows: yog's own named walls only.
    assert!(crate::nav::tabs::is_named(&rows, "cobalt"));
    assert!(!crate::nav::tabs::is_named(&rows, "adhoc"), "foreign");
    assert!(!crate::nav::tabs::is_named(&rows, "nowhere"), "absent");
}

#[test]
fn focused_ws_name_is_the_focused_leaf_else_none() {
    // The Assign/Move target name (§8.2/§3.2): the focused workspace's leaf.
    let w = world();
    let (_c, mut m) = model(&w);
    m.focus_workspace(&crate::naming::leaf(&w.ws_cobalt));
    assert_eq!(m.focused_ws_name(), Some("cobalt".to_owned()));
    let (_root, empty) = empty_model();
    assert_eq!(empty.focused_ws_name(), None, "no focus ⇒ no target");
}

#[test]
fn resumable_targets_the_bound_balls_own_workspace() {
    // ▶ Continue (§8.1 resume, addendum): a Bound ball re-plans into its *own*
    // claimant workspace, never the focused one, so it can be reached even when
    // another workspace is focused.
    let w = world();
    let (_c, m) = model(&w);
    let cards = m.resumable();
    assert_eq!(cards.len(), 1, "only bl-work (Bound under cobalt)");
    assert_eq!(cards[0].workspace, w.ws_cobalt);
    let Payload::Ball {
        ball: BallSpec::Existing { id, join, .. },
        ..
    } = &cards[0].payload
    else {
        panic!("existing-ball payload");
    };
    assert_eq!(id, "bl-work");
    assert_eq!(*join, JoinState::Bound);
}
