//! `confinement: required` — a workspace that fires a drone only where the
//! platform's backend proves itself at that very birth, at both doors a drone
//! is born through. The gate's verdict is asserted against the same derivation
//! the product spends ([`crate::control::confine::available`]), so these tests
//! hold on a box with the backend and on one without, and the refusal arms are
//! covered either way (`crate::control::confine::tests` drives them directly).

use super::*;

#[test]
fn confinement_required_gates_births_by_the_derived_availability() {
    let world = World::new();
    world.repo();
    // No policy at all: the gate is a no-op with nothing configured.
    assert_eq!(confinement_gate(&world.workspace()), Ok(()));
    world.policy("confinement: required\n");
    let guard = crate::test_support::spawn_guard();
    let gated = confinement_gate(&world.workspace());
    let derived = crate::control::confine::available();
    drop(guard);
    // The gate IS the derivation — required passes exactly where the backend
    // proves itself at this moment, and nothing anywhere stores the answer.
    assert_eq!(gated.is_ok(), derived.is_ok(), "{gated:?} vs {derived:?}");
    if let Err(err) = gated {
        assert!(err.contains("confinement"), "{err}");
        assert!(err.contains(CAPABILITY_YAML), "{err}");
    }
    // Removing the line removes the policy, not the code (severability).
    world.policy("confinement: optional\n");
    assert_eq!(confinement_gate(&world.workspace()), Ok(()));
}

#[test]
fn a_workspace_with_no_repo_at_all_gates_nothing() {
    assert_eq!(confinement_gate(Path::new("/no/such/workspace")), Ok(()));
}

/// The gate sits on **both** doors a drone is born through; the attempt's is
/// the chokepoint's `Fork` arm. And past the gate the fire is **wrapped**:
/// under a required policy the spawn is the backend's argv around the verb, so
/// on a box whose backend is available the failure below is the *sandbox*
/// reporting the missing tool — never a bare, unconfined exec — and on a box
/// without one the birth is refused by name. No third outcome exists.
#[test]
fn an_attempt_is_gated_and_a_permitted_fire_is_wrapped() {
    let world = World::new();
    world.repo();
    let mut deps = world.deps();
    deps.lernie = Cli::new("/no/such/lernie");
    // A hermetic world root the wrapper can bind (the real one need not exist
    // under a scratch `XDG_DATA_HOME`).
    let data = world.dir.path().join("data");
    std::fs::create_dir_all(data.join("yog").join("world")).unwrap();
    deps.world = crate::xdg::Env::from_pairs([("XDG_DATA_HOME", data.to_string_lossy())]);
    let attempt = crate::boundary::Action::Fork {
        workspace: crate::naming::leaf(&(world.workspace())),
        parent: AGENT.to_owned(),
        attempt: crate::fork::Attempt {
            from: "config/default".to_owned(),
            role: "worker".to_owned(),
            skills: Vec::new(),
        },
        goal: "go".to_owned(),
    };
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    // Ungated, the fork runs bare and fails on its own terms — the executor's
    // error, not the policy's.
    let guard = crate::test_support::spawn_guard();
    let ran = crate::boundary::dispatch::dispatch(&deps, &mut ui, "1", &attempt)
        .expect_err("no lernie to fork with");
    assert!(!ran.contains("confinement"), "{ran}");
    // Gated, the policy decides: the spawn is wrapped where the backend is
    // available, refused by name where it is not.
    world.policy("confinement: required\n");
    let wrapper = crate::control::confine::wrapper(&deps.world, &world.workspace());
    let fired = crate::boundary::dispatch::dispatch(&deps, &mut ui, "2", &attempt);
    let derived = crate::control::confine::available();
    drop(guard);
    // The fold is unconditional under the policy, probe or no probe.
    assert_eq!(wrapper.first().map(String::as_str), Some("bwrap"));
    assert_eq!(wrapper.last().map(String::as_str), Some("--"));
    let ws = world.workspace().to_string_lossy().into_owned();
    assert!(wrapper.contains(&ws), "{wrapper:?}");
    match derived {
        Err(_) => {
            let refused = fired.expect_err("no confinement layer exists here");
            assert!(refused.contains("confinement"), "{refused}");
        }
        Ok(()) => match fired {
            Ok(Reply::Outcome(o)) => {
                assert!(!o.ok(), "the sandboxed exec of a missing tool fails");
                assert!(o.stderr.contains("lernie"), "{}", o.stderr);
            }
            other => panic!("a wrapped fire completes with the backend's own verdict: {other:?}"),
        },
    }
}
