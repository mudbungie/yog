//! The roster, attention strip, and seen-acknowledgement surface of
//! [`AppModel`] (DESIGN §6, §11, §15 Y11).
//!
//! All read paths derive attention from the snapshot map plus the one `ui.json`
//! query the module needs — "is this evidence oid acknowledged?" — so the
//! seen-lookup closure is built here from [`UiState::is_seen`] and handed to the
//! pure [`attention`] functions. The write path is the **acknowledgement
//! state**: focusing an agent ([`focus_agent`](AppModel::focus_agent)) records
//! that agent's current evidence oids as seen (§6), and
//! [`ack_focused`](AppModel::ack_focused) re-records them on every tick it stays
//! focused — the ack is a state, not a one-shot gesture (§6, bl-aa1f), so
//! evidence landing on the conversation you are reading is already seen. Durable
//! and converging (§13.1), never mirrored live focus.
//!
//! The one focus no gesture can set outright is the §3.4 start claim
//! ([`await_conversation`](AppModel::await_conversation)): the conversation a
//! fire focuses does not exist yet, so the claim is held by the minted §3.3
//! name and spent by [`adopt_started`](AppModel::adopt_started). That claim and
//! the §7.2 echo it carries live in [`claim`], split off at §12's budget on the
//! seam this paragraph already drew (bl-78d8).

/// The §3.4 claim and its echo — see the module doc above.
mod claim;

use super::{AppModel, Focus};
use crate::attention;
use crate::boundary::answer::queue;
use crate::keymap::InspectorTab;
use crate::nav::ws_key;
use crate::ui_state::SeenKind;
use std::path::{Path, PathBuf};

impl AppModel {
    /// Focus a workspace by its §3.1 **name** (center-panel target) without
    /// selecting an agent — no acknowledgement (§6: focus records seen only for
    /// a *selected agent*). The selected inspector tab is sticky across the
    /// move (§11).
    ///
    /// The name rather than a path since bl-7407: the wire spelling, so a
    /// selection and the gesture it becomes say the same word. A name nothing
    /// enumerates simply resolves to nothing at the doors — the general path
    /// with an empty set, exactly as an unfetched path did.
    pub fn focus_workspace(&mut self, name: &str) {
        self.focus = Focus {
            ws: Some(name.to_owned()),
            agent: None,
            tab: self.focus.tab,
        };
    }

    /// Focus an agent — the **acknowledgement** (§6): select it in the inspector
    /// and record its current evidence oids as seen, converging the ack across
    /// instances (§13.1). The selected inspector tab is sticky (§11). The ack
    /// does not end here: [`ack_focused`](Self::ack_focused) keeps it stamped
    /// for as long as the agent stays focused.
    /// It takes the workspace **path** because the acknowledgement it writes is
    /// keyed by one (§4.1 `seen`, durable): the path is the caller's already —
    /// off the §6 roster, off the echo, off a search hit — so resolving it back
    /// out of a name here would be a join over a fact the caller holds.
    pub fn focus_agent(&mut self, ws: &Path, agent_id: &str) {
        self.focus = Focus {
            ws: Some(crate::naming::leaf(ws)),
            agent: Some(agent_id.to_string()),
            tab: self.focus.tab,
        };
        self.record_seen(ws, agent_id);
    }

    /// Re-record the focused agent's present evidence as seen — the ack held as
    /// a **state** rather than spent as a gesture (§6, bl-aa1f). The tick calls
    /// it every frame, so evidence that lands while you are already looking at a
    /// conversation never raises the flag at the thing you are reading:
    /// attention is evidence that arrived while you weren't looking. Free by
    /// §4.1's write discipline — an unchanged document hashes to what is already
    /// on disk and elides the write outright, so this costs a write only on the
    /// frame the evidence actually lands. A focused workspace with no agent
    /// selected acks nothing (the general empty path).
    pub(super) fn ack_focused(&mut self) {
        if let (Some(ws), Some(agent)) = (self.focused_workspace(), self.focus.agent.clone()) {
            self.record_seen(&ws, &agent);
        }
    }

    /// The selected §11 Altitude-2 inspector tab (RAM, §5.3) — the digit-key
    /// nav target and the shell's tab-strip highlight.
    pub fn inspector_tab(&self) -> InspectorTab {
        self.focus.tab
    }

    /// Select an inspector tab (§11 digit keys / tab-strip click). Viewport
    /// ephemera: no `ui.json` write, no acknowledgement.
    pub fn select_tab(&mut self, tab: InspectorTab) {
        self.focus.tab = tab;
    }

    /// The agent with `id` in `ws`'s snapshot, if both are present.
    fn agent_in(&self, ws: &Path, id: &str) -> Option<&crate::git_tree::Agent> {
        self.snap
            .trees
            .get(ws)?
            .agents
            .iter()
            .find(|a| a.agent_id == id)
    }

    /// Record every present evidence oid for `(ws, agent)` as seen (§6): notify,
    /// rest (the branch tip, unless abandoned), budget, conflicted. Recording
    /// the *current* oid acknowledges it; a later moved ref re-arms (§4.1). A
    /// phantom agent contributes no evidence, so this is a no-op for it.
    fn record_seen(&mut self, ws: &Path, agent_id: &str) {
        let key = ws_key(ws);
        let marks = self.evidence_oids(ws, agent_id);
        self.ui.record_seen(&key, agent_id, &marks);
    }

