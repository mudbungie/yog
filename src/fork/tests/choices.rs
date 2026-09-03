//! **S12-T2 fire-time-policy**: what a ref declares, read off a real workspace
//! repo — the model each role names at the fork point. The model is shown at
//! the point of choice because it is read from the very file the run will
//! resolve against; nothing here is a yog list. What a *seat offers* — the
//! point list and the pool listing — left with the composer (bl-7cc8).

use crate::fork::roles_at;
use crate::git_tree::tests::fixture::Fixture;
use crate::test_support::TEMPLATE_PROVIDERS;

const CONV: &str = "20260803T090000Z-aaaa";

/// A workspace whose config lineage declares the shipped roles.
fn workspace() -> Fixture {
    let fx = Fixture::new();
    fx.commit_other("providers.yaml", TEMPLATE_PROVIDERS);
    fx.build_agent(CONV, "walk the rail");
    fx
}

/// A ref carries the roles its **governing config commit** declares, each with
/// the model that config binds to it — worker on sonnet, compactor on haiku,
/// exactly as the file says.
#[test]
fn every_point_names_the_model_its_config_binds() {
    let fx = workspace();
    let named: Vec<(String, String)> = roles_at(&fx.path, &format!("agents/{CONV}"))
        .iter()
        .map(|r| (r.role.clone(), r.model.clone()))
        .collect();
    assert_eq!(
        named,
        vec![
            ("worker".to_owned(), "claude-sonnet-5".to_owned()),
            ("compactor".to_owned(), "claude-haiku-4-5".to_owned()),
        ]
    );
}

/// Two ways a ref declares nothing — no config lineage reaches it, and a
/// lineage with no `providers.yaml` — and both are the same value: no roles.
#[test]
fn a_ref_that_declares_nothing_offers_nothing() {
    let fx = Fixture::new();
    fx.orphan_agent("stray");
    assert!(roles_at(&fx.path, "agents/stray").is_empty());
    // The lineage is reachable, but carries no providers.yaml at all.
    assert!(roles_at(&fx.path, "config/default").is_empty());
}
