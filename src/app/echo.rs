//! The **pending echo** (DESIGN §7.2, §3.4, bl-915e): the operator's own last
//! send, held until the derivation shows it.
//!
//! A [`Snapshot`] is what a completed derivation read off disk, and that was
//! the only source a frame had — so between Enter and the detached driver's
//! first write, the text the operator had just typed existed nowhere in yog's
//! model. Operator: *"you send the message, but before it goes into the inbox,
//! it's just missing for a minute."* Nothing was blocked; there was nothing to
//! render.
//!
//! This is not a synchronous write and not a spinner — the frame still does no
//! IO and still renders a completed derivation. It is an optimistic echo,
//! reconciled by the next snapshot, and it is the **same value** as the §3.4
//! start claim rather than a second pending concept beside it: one thing names
//! the conversation, holds the text, and is retired by one predicate.
//!
//! [`compose`] is the one place snapshot and pending meet. What it writes is
//! the fact an unflushed message already is — a pending deposit (§5.1 #11) —
//! so the `✉n` badge, the Inbox tab and the §11 inbox-composer queue carry it
//! with no new seat. A start has no agent to hang it on, so the fold mints a
//! **pending conversation** keyed by the minted §3.3 name: one row in the §11
//! list, in the operator's own words.

use super::Snapshot;
use crate::git_tree::{Agent, AgentState};
use crate::inboxview::{Deposit, InboxEntry};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The deposit sender an echo speaks for — the operator, exactly as a real
/// `user` deposit's frontmatter says it, so a pending row reads identically
/// whether yog or the substrate put it there.
const SENDER: &str = "user";

/// Who an echo addresses. The two arms are the two things that can be true when
/// a message is sent, and the difference between them is real: a start has no
/// agent id yet (only the §3.3 name it minted), and a start focuses what it
/// started (§3.4) while a follow-up must not — the operator was already there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    /// A §3.4 start: the minted §3.3 name, the only identity the conversation
    /// has until the detached driver writes its branch.
    Conversation(String),
    /// A §8.2 follow-up: the agent id already on the roster.
    Agent(String),
}

/// One message yog has sent that the derivation has not shown yet — the §3.4
/// start claim, carrying the operator's text (§7.2). Per-instance RAM (§5.3,
/// §13.1); nothing about it is written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Echo {
    pub(crate) ws: PathBuf,
    pub(crate) target: Target,
    /// The operator's text, verbatim — the payload the composer sent.
    pub(crate) text: String,
    /// How many `messages/` entries the target held when this was made — the
    /// reconciliation baseline (§5.1 #12). Zero for a start, whose root does
    /// not exist.
    pub(crate) baseline: usize,
    /// Wall-clock seconds at the send: the deposit header's `at`, and the
    /// recency that lifts the row.
    pub(crate) at_unix: i64,
}

impl Echo {
    /// The echo a fired §3.4 start leaves: the minted name, the goal verbatim,
    /// and a zero baseline — the root does not exist, so any landed message
    /// under that name is the one this stands in for.
    pub(crate) fn started(ws: &Path, conversation: &str, goal: &str, at_unix: i64) -> Self {
        Self {
            ws: ws.to_path_buf(),
            target: Target::Conversation(conversation.to_owned()),
            text: goal.to_owned(),
            baseline: 0,
            at_unix,
        }
    }

    /// The echo a §8.2 `message` leaves: the agent it was aimed at and the
    /// count of messages already landed there, read off the derivation the
    /// gesture was fired against.
    pub(crate) fn messaged(
        snap: &Snapshot,
        ws: &Path,
        agent: &str,
        content: &str,
        at_unix: i64,
    ) -> Self {
        let target = Target::Agent(agent.to_owned());
        let baseline = index_of(snap, ws, &target)
            .and_then(|i| snap.trees.get(ws)?.agents.get(i))
            .map_or(0, |a| a.messages);
        Self {
            ws: ws.to_path_buf(),
            target,
            text: content.to_owned(),
            baseline,
            at_unix,
        }
    }

    /// The agent id a *start*'s target has acquired, once the roster carries
    /// the root wearing its minted §3.3 name — the §3.4 claim resolving. The
    /// echo then **takes that id** ([`Target::Agent`]): a conversation getting
    /// its id is what actually happened in the world, so it is one value
    /// changing, not a second one starting. `None` for an already-resolved
    /// target and for a root not written yet, which is the general path with
    /// the branch absent rather than a wait state.
    pub(crate) fn resolved(&self, derived: &Snapshot) -> Option<String> {
        let Target::Conversation(_) = &self.target else {
            return None;
        };
        let i = index_of(derived, &self.ws, &self.target)?;
        Some(derived.trees.get(&self.ws)?.agents.get(i)?.agent_id.clone())
    }

    /// Whether `derived` now shows the message this echo stands in for — the
    /// one reconciliation predicate (§7.2): the target is on the roster and its
    /// landed `messages/` count has passed the baseline the echo recorded.
    /// False holds the echo.
    pub(crate) fn landed(&self, derived: &Snapshot) -> bool {
        index_of(derived, &self.ws, &self.target)
            .and_then(|i| derived.trees.get(&self.ws)?.agents.get(i))
            .is_some_and(|a| a.messages > self.baseline)
    }

