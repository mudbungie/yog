//! §8.7 tag→lineage selection, over real-git workspaces (the shared
//! [`Fixture`](crate::git_tree::tests::fixture::Fixture), which ships one
//! `config/default` and grows lineages with `config_off`).

use super::select;
use crate::git_tree::tests::fixture::Fixture;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload};
use tempfile::tempdir;

/// A ball rung carrying `tags`, ready and startable.
fn ball(tags: &[&str]) -> Payload {
    Payload::Ball {
        project: "dev/proj".to_owned(),
        ball: BallSpec::Existing {
            id: "bl-1111".to_owned(),
            title: "t".to_owned(),
            body: "b".to_owned(),
            join: JoinState::ReadyStartable,
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
        },
    }
}

#[test]
fn a_tag_naming_a_lineage_selects_it() {
    let fx = Fixture::new();
    fx.config_off("deep", "config/default");
    assert_eq!(select(&fx.path, &ball(&["deep"])), Some("deep".to_owned()));
}

#[test]
fn the_first_tag_naming_a_lineage_wins() {
    // The conflict rule: the ball's own tag order, not a yog-side priority.
    // `quick` sits before `deep`, and both name lineages.
    let fx = Fixture::new();
    fx.config_off("deep", "config/default");
    fx.config_off("quick", "config/default");
    assert_eq!(
        select(&fx.path, &ball(&["missing", "quick", "deep"])),
        Some("quick".to_owned()),
    );
}

#[test]
fn a_tag_naming_no_lineage_selects_the_default() {
    // No policy for this tag, so no flag: `None` is litany's own
    // `config/default`, never a name yog spells.
    let fx = Fixture::new();
    assert_eq!(select(&fx.path, &ball(&["deep"])), None);
}

#[test]
fn an_untagged_ball_and_the_tagless_rungs_are_one_case() {
    let fx = Fixture::new();
    fx.config_off("deep", "config/default");
    assert_eq!(select(&fx.path, &ball(&[])), None);
    assert_eq!(select(&fx.path, &Payload::Bare), None);
    assert_eq!(
        select(
            &fx.path,
            &Payload::Ball {
                project: "dev/proj".to_owned(),
                ball: BallSpec::New {
                    title: "t".to_owned(),
                    body: "b".to_owned(),
                },
            },
        ),
        None,
    );
}

#[test]
fn a_workspace_with_no_repository_selects_the_default() {
    // The bootstrap start: `EnsureWorkspace` has not run, so there is no ref to
    // enumerate and the git failure IS the answer.
    let dir = tempdir().unwrap();
    assert_eq!(select(dir.path(), &ball(&["deep"])), None);
}
