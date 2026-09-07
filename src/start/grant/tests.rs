//! The birth grant's fixed point, over the very template litany ships.

use super::*;
use crate::git_tree::tests::fixture::Fixture;
use crate::model_pick::BRANCH;
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

/// The paths `drift` would stage, in order.
fn staged(fixture: &Fixture) -> Vec<String> {
    drift(&fixture.path, BRANCH)
        .into_iter()
        .map(|d| d.rel_path)
        .collect()
}

/// **The regression** (bl-7d33): bl-0460 wrote the grant alone, and litany's
/// descriptions-always rule refuses a fork whose role grants a tool the
/// governing config commit does not describe — so every `/prompt` on main
/// answered `started` and died at the fork. The grant and its description are
/// one fact: both are staged, the half-written state converges, and the steady
/// state stages nothing.
#[test]
fn the_grant_and_the_description_it_is_worthless_without_move_together() {
    let fixture = Fixture::new();
    fixture.commit_other(PROVIDERS_YAML, TEMPLATE_PROVIDERS);
    assert_eq!(staged(&fixture), [PROVIDERS_YAML, SCHEMA_PATH]);

    // bl-0460's half-written state, which is exactly what main shipped: the
    // grant committed, nothing describing it. The description is still due.
    fixture.commit_other(PROVIDERS_YAML, &authored(TEMPLATE_PROVIDERS));
    assert_eq!(staged(&fixture), [SCHEMA_PATH]);

    // Both committed: the steady state stages nothing and spawns nothing.
    fixture.commit_other(SCHEMA_PATH, &described());
    assert!(staged(&fixture).is_empty());
}

/// The description is **the schema the injection declares**, so the config
/// commit and the wire cannot disagree — and it is what an agent reads with
/// `bash`, since litany's bl-55b1 cut made `descriptions/tools/` the callable
/// set. It parses, which is the check litany's own snapshot runs.
#[test]
fn the_description_is_the_declared_schema_verbatim() {
    let said: serde_json::Value = serde_json::from_str(&described()).expect("valid JSON");
    assert_eq!(said, clients::schema());
    assert!(described().ends_with('\n'));
}

/// A worker granted nothing is described nothing: yog does not invent a
/// description for a tool it did not grant, and a lineage it cannot read at
/// all stages nothing rather than authoring into the dark.
#[test]
fn no_grant_means_no_description_and_no_lineage_means_neither() {
    let fixture = Fixture::new();
    fixture.commit_other(
        PROVIDERS_YAML,
        "roles:\n  worker:\n    provider: anthropic\n    model: m\n",
    );
    assert!(staged(&fixture).is_empty());
    // A lineage with no `providers.yaml` at all.
    assert!(drift(&Fixture::new().path, BRANCH).is_empty());
}
