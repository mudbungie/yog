//! The §16.3 knob and the §9.4 pick, driven through the chokepoint (bl-3f46):
//! a recorder `bl` for the knob's `conf` writes and reads, a real-git workspace
//! and a recorder `lernie` for the pick's two halves.

use super::{ACME, deps_at, fire, quiet, script, seed_wall};
use crate::boundary::Action;
use crate::boundary::reply::Reply;
use crate::git_tree::tests::fixture::Fixture;
use crate::test_support::{TEMPLATE_PROVIDERS, spawn_guard};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn set_marks_answers_with_the_branch_it_read_back_and_logs_the_write() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let ws = root.path().join("workspaces").join("home");
    let reply = fire(
        &deps,
        &Action::SetMarks {
            workspace: ws.clone(),
            branch: "balls/agents/home".to_owned(),
        },
    );
    let space = crate::world::marks::read(&deps.world, &ws);
    assert_eq!(
        reply,
        Ok(Reply::Marks {
            branch: "balls/agents/home".to_owned(),
            space: space.state.clone(),
        })
    );
    // The write is on the trail (§4.2) as the non-spawn step it is — no `bl`
    // was run, because the value's home is the space's own balls config.
    let logged = crate::opslog::tail(&deps.state_root, 10);
    assert_eq!(logged.len(), 1);
    assert_eq!(logged[0].argv, ["yog-step", "marks", "balls/agents/home"]);
}

#[test]
fn reading_marks_never_refuses_even_for_a_workspace_with_no_project() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let ws = root.path().join("nowhere");
    let reply = super::ask(
        &deps,
        &crate::boundary::Query::Marks {
            workspace: ws.clone(),
        },
    );
    // The launched-then-told-to-work-on-a-project case: nothing is primed,
    // nothing is bound, and the agent still has a branch — balls' default.
    assert_eq!(
        reply,
        Ok(Reply::Marks {
            branch: crate::world::marks::SHARED_BRANCH.to_owned(),
            space: crate::world::marks::read(&deps.world, &ws).state,
        })
    );
}

#[test]
fn set_marks_refuses_an_unlawful_branch_rather_than_writing_one() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let err = fire(
        &deps,
        &Action::SetMarks {
            workspace: root.path().join("workspaces").join("home"),
            branch: "balls/config".to_owned(),
        },
    )
    .unwrap_err();
    assert!(err.contains("landing branch"), "{err}");
}

/// A real-git workspace carrying lernie's own `providers.yaml` on
/// `config/default` — what the pick reads before it rewrites it.
fn workspace() -> Fixture {
    let fx = Fixture::new();
    fx.commit_other("providers.yaml", TEMPLATE_PROVIDERS);
    fx
}

fn pick(role: &str, provider: &str, model: &str, ws: &Path) -> Action {
    Action::PickModel {
        workspace: ws.to_path_buf(),
        role: role.to_owned(),
        provider: provider.to_owned(),
        model: model.to_owned(),
    }
}

#[test]
fn a_pick_declares_the_model_then_commits_the_assignment() {
    let g = spawn_guard();
    let root = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let log = bin.path().join("log");
    let lernie = script(
        bin.path(),
        "lernie",
        &format!(
            "cat \"$YOG_EDIT_SRC/providers.yaml\" > {}\nexit 0\n",
            log.display()
        ),
    );
    let fx = workspace();
    let deps = deps_at(root.path(), &lernie, Path::new("/no/bl"));
    // The pick's provider gate reads the rows of the workspace being picked
    // for (bl-fcd5), so `acme` has to be live in *that* sphere's wall.
    seed_wall(&deps, &fx.path, ACME);
    let reply = fire(&deps, &pick("worker", "acme", "m-9", &fx.path));
    drop(g);
    assert!(
        matches!(&reply, Ok(Reply::Outcome(o)) if o.ok()),
        "{reply:?}"
    );
    // models.yaml first — a role naming an undeclared model bricks a workspace.
    let models = fs::read_to_string(root.path().join("lernie/models.yaml")).unwrap();
    assert!(models.contains("m-9"), "{models}");
    assert!(models.contains("acme"), "{models}");
    // …then the assignment, staged for lernie's own commit.
    let staged = fs::read_to_string(&log).unwrap();
    assert!(staged.contains("provider: acme"), "{staged}");
    assert!(staged.contains("model: m-9"), "{staged}");
}

