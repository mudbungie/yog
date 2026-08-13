//! The named fields of `ui.json` (DESIGN §4.1, §6): the four attention `seen`
//! watermarks, the pin list, the collapse overrides and the last-used identity
//! — each a *query over the one root map*, each mutator ending on [`super`]'s
//! write-through `save`.
//!
//! A child module so [`super`] stays inside its line budget (§12), on the same
//! terms as [`super::knobs`]: privacy is unaffected (a child sees its
//! ancestor's private fields), and the parent keeps only the file mechanics —
//! forgiving load, echo hash, atomic write.

use super::{UiState, descend, string_array};
use serde_json::Value;
use std::collections::BTreeSet;

/// The four seen-gated attention kinds (§6); each names one watermark slot in
/// a `seen[ws][agent]` object (unknown kinds round-trip as plain map keys).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeenKind {
    Notify,
    Stopped,
    Budget,
    Conflicted,
}

impl SeenKind {
    fn key(self) -> &'static str {
        match self {
            SeenKind::Notify => "notify",
            SeenKind::Stopped => "stopped",
            SeenKind::Budget => "budget",
            SeenKind::Conflicted => "conflicted",
        }
    }
}

impl UiState {
    /// Record every `(kind, oid)` in `marks` as a `(ws, agent)` seen watermark
    /// (§6). The whole acknowledgement gesture is one call and therefore one
    /// write — focusing an agent carrying four signals is not four writes, and
    /// an agent carrying none (a phantom, §3.5) descends into nothing, so no
    /// empty slot is materialized and the document is left untouched.
    pub fn record_seen(&mut self, ws: &str, agent: &str, marks: &[(SeenKind, String)]) {
        for (kind, oid) in marks {
            let by_ws = descend(&mut self.root, "seen".to_string());
            let by_agent = descend(by_ws, ws.to_string());
            let slot = descend(by_agent, agent.to_string());
            slot.insert(kind.key().to_string(), Value::String(oid.clone()));
        }
        self.save();
    }

    /// True iff the `(kind, ws, agent)` watermark equals `oid` (else unseen).
    pub fn is_seen(&self, kind: SeenKind, ws: &str, agent: &str, oid: &str) -> bool {
        self.root
            .get("seen")
            .and_then(Value::as_object)
            .and_then(|m| m.get(ws))
            .and_then(Value::as_object)
            .and_then(|m| m.get(agent))
            .and_then(Value::as_object)
            .and_then(|m| m.get(kind.key()))
            .and_then(Value::as_str)
            == Some(oid)
    }

    /// The ordered pin list (user order preserved; non-strings ignored).
    pub fn pinned(&self) -> Vec<String> {
        string_array(&self.root, "pinned")
    }

    /// Replace the pin list.
    pub fn set_pinned(&mut self, list: Vec<String>) {
        let arr = list.into_iter().map(Value::String).collect();
        self.root.insert("pinned".to_string(), Value::Array(arr));
        self.save();
    }

    /// Whether `key` (`proj:…` / `ws:…`) carries an explicit collapse override.
    pub fn is_collapsed(&self, key: &str) -> bool {
        string_array(&self.root, "collapsed").contains(&key.to_string())
    }

    /// Add/remove a collapse override, kept sorted for byte-determinism.
    pub fn set_collapsed(&mut self, key: &str, collapsed: bool) {
        let mut set: BTreeSet<String> = BTreeSet::new();
        set.extend(string_array(&self.root, "collapsed"));
        if collapsed {
            set.insert(key.to_string());
        } else {
            set.remove(key);
        }
        let arr = set.into_iter().map(Value::String).collect();
        self.root.insert("collapsed".to_string(), Value::Array(arr));
        self.save();
    }

    /// The identity prefilling `--as` in the claim dialog, if recorded.
    pub fn identity_last_used(&self) -> Option<String> {
        self.root
            .get("identity_last_used")
            .and_then(Value::as_str)
            .map(String::from)
    }

    pub fn set_identity(&mut self, identity: &str) {
        self.root.insert(
            "identity_last_used".to_string(),
            Value::String(identity.to_string()),
        );
        self.save();
    }
}

#[cfg(test)]
mod tests;
