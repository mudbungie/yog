//! Tests for the git-tree view-model.
//!
//! Tests are split by concern: [`fixture`] owns the shared workspace-
//! building helpers (its one `git` fork site lives in [`git`]) and
//! [`disk_fixture`] the ones writing plain files git never sees, [`unit`]
//! drives the pure-function layer (detection,
//! parsing, preview extraction), and [`repo`] covers the end-to-end
//! [`super::GitTree::from_repo`] flow against real tempdir-backed
//! workspaces — its skeleton half, with [`steps`] holding the step-content
//! projection (streaming text, tool calls) and [`activity`] the §11 recency
//! fact folded from the tip, `messages/` and the live tail (bl-cad5). Agent-state coverage (§3.5) lives in
//! [`state_repo`] (end-to-end quiescent/stopped classification against
//! real fixtures) and [`state_unit`] (the probe-injected tri-state mapping
//! table, where the `live`/`in_flight` and `Unknown`-uncertainty rows are
//! proven); the procfs probe backends are unit-tested in `super::fd_probe`
//! and `super::lock_probe`. The state-badge glyph/hue mapping lives in
//! `theme` (§11's single colour authority) with its tests; badge paint
//! reachability is the shell acceptance smoke's concern.

mod activity;
pub(crate) mod config_fixture;
mod disk_fixture;
pub(crate) mod fixture;
pub(crate) mod git;
mod repo;
mod starts;
mod state_repo;
mod state_unit;
mod steps;
mod unit;

/// A path's mtime in whole unix seconds, the way every derivation that dates a
/// file reads it. One spelling shared by the two suites that assert on a stamp —
/// [`activity`]'s recency fold and [`starts`]'s elapsed starts.
fn mtime(path: &std::path::Path) -> i64 {
    let modified = std::fs::metadata(path).unwrap().modified().unwrap();
    i64::try_from(
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap()
}
