//! The live conversation-name enumeration, over real fixtures — the read is
//! two git calls against a bare repo, so mocking it would test the mock.

use super::living_agents;
use crate::git_tree::tests::fixture::Fixture;

#[test]
fn a_named_agent_answers_with_its_id_and_its_name() {
    let fx = Fixture::new();
    fx.build_agent("20260101T000000Z-aaaa", "one");
    fx.name_agent("20260101T000000Z-aaaa", "pale-otter");
    assert_eq!(
        living_agents(&fx.path),
        vec![(
            "20260101T000000Z-aaaa".to_owned(),
            Some("pale-otter".to_owned())
        )]
    );
}

#[test]
fn an_unnamed_agent_answers_with_no_name_rather_than_a_second_shape() {
    let fx = Fixture::new();
    fx.build_agent("20260101T000000Z-bbbb", "one");
    fx.build_agent("20260101T000000Z-cccc", "two");
    fx.name_agent("20260101T000000Z-cccc", "grey-heron");
    // The first wears an empty `name` blob, which is what litany writes for an
    // unnamed agent — absence and emptiness are one fact.
    fx.name_agent("20260101T000000Z-bbbb", "");
    let mut seen = living_agents(&fx.path);
    seen.sort();
    assert_eq!(
        seen,
        vec![
            ("20260101T000000Z-bbbb".to_owned(), None),
            (
                "20260101T000000Z-cccc".to_owned(),
                Some("grey-heron".to_owned())
            ),
        ]
    );
}

#[test]
fn a_workspace_with_no_readable_repository_addresses_nothing() {
    let dir = tempfile::tempdir().unwrap();
    assert!(living_agents(dir.path()).is_empty());
}
