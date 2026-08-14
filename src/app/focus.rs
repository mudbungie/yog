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
//! name and spent by [`adopt_started`](AppModel::adopt_started).

use super::echo::{Echo, Target};
use super::{AppModel, Focus};
use crate::attention;
use crate::boundary::answer::queue;
use crate::keymap::InspectorTab;
use crate::nav::ws_key;
use crate::ui_state::SeenKind;
use std::path::{Path, PathBuf};

impl AppModel {
    /// Focus a workspace (center-panel target) without selecting an agent — no
    /// acknowledgement (§6: focus records seen only for a *selected agent*). The
    /// selected inspector tab is sticky across the move (§11).
    pub fn focus_workspace(&mut self, ws: &Path) {
        self.focus = Focus {
            ws: Some(ws.to_path_buf()),
            agent: None,
            tab: self.focus.tab,
        };
    }

    /// Focus an agent — the **acknowledgement** (§6): select it in the inspector
    /// and record its current evidence oids as seen, converging the ack across
    /// instances (§13.1). The selected inspector tab is sticky (§11). The ack
    /// does not end here: [`ack_focused`](Self::ack_focused) keeps it stamped
    /// for as long as the agent stays focused.
    pub fn focus_agent(&mut self, ws: &Path, agent_id: &str) {
        self.focus = Focus {
            ws: Some(ws.to_path_buf()),
            agent: Some(agent_id.to_string()),
            tab: self.focus.tab,
        };
        self.record_seen(ws, agent_id);
    }

    /// Claim the conversation a fired start just minted (§3.4): **a start
    /// focuses what it started**, and what it started is a conversation, not
    /// merely the workspace it resolved. The claim is held by the minted
    /// `conversation` name because that is all the fire knows — the root has no
    /// agent id until the detached driver writes `agents/<id>` — and the name is
    /// the lernie-stored fact (`--name` at fire, §3.3), which the derivation
    /// reads back off every root as its `name_fact`.
    /// Spent by [`adopt_started`](Self::adopt_started); RAM only (§13.1),
    /// as the focus it becomes is.
    ///
    /// **It carries `goal` with it** (§7.2, bl-915e): a handle that painted no
    /// row left the operator's text with no representation anywhere in yog
    /// until the driver wrote it, which is what read as the UI waiting for the
    /// send. The claim and the echo are one value with one lifetime.
    pub(crate) fn await_conversation(&mut self, ws: &Path, conversation: &str, goal: &str) {
        self.started = Some(Echo::started(ws, conversation, goal, self.now_unix()));
    }

    /// Hold the echo a §8.2 `message` leaves (§7.2): the same mechanism one
    /// door over — the deposit is piped and its `NNN-user.md` only appears on
    /// the driver's next step boundary, so the identical gap was open there.
    /// No focus claim rides on it: the operator was already looking at this
    /// conversation, and their own message landing must not yank them back from
    /// wherever they have since navigated.
    pub(crate) fn await_message(&mut self, ws: &Path, agent: &str, content: &str) {
        self.started = Some(Echo::messaged(
            &self.derived,
            ws,
            agent,
            content,
            self.now_unix(),
        ));
    }

    /// Carry the one pending value forward against the derivation (§3.4, §7.2)
    /// — two moves on one `Echo`, never on two concepts:
    ///
    /// 1. **The claim resolves.** The roster carries the root wearing the
    ///    minted §3.3 name, so the conversation has an id: focus is spent
    ///    through [`focus_agent`](Self::focus_agent), the same path the ↓ key
    ///    lands through (so the arrival acknowledges exactly as any other
    ///    selection does, §6), and the echo takes that id. The claim is spent
    ///    once and never again — a resolved target has no name left to match —
    ///    so the operator's own later selection stands.
    /// 2. **The echo retires** when the derivation shows the message it stood
    ///    in for. Nothing claimed, or nothing landed, holds it: the general
    ///    path with the file absent, not a wait state — and there is no timeout
    ///    beside it, because a faded row (§11) is not a claim that can go stale
    ///    (the faded-send ruling; §7.2 spells the whole expiry).
    pub(super) fn adopt_started(&mut self) {
        let Some(mut echo) = self.started.take() else {
            return;
        };
        if let Some(agent) = echo.resolved(&self.derived) {
            echo.target = Target::Agent(agent.clone());
            self.focus_agent(&echo.ws, &agent);
        }
        if echo.landed(&self.derived) {
            return;
        }
        self.started = Some(echo);
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
        if let (Some(ws), Some(agent)) = (self.focus.ws.clone(), self.focus.agent.clone()) {
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
        if let Some(k) = attention::next_attention(&roster, self.focus_pos()) {
            self.focus_agent(&PathBuf::from(k.ws), &k.agent_id);
        }
    }

    /// The current focus as a `(ws, agent)` position (both a workspace and an
    /// agent selected), else `None` — the jump then starts from the front.
    fn focus_pos(&self) -> Option<(&str, &str)> {
        let ws = self.focus.ws.as_deref()?.to_str()?;
        let agent = self.focus.agent.as_deref()?;
        Some((ws, agent))
    }

    /// Toggle `key`'s pin (§4.1 `pinned`, user order): appended when unpinned,
    /// removed when pinned. Durable, converging via `ui.json`.
    pub fn toggle_pin(&mut self, key: &str) {
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
            return Focus {
                ws: Some(ws),
                ..Focus::default()
            };
        }
        let mut roster: Vec<String> = over.iter().map(|w| ws_key(&w.path)).collect();
        roster.sort();
        let mut attention: Vec<String> = Vec::new();
        for w in over {
            if self.workspace_stats(&w.path).0 > 0 {
                attention.push(ws_key(&w.path));
            }
        }
        let roster_refs: Vec<&str> = roster.iter().map(String::as_str).collect();
        let attention_refs: Vec<&str> = attention.iter().map(String::as_str).collect();
        Focus {
            ws: crate::ui_state::derive_startup_focus(&roster_refs, &attention_refs)
                .map(std::path::PathBuf::from),
            ..Focus::default()
        }
    }
}