#[test]
fn a_pick_on_a_row_brazen_lacks_writes_neither_half() {
    let root = tempdir().unwrap();
    let fx = workspace();
    let deps = quiet(root.path());
    let err = fire(&deps, &pick("worker", "nope", "m-9", &fx.path)).unwrap_err();
    assert!(err.contains("no provider row `nope`"), "{err}");
    assert!(!root.path().join("lernie/models.yaml").exists());
}

/// Write the model cache `bz --list-models` wholesale-writes for `provider`
/// inside `workspace`'s own wall — the roster the picker read to offer the id,
/// which is where the pick's declared window is seeded from (bl-848f).
fn seed_roster(
    deps: &crate::boundary::dispatch::Deps,
    workspace: &Path,
    provider: &str,
    doc: &str,
) {
    let dir = crate::config_edit::brazen::BrazenPaths::in_wall(&crate::world::wall::root_of(
        &deps.world,
        workspace,
    ))
    .models_cache_dir;
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{provider}.json")), doc).unwrap();
}

/// bl-848f. The declared window is the number brazen served for that id on that
/// row — not the 200k default sitting next to a fact the roster already had.
#[test]
fn a_pick_declares_the_window_the_provider_served() {
    let g = spawn_guard();
    let root = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let lernie = script(bin.path(), "lernie", "exit 0\n");
    let fx = workspace();
    let deps = deps_at(root.path(), &lernie, Path::new("/no/bl"));
    seed_wall(&deps, &fx.path, ACME);
    seed_roster(
        &deps,
        &fx.path,
        "acme",
        r#"{"models":[{"default":false,"id":"m-9","context_window":1048576}],"last_used":"m-9"}"#,
    );
    let reply = fire(&deps, &pick("worker", "acme", "m-9", &fx.path));
    drop(g);
    assert!(
        matches!(&reply, Ok(Reply::Outcome(o)) if o.ok()),
        "{reply:?}"
    );
    let models = fs::read_to_string(root.path().join("lernie/models.yaml")).unwrap();
    assert!(models.contains("    context_window: 1048576"), "{models}");
    assert!(
        models.contains("the number this provider served"),
        "{models}"
    );
}

/// The honest miss: a row whose roster carries no window — or that was never
/// listed in this wall at all — declares §9.4's default, under the note that
/// says it is one.
#[test]
fn a_pick_with_no_served_window_declares_the_default() {
    let g = spawn_guard();
    let root = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let lernie = script(bin.path(), "lernie", "exit 0\n");
    let fx = workspace();
    let deps = deps_at(root.path(), &lernie, Path::new("/no/bl"));
    seed_wall(&deps, &fx.path, ACME);
    seed_roster(
        &deps,
        &fx.path,
        "acme",
        r#"{"models":[{"default":false,"id":"m-9"}]}"#,
    );
    let reply = fire(&deps, &pick("worker", "acme", "m-9", &fx.path));
    drop(g);
    assert!(
        matches!(&reply, Ok(Reply::Outcome(o)) if o.ok()),
        "{reply:?}"
    );
    let models = fs::read_to_string(root.path().join("lernie/models.yaml")).unwrap();
    let default = crate::model_pick::grammar::DEFAULT_CONTEXT_WINDOW;
    assert!(
        models.contains(&format!("    context_window: {default}")),
        "{models}"
    );
    assert!(
        models.contains("declared defaults, not discoveries"),
        "{models}"
    );
}

#[test]
fn a_pick_needs_a_lineage_it_can_read_the_assignment_from() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let err = fire(&deps, &pick("worker", "acme", "m-9", root.path())).unwrap_err();
    assert!(!err.is_empty(), "{err}");
}

#[test]
fn a_pick_whose_models_file_cannot_be_read_refuses_before_planning() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    fs::create_dir_all(root.path().join("lernie/models.yaml")).unwrap();
    let fx = workspace();
    assert!(fire(&deps, &pick("worker", "acme", "m-9", &fx.path)).is_err());
}
