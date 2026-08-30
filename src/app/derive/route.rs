//! Where a dirty root goes (DESIGN §7.1, §7.2).
//!
//! One table, and it is the *whole* frame→worker vocabulary: the frame never
//! sends the worker a request, it names a root that changed, and this decides
//! what that means. Split from [`super`] per §12's budget — that file is the
//! pass, this is the routing it opens with.
//!
//! Reading `ui.json` lives here rather than on the frame for the same reason
//! everything else does: it is disk I/O. The *document* stays frame-owned
//! (write-through at the gesture, §4.1), so the bytes ride out on the snapshot
//! and the frame decides whether they are its own echo. The derivation reads
//! **no** knob out of them: `show_internal` — a filter over which clones to
//! walk — was the only one, and bl-e3e7 deleted it, so `ui.json` is now purely
//! a view document the worker ferries and never interprets.

use super::Deriver;
use crate::watch::Mark;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

impl Deriver {
    /// Route each dirty root by kind (§7.1): the yog-state root re-reads
    /// `ui.json` and the ops tail; the balls-clones root re-fetches every ball
    /// list; an **enumerated project** re-fetches that one project (§7.2 fetch
    /// cadence, §5.1 #4); an enumeration root re-enumerates + reconciles;
    /// anything else is a workspace root and opens a debounce window.
    /// Everything here was announced, so nothing it finds is drift.
    pub(super) fn dispatch_dirty(&mut self, roots: BTreeMap<PathBuf, Mark>) {
        let mut workspaces = Vec::new();
        for (root, mark) in roots {
            if root == self.roots.yog_state {
                self.adopt_ui();
                self.adopt_cadence();
                self.refresh_ops();
            } else if root == self.roots.balls_clones {
                self.refresh_balls();
            } else if self.projects.contains(&root) {
                self.refetch_project(&root);
            } else if self.is_enum_root(&root) {
                self.reconcile(Mark::Watch);
            } else {
                workspaces.push((root, mark));
            }
        }
        self.schedule.mark(workspaces);
    }

    /// An enumeration root (§7.1): the flat names root, the litany workspaces
    /// root, or the replays root — a create/remove there changes the workspace
    /// set.
    fn is_enum_root(&self, root: &Path) -> bool {
        root == self.roots.names()
            || root == self.roots.workspaces()
            || root == self.roots.replays()
    }

    /// Read `ui.json` (§4.1, I5) and carry the bytes to the frame, which owns
    /// the document and decides whether they are its own echo. The worker itself
    /// needs **no** knob out of it: `show_internal` was the only one — a filter
    /// over which clones to walk — and deleting it (bl-e3e7) left the derivation
    /// independent of the view document. A missing/unreadable file is left alone.
    fn adopt_ui(&mut self) {
        let Ok(bytes) = std::fs::read(self.roots.ui_json()) else {
            return;
        };
        self.ui_bytes = Some(bytes);
        self.changed = true;
    }

    /// Read `cadence.yaml` (bl-3381) and, when the periods changed, re-tune the
    /// schedule and republish — the clock's one setting, adopted like any other
    /// announced change. Unlike [`adopt_ui`](Self::adopt_ui), a missing file is
    /// **not** left alone: [`parse`](crate::app::cadence::parse) is total, so
    /// deleting the file is the reset to defaults, and a hand-broken one
    /// degrades to the shipped rhythm rather than to whatever came before.
    ///
    /// The §4.3 fleet arming rides the same read (bl-66fb): it is the same file,
    /// announced by the same watch, and reading it twice would be two answers
    /// to one question. An entry that names no project or no readable cap is
    /// **not armed** — [`crate::fleet::arming::policy`] declines it, so a
    /// half-written file arms nothing rather than arming a guess.
    pub(super) fn adopt_cadence(&mut self) {
        let text =
            std::fs::read_to_string(self.roots.yog_state.join(crate::app::cadence::CADENCE_YAML))
                .unwrap_or_default();
        let next = crate::app::cadence::parse(&text);
        if next != self.cadence {
            self.cadence = next;
            self.schedule.set_cadence(next);
            self.changed = true;
        }
        let fleet: std::collections::BTreeMap<String, crate::fleet::Policy> =
            crate::fleet::arming::armed(&text)
                .into_iter()
                .filter_map(|key| {
                    let policy = crate::fleet::arming::policy(&text, &key)?;
                    Some((key, policy))
                })
                .collect();
        if fleet != self.fleet {
            self.fleet = fleet;
            self.changed = true;
        }
    }

    /// Read the §9.2 global `models.yaml` for the context windows it declares
    /// (§5.1 #35), republishing only when they moved. Total like
    /// [`adopt_cadence`](Self::adopt_cadence): a missing or hand-broken file
    /// declares nothing, and nothing declared renders no figure — never a
    /// stale one.
    ///
    /// It rides the **boot and the 15 s full sweep** rather than a watch of its
    /// own. The file is world-global and hand-edited (§9.2), so it changes at
    /// operator speed; arming a fifth root over one file would buy latency
    /// nobody can perceive at the cost of a root kind, a reconcile arm and a
    /// routing branch (§7.1's roots are the enumeration set, not a file list).
    pub(super) fn adopt_windows(&mut self) {
        let path =
            crate::config_edit::litany_global::LitanyGlobal::resolve(&self.roots.world).models();
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let next = crate::model_pick::grammar::context_windows(&text);
        if next != self.windows {
            self.windows = next;
            self.changed = true;
        }
    }
}
