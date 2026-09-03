//! The named fields (DESIGN §4.1, §6): the four attention `seen` watermarks and
//! the pin list — each a *query over one root map*, each mutator ending on the
//! document's write-through `save`.
//!
//! **Both are world facts**, which since bl-f936 is the only kind this document
//! holds: they are the operator's assertions about the world and every seat
//! shares them. The `collapsed` overrides and `identity_last_used` stood here
//! too and are gone — the first is a fact about a pane of glass and is each
//! seat's own (REMOTE §7 as amended), the second was read and never written, so
//! the `--as` fallback outside a workspace was always `$USER` and now says so.
//!
//! A child module so [`super`] stays inside its line budget (§12): privacy is
//! unaffected (a child sees its ancestor's private fields), and the parent
//! keeps only the file mechanics — forgiving load, echo hash, atomic write.

use super::{UiState, descend, string_array};
use serde_json::Value;

/// The four seen-gated attention kinds (§6); each names one watermark slot in
/// a `seen[ws][agent]` object (unknown kinds round-trip as plain map keys).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeenKind {
    Notify,
    Stopped,
    Budget,
    Conflicted,
    /// The §6 rule-7 flag (bl-6f2f): the evidence is the raising row's
    /// timestamp rather than a ref oid, and behaves identically — a later flag
    /// is a later stamp and fires again.
    Flag,
}

impl SeenKind {
    fn key(self) -> &'static str {
        match self {
            SeenKind::Notify => "notify",
            SeenKind::Stopped => "stopped",
            SeenKind::Budget => "budget",
            SeenKind::Conflicted => "conflicted",
            SeenKind::Flag => "flag",
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
            let by_ws = descend(&mut self.world.root, "seen".to_string());
            let by_agent = descend(by_ws, ws.to_string());
            let slot = descend(by_agent, agent.to_string());
            slot.insert(kind.key().to_string(), Value::String(oid.clone()));
        }
        self.world.save();
    }

    /// True iff the `(kind, ws, agent)` watermark equals `oid` (else unseen).
    pub fn is_seen(&self, kind: SeenKind, ws: &str, agent: &str, oid: &str) -> bool {
        self.world
            .root
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
        string_array(&self.world.root, "pinned")
    }

    /// Replace the pin list.
    pub fn set_pinned(&mut self, list: Vec<String>) {
        let arr = list.into_iter().map(Value::String).collect();
        self.world
            .root
            .insert("pinned".to_string(), Value::Array(arr));
        self.world.save();
    }
}

#[cfg(test)]
mod tests;
