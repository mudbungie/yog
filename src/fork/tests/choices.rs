//! **S12-T2 fire-time-policy**: what a pinned notch offers, read off a real
//! workspace repo — the fork points, and the model each role names at each of
//! them. The model is shown at the point of choice because it is read from the
//! very file the run will resolve against; nothing here is a yog list.

use crate::fork::{Choices, choices, roles_at};
use crate::git_tree::tests::fixture::Fixture;
use crate::test_support::TEMPLATE_PROVIDERS;
use std::path::Path;

const CONV: &str = "20260803T090000Z-aaaa";

/// A workspace whose config lineage declares the shipped roles.
fn workspace() -> Fixture {
    let fx = Fixture::new();
    fx.commit_other("providers.yaml", TEMPLATE_PROVIDERS);
    fx.build_agent(CONV, "walk the rail");
    fx
}

/// The points are **here** — the pinned commit, a fork carrying the
/// conversation's own history — then every config branch, a clean start each.
/// One control, two kinds of value.
#[test]
fn the_points_are_here_and_every_config_branch() {
    let fx = workspace();
    fx.config_off("strict", "config/default");
    let out = choices(&fx.path, &format!("agents/{CONV}"), Path::new("/no-pool"));
    let labels: Vec<&str> = out.points.iter().map(|p| p.label.as_str()).collect();
    assert_eq!(labels, vec!["here", "default", "strict"]);
    let refs: Vec<&str> = out.points.iter().map(|p| p.refspec.as_str()).collect();
    assert_eq!(
        refs,
        vec![
            format!("agents/{CONV}").as_str(),
            "config/default",
            "config/strict"
        ]
    );
}

/// Every point carries the roles its **governing config commit** declares,
/// each with the model that config binds to it — worker on sonnet, compactor
/// on haiku, exactly as the file says.
#[test]
fn every_point_names_the_model_its_config_binds() {
    let fx = workspace();
    let out = choices(&fx.path, &format!("agents/{CONV}"), Path::new("/no-pool"));
    let here = out
        .point(&format!("agents/{CONV}"))
        .expect("here is offered");
    let named: Vec<(String, String)> = here
        .roles
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
    assert!(out.fireable());
}

/// Two ways a ref declares nothing — no config lineage reaches it, and a
/// lineage with no `providers.yaml` — and both are the same value: no roles.
/// The point still shows, so the operator reads a fact about the workspace
/// rather than a silence.
#[test]
fn a_ref_that_declares_nothing_offers_nothing() {
    let fx = Fixture::new();
    fx.orphan_agent("stray");
    assert!(roles_at(&fx.path, "agents/stray").is_empty());
    // The lineage is reachable, but carries no providers.yaml at all.
    assert!(roles_at(&fx.path, "config/default").is_empty());
    let out = choices(&fx.path, "config/default", Path::new("/no-pool"));
    assert!(!out.fireable(), "nothing to fire ⇒ no seat paints");
    assert!(out.point("config/default").is_some(), "the ref still shows");
    assert!(out.point("nowhere").is_none());
}

/// An empty seat is not fireable and offers no point — the general path with
/// no inputs, which is what a workspace yog cannot read reduces to.
#[test]
fn nothing_offered_is_nothing_to_fire() {
    let empty = Choices::default();
    assert!(!empty.fireable());
    assert!(empty.point("here").is_none());
}
