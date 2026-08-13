//! `confinement: required` — a workspace that will not fire a drone while no
//! OS confinement layer exists, at both doors a drone is born through.

use super::*;

#[test]
fn confinement_required_refuses_every_birth_and_absence_gates_nothing() {
    let world = World::new();
    world.repo();
    // No policy at all: the gate is a no-op with nothing configured.
    assert_eq!(confinement_gate(&world.workspace()), Ok(()));
    world.policy("confinement: required\n");
    let err = confinement_gate(&world.workspace()).expect_err("no layer exists");
    assert!(err.contains("confinement"), "{err}");
    assert!(err.contains(CAPABILITY_YAML), "{err}");
    // Removing the line removes the policy, not the code (severability).
    world.policy("confinement: optional\n");
    assert_eq!(confinement_gate(&world.workspace()), Ok(()));
}

#[test]
fn a_workspace_with_no_repo_at_all_gates_nothing() {
    assert_eq!(confinement_gate(Path::new("/no/such/workspace")), Ok(()));
}

/// The gate sits on **both** doors a drone is born through. The attempt's is
/// the chokepoint's `Fork` arm: a workspace that requires a layer nobody has
/// forks nothing, and the refusal names the policy rather than the fork.
#[test]
fn an_attempt_is_refused_by_the_same_gate_a_start_is() {
    let world = World::new();
    world.repo();
    let mut deps = world.deps();
    deps.lernie = Cli::new("/no/such/lernie");
    let attempt = crate::boundary::Action::Fork {
        workspace: world.workspace(),
        parent: "a-1".to_owned(),
        attempt: crate::fork::Attempt {
            from: "config/default".to_owned(),
            role: "worker".to_owned(),
            skills: Vec::new(),
        },
        goal: "go".to_owned(),
    };
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    // Ungated, the fork runs and fails on its own terms — the executor's error,
    // not the policy's.
    let guard = crate::test_support::spawn_guard();
    let ran = crate::boundary::dispatch::dispatch(&deps, &mut ui, "1", &attempt)
        .expect_err("no lernie to fork with");
    drop(guard);
    assert!(!ran.contains("confinement"), "{ran}");
    // Gated, nothing is attempted at all.
    world.policy("confinement: required\n");
    let refused = crate::boundary::dispatch::dispatch(&deps, &mut ui, "2", &attempt)
        .expect_err("no confinement layer exists");
    assert!(refused.contains("confinement"), "{refused}");
}
