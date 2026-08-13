//! Per-snapshot memo for frame-side view-model builds (§7.2, bl-e90a).
//!
//! The frame renders snapshots and derives nothing per frame — but the
//! Altitude-2 inspector's view-models (transcript, steps) are functions of
//! disk *and* of frame-owned selection (§5.3), so the worker cannot build
//! them ahead: it does not know which agent is focused or which tab is open.
//! The honest middle is this memo: the frame builds such a view-model **at
//! most once per published snapshot per key**, because every disk fact those
//! builds read is change-tracked by the snapshot itself (`last_action_unix`
//! folds the newest `messages/` mtime, `Agent::stream` the live tail — a
//! change that matters forces a publish, §7.2). A pulse repaint or a scroll
//! frame therefore re-reads nothing: the 60 Hz cost is a pointer compare,
//! and the rebuild cadence is the snapshot's own (debounce-bounded, ≤10/s
//! under a storm).
//!
//! **The snapshot a memo keys on is the *derivation*, never the fold**
//! (bl-54f7, [`AppModel::derivation`](crate::AppModel::derivation)). A memo
//! caches a read of disk, and the two non-derived facts a frame paints — the
//! pending echo and the live tail — are by definition not on disk. Keying one
//! on the rendered snapshot would rebuild every disk read whenever an echo
//! landed or a live character did, which is exactly the cost bl-e90a removed,
//! restored with a new trigger.
//!
//! One slot, not a table: a memo serves one surface, and a surface shows one
//! subject at a time — a new key is a new question and the old answer is not
//! worth keeping (the same shape as [`super::WoundGrace`]).

use std::sync::Arc;

use super::Snapshot;

/// One surface's per-snapshot build cache: the snapshot it answered, the key
/// it answered for, and the answer. See the module doc for why this exists
/// and why snapshot identity is a sound invalidation signal.
pub(crate) struct SnapMemo<K: PartialEq, V> {
    slot: Option<(Arc<Snapshot>, K, V)>,
}

impl<K: PartialEq, V> Default for SnapMemo<K, V> {
    fn default() -> Self {
        Self { slot: None }
    }
}

impl<K: PartialEq, V> SnapMemo<K, V> {
    /// The memoized value for `key` under `snap`, building it only when the
    /// snapshot (by `Arc` identity — the worker never mutates a published one,
    /// §7.2) or the key differs from the held answer. The builder is `dyn` so
    /// `read` has one instantiation per (K, V) pair rather than one per
    /// call-site closure — the §12.1 monomorphized-per-closure llvm-cov
    /// phantom, which the coverage-excluded shell's call sites would otherwise
    /// attribute onto these lines.
    pub(crate) fn read(
        &mut self,
        snap: &Arc<Snapshot>,
        key: K,
        build: &mut dyn FnMut() -> V,
    ) -> &V {
        let hit = match &self.slot {
            Some((held, k, _)) => Arc::ptr_eq(held, snap) && *k == key,
            None => false,
        };
        if !hit {
            self.slot = None;
        }
        let fill = || (Arc::clone(snap), key, build());
        &self.slot.get_or_insert_with(fill).2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn snap() -> Arc<Snapshot> {
        Arc::new(Snapshot::empty(Instant::now()))
    }

    /// The caching contract: one build per (snapshot, key), however many
    /// frames read it.
    #[test]
    fn a_second_frame_on_the_same_snapshot_and_key_builds_nothing() {
        let mut memo: SnapMemo<&str, u32> = SnapMemo::default();
        let snap = snap();
        let mut builds = 0;
        for _ in 0..3 {
            let v = *memo.read(&snap, "agent-1", &mut || {
                builds += 1;
                7
            });
            assert_eq!(v, 7);
        }
        assert_eq!(builds, 1, "same snapshot, same key: one build");
    }

    #[test]
    fn a_new_snapshot_rebuilds_even_under_the_same_key() {
        let mut memo: SnapMemo<&str, u32> = SnapMemo::default();
        let (a, b) = (snap(), snap());
        let mut builds = 0;
        let mut bump = || {
            builds += 1;
            builds
        };
        assert_eq!(*memo.read(&a, "agent-1", &mut bump), 1);
        assert_eq!(*memo.read(&b, "agent-1", &mut bump), 2, "new snapshot");
    }

    /// The shell's two live shapes hold the same contract — the structural
    /// assertion that the steps view and the transcript are each built once
    /// per snapshot, exercised on the exact key/value types the shell keys
    /// (each generic instantiation carries its own coverable copy of `read`).
    #[test]
    fn the_shells_two_memo_shapes_build_once_per_snapshot() {
        use crate::git_tree::AgentState;
        use crate::steps_view::StepsView;
        use crate::transcript::Transcript;
        use std::path::PathBuf;
        let snap = snap();
        let key = || (PathBuf::from("/ws"), "agent-1".to_string(), true);
        let mut builds = 0;
        let mut steps: SnapMemo<(PathBuf, String, AgentState), StepsView> = SnapMemo::default();
        for _ in 0..2 {
            let _ = steps.read(
                &snap,
                (
                    PathBuf::from("/ws"),
                    "agent-1".into(),
                    AgentState::Quiescent,
                ),
                &mut || {
                    builds += 1;
                    StepsView::default()
                },
            );
        }
        let mut tx: SnapMemo<(PathBuf, String, bool), Arc<Transcript>> = SnapMemo::default();
        for _ in 0..2 {
            let _ = tx.read(&snap, key(), &mut || {
                builds += 1;
                Arc::new(Transcript::default())
            });
        }
        assert_eq!(builds, 2, "one steps build + one transcript build");
    }

    #[test]
    fn a_new_key_rebuilds_and_evicts_the_old_answer() {
        let mut memo: SnapMemo<&str, u32> = SnapMemo::default();
        let snap = snap();
        let mut builds = 0;
        let mut bump = || {
            builds += 1;
            builds
        };
        assert_eq!(*memo.read(&snap, "agent-1", &mut bump), 1);
        assert_eq!(*memo.read(&snap, "agent-2", &mut bump), 2, "new key");
        // One slot: returning to the first key is a rebuild, not a hit.
        assert_eq!(*memo.read(&snap, "agent-1", &mut bump), 3);
    }
}
