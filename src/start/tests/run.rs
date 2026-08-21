//! [`prepare`] end-to-end per rung (§8.1): the executor runs the planner's
//! output in the amended order (substrate → `bl` mutations → deferred prompt),
//! converging on skips and re-planning a new ball. The abort-before-claim proof
//! (a substrate failure precedes every `bl` mutation) is the load-bearing order.

use super::{World, ball, fake_bl, fake_fail};
use crate::binding::{work_worktree_path, workspace_path};
use crate::cli_outbound::Cli;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Deps, Payload, prepare};
use std::path::Path;

fn deps(w: &World, bl: &Cli, lernie: &Cli) -> Deps {
    Deps {
        bl: bl.clone(),
        lernie: lernie.clone(),
        state_root: w.state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
    }
}

/// Materialize a seeded home (`models.yaml`) so `prime` skips.
fn mark_seeded(w: &World) {
    let lernie = crate::world::layout_under(w.yog.path()).lernie;
    std::fs::create_dir_all(&lernie).unwrap();
    std::fs::write(lernie.join("models.yaml"), b"models: {}\n").unwrap();
}

#[test]
fn prepare_bare_bootstrap_seeds_and_news_under_the_default_name() {
    let w = World::new();
    let lernie = w.lernie();
    let bl = Cli::new("/no/bl"); // no ball rung → bl never runs
    let inputs = w.inputs(crate::names::DEFAULT_NAME, Payload::Bare);
    let p = prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    assert_eq!(
        p.workspace, "home",
        "the bootstrap names without asking (§3.1)"
    );
    assert_eq!(p.binding, None, "the bare rung binds no work target");
    assert_eq!(p.goal, "", "the operator types the bare payload");
    // Substrate only, in order — no `bl` mutation.
    assert_eq!(w.verbs(), vec!["prime", "new"]);
    assert_eq!(
        w.ops()[1].argv[2],
        workspace_path(w.yog.path(), &p.workspace).to_string_lossy(),
        "`lernie new` targets `<names-root>/home`"
    );
}

/// §16.7 W9: founding the world seeds the agent-tool shim, so the `bl` an agent
/// finds on the world `PATH` exists before any driver does. It re-execs the very
/// `Cli` yog drives `bl` through, and it lands under `<world>/tools` — the dir
/// the override set fronts `PATH` with.
#[test]
fn prepare_seeds_the_world_bl_shim_before_the_prompt() {
    let w = World::new();
    let lernie = w.lernie();
    let bl = Cli::new("/some/bl");
    let inputs = w.inputs(crate::names::DEFAULT_NAME, Payload::Bare);
    prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    let tools = crate::world::layout_under(w.yog.path()).tools;
    let shim = tools.join(crate::world::tools::BL);
    assert_eq!(
        std::fs::read_to_string(&shim).unwrap(),
        crate::world::tools::shim_script(crate::world::tools::BL, &bl.exec_words()),
    );
    assert_eq!(
        crate::world::tools::prepend_path(&tools, Some("/bin".to_owned())),
        format!("{}:/bin", tools.display()),
        "the seeded dir is the one the world PATH fronts",
    );
}

#[test]
fn prepare_prompt_into_existing_skips_prime_new_and_mint() {
    let w = World::new();
    mark_seeded(&w);
    let name = "cobalt-gecko";
    std::fs::create_dir_all(workspace_path(w.yog.path(), name).join("repo.git")).unwrap();
    let lernie = Cli::new("/no/lernie"); // a spawn would surface as an error
    let bl = Cli::new("/no/bl");
    let inputs = w.inputs(name, Payload::Bare);
    let p = prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    assert_eq!(p.workspace, name);
    assert!(
        w.ops().is_empty(),
        "seeded + existing → nothing spawns (S1-T1)"
    );
}

#[test]
fn prepare_path_rung_composes_the_target_and_runs_no_bl() {
    let w = World::new();
    let lernie = w.lernie();
    let bl = Cli::new("/no/bl");
    let dir = w.home.path().join("work");
    let inputs = w.inputs("cobalt-gecko", Payload::Path { dir: dir.clone() });
    let p = prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    assert_eq!(
        p.binding.as_ref(),
        Some(&dir),
        "the binding is the directory"
    );
    assert!(p.goal.contains(&dir.display().to_string()));
    assert!(!w.verbs().iter().any(|v| v == "claim" || v == "create"));
}

