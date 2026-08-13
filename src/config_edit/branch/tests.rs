//! Tests for the config-branch browse surface (§9.3) and governing-config
//! derivation (§5.1 #17), over real-git workspaces built by the shared
//! [`Fixture`] (extended in `git_tree::tests::config_fixture`). Split by
//! concern to stay under the 300-line source cap: [`browse`] covers the
//! branch list / tree / file reads and the line parser; [`governing`] covers
//! the merge-base fold and every one of its arms.
//!
//! [`Fixture`]: crate::git_tree::tests::fixture::Fixture

mod browse;
mod governing;
