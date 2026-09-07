//! The birth grant's fixed point, over the very template litany ships.

use super::*;
use crate::test_support::TEMPLATE_PROVIDERS;

/// **The defect** (bl-0460): litany's shipped template cannot name `clients`,
/// because the tool is yog's — so after bl-52b7 the worker was declared nothing
/// at all and no role could reach a foot. The grant adds the one name, and
/// authoring an authored file reproduces it byte for byte.
#[test]
fn the_worker_gains_clients_and_authoring_is_a_fixed_point() {
    let once = authored(TEMPLATE_PROVIDERS);
    assert_ne!(once, TEMPLATE_PROVIDERS, "the shipped template drifts");
    let granted = entry_field(&once, ROLES, WORKER_ROLE, TOOLS)
        .and_then(|v| flow_members(&v))
        .expect("the worker's grant");
    assert!(granted.iter().any(|n| n == clients::NAME), "{granted:?}");
    // Every tool litany granted survives — the file is the operator's and
    // litany's, and this asserts one name inside it.
    for shipped in ["apply_patch", "bash", "dispatch", "read_file"] {
        assert!(granted.iter().any(|n| n == shipped), "{granted:?}");
    }
    assert_eq!(authored(&once), once, "the fixed point");
}

/// **Machinery is never granted.** The compactor's empty grant is the
/// confinement bl-52b7 restored, and the reviewer's list is stated as its own
/// confinement in litany's template: neither is touched, and neither gains a
/// `tools:` line it did not have.
#[test]
fn the_checkpoint_roles_are_left_exactly_as_they_were() {
    let once = authored(TEMPLATE_PROVIDERS);
    assert_eq!(entry_field(&once, ROLES, "compactor", TOOLS), None);
    assert_eq!(
        entry_field(&once, ROLES, "reviewer", TOOLS),
        entry_field(TEMPLATE_PROVIDERS, ROLES, "reviewer", TOOLS),
    );
}

/// A worker that declares no grant, or one written in a form the §9.4 grammar
/// does not read, is the operator's own file and comes back unchanged: an
/// absent list is a role granted nothing, which is a thing an operator may
/// mean.
#[test]
fn a_file_this_has_nothing_to_say_about_is_unchanged() {
    for base in [
        "",
        "roles:\n  worker:\n    provider: anthropic\n    model: m\n",
        "roles:\n  worker:\n    provider: anthropic\n    model: m\n    tools:\n      - bash\n",
        "roles:\n  compactor:\n    provider: anthropic\n    model: m\n",
    ] {
        assert_eq!(authored(base), base, "{base:?}");
    }
}