#[test]
fn prepare_ball_ready_claims_after_new() {
    let w = World::new();
    let canonical = work_worktree_path(w.balls.path(), w.project.path(), "bl-r", None);
    let bl = Cli::new(fake_bl(w.bin.path(), "x", &canonical));
    let lernie = w.lernie();
    let inputs = w.inputs(
        "cobalt-gecko",
        ball(w.project.path(), "bl-r", JoinState::ReadyStartable),
    );
    let p = prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    assert_eq!(
        p.binding.as_ref(),
        Some(&canonical),
        "the binding is the claim's worktree"
    );
    assert!(p.goal.contains("Ball bl-r: T"));
    // The amended §8.1 order: seed → new → claim.
    assert_eq!(w.verbs(), vec!["prime", "new", "claim"]);
    assert_eq!(
        &w.ops()[2].argv[1..],
        &["claim", "bl-r", "--as", "cobalt-gecko"],
        "claim stamped with the workspace name"
    );
}

#[test]
fn prepare_new_ball_creates_then_converges_to_one_claim() {
    let w = World::new();
    let canonical = work_worktree_path(w.balls.path(), w.project.path(), "bl-mint", None);
    let bl = Cli::new(fake_bl(w.bin.path(), "bl-mint", &canonical));
    let lernie = w.lernie();
    let payload = Payload::Ball {
        project: crate::naming::leaf(w.project.path()),
        ball: BallSpec::New {
            title: "Fresh".to_owned(),
            body: "New body".to_owned(),
        },
    };
    let inputs = w.inputs("cobalt-gecko", payload);
    let p = prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    assert!(
        p.goal.contains("Ball bl-mint: Fresh"),
        "re-planned as existing"
    );
    // Substrate before every `bl`, one claim (the re-plan's seed/new skip).
    assert_eq!(w.verbs(), vec!["prime", "new", "create", "claim"]);
}

#[test]
fn prepare_bound_ball_resumes_without_a_claim_or_mint() {
    let w = World::new();
    mark_seeded(&w);
    std::fs::create_dir_all(workspace_path(w.yog.path(), "cobalt-gecko").join("repo.git")).unwrap();
    // A bl that would *fail* if a claim ran — proof the claim is skipped.
    let bl = Cli::new(fake_fail(w.bin.path(), "bl", "should not run"));
    let lernie = Cli::new("/no/lernie");
    let inputs = w.inputs(
        "cobalt-gecko",
        ball(w.project.path(), "bl-c", JoinState::Bound),
    );
    let p = prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    assert_eq!(
        p.binding,
        Some(work_worktree_path(
            w.balls.path(),
            w.project.path(),
            "bl-c",
            None
        ))
    );
    assert!(w.ops().is_empty(), "resume: no claim, no mint, no re-seed");
}

/// §8.1, bl-7fc8: the pinned template already grants the worker role the whole
/// tool pool (`message` and `dispatch` included), so a freshly authored
/// workspace needs no second commit — the exact bytes `lernie new` committed
/// are the exact bytes still on `config/default` once `prepare` returns.
#[test]
fn a_fresh_workspace_keeps_the_templates_grant_with_no_extra_commit() {
    let w = World::new();
    let lernie = w.lernie();
    let bl = Cli::new("/no/bl");
    let inputs = w.inputs(crate::names::DEFAULT_NAME, Payload::Bare);
    let p = prepare(&deps(&w, &bl, &lernie), &inputs, "TS").unwrap();
    // No `config` step: the template's grant is already complete.
    assert_eq!(w.verbs(), vec!["prime", "new"]);
    let committed = crate::config_edit::branch::config_file(
        &workspace_path(w.yog.path(), &p.workspace),
        "config/default",
        "providers.yaml",
    )
    .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&committed),
        crate::test_support::TEMPLATE_PROVIDERS,
        "the workspace's first commit is untouched — yog is not a second policy \
         authority over it",
    );
    assert!(
        String::from_utf8_lossy(&committed).contains("message")
            && String::from_utf8_lossy(&committed).contains("dispatch"),
        "the worker role already carries yog's two agent-to-agent primitives",
    );
}

#[test]
fn prepare_ball_aborts_before_any_bl_on_a_substrate_failure() {
    // S3-T6 load-bearing order: a failed `prime` precedes every `bl` mutation, so
    // no `bl create`/`bl claim` is recorded — the orphaned-claim wound is closed.
    let w = World::new();
    let bl = Cli::new(fake_bl(w.bin.path(), "x", Path::new("/wt")));
    let lernie = Cli::new(fake_fail(w.bin.path(), "lernie", "no seed"));
    let inputs = w.inputs(
        "cobalt-gecko",
        ball(w.project.path(), "bl-r", JoinState::ReadyStartable),
    );
    assert!(prepare(&deps(&w, &bl, &lernie), &inputs, "TS").is_err());
    assert_eq!(
        w.verbs(),
        vec!["prime"],
        "aborted at the seed — no bl mutation"
    );
}