    /// The present `(kind, oid)` acknowledgement evidence for `(ws, agent)` — an
    /// owned list so [`record_seen`](Self::record_seen)'s mutable `ui` write does
    /// not overlap the snapshot borrow. Empty when the agent is absent.
    /// [`attention::evidence`] is the one definition — read here and by the
    /// §8.5 `seen` action — so the window's ack and a headless one write the
    /// same bytes, and widening a signal (bl-2194) could not leave either
    /// behind.
    fn evidence_oids(&self, ws: &Path, agent_id: &str) -> Vec<(SeenKind, String)> {
        self.agent_in(ws, agent_id)
            .map(attention::evidence)
            .unwrap_or_default()
    }

    /// Jump-to-next-attention (§6): advance focus to the next attention-bearing
    /// agent after the current focus (wrapping), and acknowledge it — the strip
    /// control that walks the operator through everything that needs them.
    ///
    /// The roster it walks is [`queue::roster`] — **one** build, shared with the
    /// §8.5 decision queue, so what this control walks and what a headless seat
    /// is handed can never be two orders. It is the sole consumer of §6's rank
    /// in the window: the ↑/↓ keys left it with bl-fa82 and now step the focused
    /// workspace's visible list rows in paint order (§11's unfold ruling).
    pub fn jump_next_attention(&mut self) {
        let roster = queue::roster(&self.snap, &self.ui);
        // The current position, when both halves are selected — the jump starts
        // from the front otherwise. The roster keys workspaces by **path**, so
        // the focused name resolves here, at the door that needs it.
        let here = self.focused_workspace().map(|p| ws_key(&p));
        let agent = self.focus.agent.clone();
        let at = here.as_deref().zip(agent.as_deref());
        if let Some(k) = attention::next_attention(&roster, at) {
            self.focus_agent(&PathBuf::from(k.ws), &k.agent_id);
        }
    }

    /// Toggle the pin on the workspace `name` addresses (§4.1 `pinned`, user
    /// order): appended when unpinned, removed when pinned. Durable, converging
    /// via `ui.json`. A name the enumeration does not answer pins nothing.
    ///
    /// **The pin toggle is a door** (bl-7407): the tab bar hands back a §3.1
    /// name, and `ui.json` keys pins by **path** — durable state whose
    /// re-keying is its own migration — so the resolution stands here, once, at
    /// the click, rather than on every painted tab.
    pub fn toggle_pin(&mut self, name: &str) {
        let Some(key) = self.workspace_path(name).map(|p| ws_key(&p)) else {
            return;
        };
        let key = key.as_str();
        let mut pins = self.ui.pinned();
        match pins.iter().position(|k| k == key) {
            Some(i) => {
                pins.remove(i);
            }
            None => pins.push(key.to_string()),
        }
        self.ui.set_pinned(pins);
    }

    /// Set a collapse override for a roster section `key` (§4.1 `collapsed`).
    pub fn set_collapsed(&mut self, key: &str, collapsed: bool) {
        self.ui.set_collapsed(key, collapsed);
    }

    /// Whether `(kind, ws, agent, oid)` is acknowledged in `ui.json` (§6) — the
    /// seen-watermark query, exposed for the convergence proof.
    pub fn is_seen(&self, kind: SeenKind, ws: &str, agent: &str, oid: &str) -> bool {
        self.ui.is_seen(kind, ws, agent, oid)
    }

    /// Startup focus (§4.1): the `--workspace` override if given, else the first
    /// attention-bearing workspace in derived (path) order, else the first.
    ///
    /// `over` is the roster to derive across rather than `self.snap.workspaces`
    /// verbatim, because the §3.6 unmaking re-derives focus *before* the worker
    /// has re-enumerated: the dead workspace is still in the snapshot and would
    /// win its own attention. One rule, told which roster it applies to — not a
    /// deletion special case.
    pub(super) fn startup_focus(
        &self,
        initial: Option<std::path::PathBuf>,
        over: &[crate::binding::Workspace],
    ) -> Focus {
        if let Some(ws) = initial {
            // `--workspace` may be spelled either way and means one thing: the
            // §3.1 name is the leaf, and the leaf of a bare name is itself.
            return Focus {
                ws: Some(crate::naming::leaf(&ws)),
                ..Focus::default()
            };
        }
        // Ranked by **path** and answered as a name (§4.1's "derived (path)
        // order" is unchanged; only the answer's spelling moved).
        let mut ranked: Vec<(String, String)> = over
            .iter()
            .map(|w| (ws_key(&w.path), crate::naming::leaf(&w.path)))
            .collect();
        ranked.sort();
        let roster: Vec<String> = ranked.into_iter().map(|(_, name)| name).collect();
        let mut attention: Vec<String> = Vec::new();
        for w in over {
            if self.workspace_stats(&w.path).0 > 0 {
                attention.push(crate::naming::leaf(&w.path));
            }
        }
        let roster_refs: Vec<&str> = roster.iter().map(String::as_str).collect();
        let attention_refs: Vec<&str> = attention.iter().map(String::as_str).collect();
        Focus {
            ws: crate::ui_state::derive_startup_focus(&roster_refs, &attention_refs),
            ..Focus::default()
        }
    }
}
