//! The §3.6 `ui.json` prune: a deleted workspace's keys leave the document
//! (step 2 of the unmake plan).
//!
//! Not hygiene. The claimant join is string equality (§3.2), so a workspace
//! raised again under
//! a dead name **re-adopts** its history — deliberately. What must not come back
//! with it is the dead sphere's acknowledgement watermarks and its pin, which
//! are yog's own assertions about a workspace that no longer exists. One
//! write-through save (§4.1) drops all three keys at once.

use super::UiState;
use serde_json::Value;

impl UiState {
    /// Drop every key the workspace `key` (its path — [`ws_key`](crate::nav::ws_key))
    /// owns: its `seen` map, its `pinned` entry, and its `ws:<path>` `collapsed`
    /// override (§4.1). Absent keys are left absent — the prune adds nothing to
    /// the document, so pruning a workspace that owned none writes nothing at all
    /// (the §4.1 content-hash elision).
    pub fn prune_workspace(&mut self, key: &str) {
        if let Some(Value::Object(seen)) = self.root.get_mut("seen") {
            seen.remove(key);
        }
        self.drop_from("pinned", key);
        self.drop_from("collapsed", &format!("ws:{key}"));
        self.save();
    }

    /// Drop a deleted conversation's acknowledgement watermarks: `seen[key][id]`
    /// for `root` and every `<root>-*` hyphen-descendant — the same subtree cut
    /// lernie's `delete` removes (ARCH §2.3), so the two sets cannot drift. The
    /// hyphen boundary is load-bearing: `a-bb` is not pruned by `a-b`'s delete
    /// (a shared byte prefix is nothing, §2.3's whole-token rule).
    pub fn prune_agent(&mut self, key: &str, root: &str) {
        if let Some(Value::Object(seen)) = self.root.get_mut("seen")
            && let Some(Value::Object(by_agent)) = seen.get_mut(key)
        {
            let prefix = format!("{root}-");
            by_agent.retain(|id, _| id != root && !id.starts_with(&prefix));
        }
        self.save();
    }

    /// Remove `value` from the string array at `field`, leaving an absent or
    /// non-array field untouched (the forgiving read, §4.1).
    fn drop_from(&mut self, field: &str, value: &str) {
        let Some(Value::Array(arr)) = self.root.get_mut(field) else {
            return;
        };
        arr.retain(|v| v.as_str() != Some(value));
    }
}

#[cfg(test)]
mod tests {
    use crate::ui_state::{SeenKind, UiState};
    use tempfile::tempdir;

    const WS: &str = "/y/workspaces/alba-koi";
    const OTHER: &str = "/y/workspaces/zeta-pug";

    fn seeded(path: std::path::PathBuf) -> UiState {
        let mut ui = UiState::open(path);
        ui.record_seen(WS, "c-1", &[(SeenKind::Notify, "oid1".to_owned())]);
        ui.record_seen(OTHER, "c-9", &[(SeenKind::Notify, "oid9".to_owned())]);
        ui.set_pinned(vec![WS.to_owned(), OTHER.to_owned()]);
        ui.set_collapsed(&format!("ws:{WS}"), true);
        ui.set_collapsed(&format!("ws:{OTHER}"), true);
        ui
    }

    #[test]
    fn the_prune_drops_seen_pin_and_collapse_and_spares_every_other_workspace() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        let mut ui = seeded(path.clone());
        ui.prune_workspace(WS);

        assert!(!ui.is_seen(SeenKind::Notify, WS, "c-1", "oid1"));
        assert_eq!(ui.pinned(), [OTHER.to_owned()]);
        assert!(!ui.is_collapsed(&format!("ws:{WS}")));
        // The neighbour's own keys are untouched — deletion is per-workspace.
        assert!(ui.is_seen(SeenKind::Notify, OTHER, "c-9", "oid9"));
        assert!(ui.is_collapsed(&format!("ws:{OTHER}")));

        // Write-through (§4.1): the prune is on disk before it returned.
        let reread = UiState::open(path);
        assert_eq!(reread.pinned(), [OTHER.to_owned()]);
        assert!(!reread.is_seen(SeenKind::Notify, WS, "c-1", "oid1"));
    }

    #[test]
    fn the_agent_prune_takes_the_subtree_on_the_hyphen_boundary_and_nothing_else() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        let mut ui = UiState::open(path.clone());
        for id in ["r-aa", "r-aa-c-bb", "r-aab", "z-zz"] {
            ui.record_seen(WS, id, &[(SeenKind::Notify, format!("oid-{id}"))]);
        }
        ui.record_seen(OTHER, "r-aa", &[(SeenKind::Notify, "elsewhere".to_owned())]);
        ui.prune_agent(WS, "r-aa");

        assert!(!ui.is_seen(SeenKind::Notify, WS, "r-aa", "oid-r-aa"));
        assert!(!ui.is_seen(SeenKind::Notify, WS, "r-aa-c-bb", "oid-r-aa-c-bb"));
        // A shared byte prefix is not descent (the whole-token rule).
        assert!(ui.is_seen(SeenKind::Notify, WS, "r-aab", "oid-r-aab"));
        assert!(ui.is_seen(SeenKind::Notify, WS, "z-zz", "oid-z-zz"));
        // Another workspace's same-named agent is untouched.
        assert!(ui.is_seen(SeenKind::Notify, OTHER, "r-aa", "elsewhere"));
        // Write-through (§4.1): the prune is on disk before it returned.
        let reread = UiState::open(path);
        assert!(!reread.is_seen(SeenKind::Notify, WS, "r-aa", "oid-r-aa"));
    }

    #[test]
    fn pruning_an_agent_nobody_acknowledged_adds_no_keys() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        let mut ui = UiState::open(path.clone());
        ui.prune_agent(WS, "r-aa");
        let doc = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
        assert!(
            !doc.contains("seen"),
            "a subtraction, never a schema seeding"
        );
    }

    #[test]
    fn pruning_a_workspace_that_owns_nothing_adds_no_keys() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        let mut ui = UiState::open(path.clone());
        ui.prune_workspace(WS);
        // No `seen`/`pinned`/`collapsed` field existed and none was materialized —
        // the prune is a subtraction, never a schema seeding.
        assert!(ui.pinned().is_empty());
        assert!(!ui.is_collapsed(&format!("ws:{WS}")));
        let doc = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
        assert!(!doc.contains("pinned") && !doc.contains("collapsed") && !doc.contains("seen"));
    }
}