    /// This echo as the pending deposit it is (§5.1 #11) — the same shape a
    /// real `inbox/<id>/*.md` parses to, so every seat that renders pending
    /// mail renders this identically.
    ///
    /// Its `name` is **empty**, and that is the whole of what makes it read as
    /// pending rather than settled ([`InboxEntry::in_memory`]): a deposit's
    /// name is its file, and this one has no file. The seats paint it faded off
    /// that one fact and brighten when the derivation replaces it (§11, the
    /// faded-send ruling).
    fn deposit(&self) -> InboxEntry {
        InboxEntry {
            name: String::new(),
            raw: self.text.clone().into_bytes(),
            deposit: Deposit {
                sender: Some(SENDER.to_owned()),
                deposited_at: Some(crate::ui_state::iso8601_extended(self.at_unix)),
                body: self.text.clone(),
                ..Deposit::default()
            },
        }
    }

    /// The **pending conversation** a start's echo mints (§3.4): an agent keyed
    /// by the minted §3.3 name — the only identity a start has before its
    /// branch — carrying the operator's goal as its preview, so the §11 list
    /// paints one row in their own words. Live, because a driver is starting;
    /// every other field is the empty one, which is what a conversation with no
    /// commits, no steps and no marks honestly has.
    ///
    /// Its **tip oid is empty**, and that is what the seats read to paint it
    /// faded ([`Agent::in_memory`]): a derived agent comes off `for-each-ref`,
    /// so it always has a tip, and this one has no branch at all.
    fn pending_conversation(&self, name: &str) -> Agent {
        Agent {
            branch_name: format!("agents/{name}"),
            agent_id: name.to_owned(),
            tip_oid: String::new(),
            tip_short_oid: String::new(),
            tip_timestamp_unix: self.at_unix,
            call_start_unix: None,
            last_action_unix: self.at_unix,
            messages: 0,
            steps: Vec::new(),
            preview: Some(self.text.clone()),
            stream: crate::git_tree::Stream::default(),
            tool_calls: Vec::new(),
            state: AgentState::Live,
            state_uncertain: false,
            pending: vec![self.deposit()],
            conflicted_oid: None,
            budget_oid: None,
            abandoned_oid: None,
            notify_oid: None,
            held: None,
            goal_ball: None,
            name: Some(name.to_owned()),
            goal_name: None,
        }
    }
}

/// The index of an echo's target in a workspace's derived agent list: by
/// `name_fact` for a start (the name is the identity until the branch exists),
/// by id for a follow-up. An index rather than a reference so no signature
/// grows a named lifetime (AGENTS.md rule 1).
fn index_of(snap: &Snapshot, ws: &Path, target: &Target) -> Option<usize> {
    snap.trees
        .get(ws)?
        .agents
        .iter()
        .position(|a| match target {
            Target::Conversation(name) => a.name_fact().as_deref() == Some(name.as_str()),
            Target::Agent(id) => &a.agent_id == id,
        })
}

/// **The one place the derivation and the non-derived facts meet** (§7.2): the
/// snapshot a frame paints is the worker's, with the pending `echo` and the
/// focused conversation's live `tail` folded in. Every render seat reads the
/// result and none of them knows either exists.
///
/// The two are folded here rather than each somewhere convenient because that
/// is the whole partition: **one function writes the painted snapshot**, so
/// "what does a frame see that disk does not say?" has one answer to read, and
/// a third such fact is a third argument here rather than a third mechanism.
///
/// With nothing pending and nothing streaming this is a pointer clone, so the
/// ordinary case allocates nothing and the rendered `Arc` is the derived one —
/// which is also why the caller may only run this when one of its inputs moved:
/// a fresh `Arc` every frame would make `SnapMemo` rebuild per frame, the exact
/// cost bl-e90a removed.
pub(crate) fn compose(
    derived: &Arc<Snapshot>,
    echo: Option<&Echo>,
    tail: Option<&super::live::LiveTail>,
) -> Arc<Snapshot> {
    if echo.is_none() && tail.is_none() {
        return Arc::clone(derived);
    }
    let mut snap = (**derived).clone();
    if let Some(tail) = tail {
        super::live::overlay(&mut snap, tail);
    }
    let Some(echo) = echo else {
        return Arc::new(snap);
    };
    let tree = snap.trees.entry(echo.ws.clone()).or_default();
    match index_of(derived, &echo.ws, &echo.target) {
        // The target is on the roster: the echo is one more undelivered
        // deposit on it, and the send is an action, so the row rises.
        Some(i) => {
            if let Some(agent) = tree.agents.get_mut(i) {
                agent.pending.push(echo.deposit());
                agent.last_action_unix = agent.last_action_unix.max(echo.at_unix);
            }
        }
        // It is not: a start whose branch does not exist yet, which is the
        // whole of what the operator could not see.
        None => {
            if let Target::Conversation(name) = &echo.target {
                tree.agents.push(echo.pending_conversation(name));
            }
        }
    }
    Arc::new(snap)
}

/// **The echo at the row altitude** (REMOTE §9.7, bl-44e9) — the same fold, over
/// an answered §11 list instead of over a snapshot, because that surface reads a
/// `Reply` now. Its own file at §12's cap; the reasoning is its own doc.
pub(crate) mod rows;

impl super::AppModel {
    /// The §11 list **as this seat paints it**: what the boundary answered for
    /// the focused workspace, with this window's own pending echo folded on
    /// (§3.4, §7.2). Orchestration, never derivation (§8.5's paint-side line):
    /// the rows are the engine's and the optimism is the seat's.
    ///
    /// An unfocused window echoes nothing, because an echo belongs to the
    /// workspace it was fired in and there is none to compare against.
    pub fn echoed(
        &self,
        rows: Vec<crate::nav::convs::ConvRow>,
        now_unix: i64,
    ) -> Vec<crate::nav::convs::ConvRow> {
        let Some(ws) = self.focus.ws.as_deref() else {
            return rows;
        };
        rows::with_echo(self.started.as_ref(), ws, rows, now_unix)
    }
}

#[cfg(test)]
mod tests;
