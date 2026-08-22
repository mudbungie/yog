//! **What a pending conversation looks like to a seat** (§3.4, §7.2, §5.1 #11)
//! — the deposits an echo stands for and the synthetic agent that carries them,
//! cut off [`super`] at §12's per-file budget on the seam that module's own doc
//! draws: what an echo *is* and when it retires lives there; this is the shape
//! it wears on the glass.
//!
//! Both facts here are **queries, never flags** (§7.2's faded-send ruling): a
//! deposit with no file has no name, and a conversation with no branch has no
//! tip — so the seats read pending-ness off the value itself and no state says
//! it twice.

use super::Echo;
use crate::git_tree::{Agent, AgentState};
use crate::inboxview::{Deposit, InboxEntry};

/// The deposit sender an echo speaks for — the operator, exactly as a real
/// `user` deposit's frontmatter says it, so a pending row reads identically
/// whether yog or the substrate put it there.
const SENDER: &str = "user";

impl Echo {
    /// This echo's own send as the pending deposit it is (§5.1 #11) — the same
    /// shape a real `inbox/<id>/*.md` parses to, so every seat that renders
    /// pending mail renders this identically.
    pub(crate) fn deposit(&self) -> InboxEntry {
        deposit_of(&self.text, self.at_unix)
    }

    /// **Every send this echo stands for, oldest first** (§7.2, bl-56c6): the
    /// one that made it, then each §3.4 held follow-up in the order the
    /// operator said it. A resolved echo holds none, so this is
    /// [`deposit`](Self::deposit) alone — the general path at zero held items
    /// rather than a case of its own.
    pub(crate) fn deposits(&self) -> Vec<InboxEntry> {
        std::iter::once(self.deposit())
            .chain(
                self.held
                    .iter()
                    .map(|send| deposit_of(&send.text, send.at_unix)),
            )
            .collect()
    }

    /// The **pending conversation** a start's echo mints (§3.4): an agent
    /// addressed by `id` and called `name` — the two are the same minted §3.3
    /// name until the branch exists ([`pending_identity`](Self::pending_identity))
    /// — carrying the operator's goal as its preview and every send still held
    /// as its queue, so the §11 list paints one row in their own words and the
    /// composer's queue paints what was said into it.
    ///
    /// Its **tip oid is empty**, and that is what the seats read to paint it
    /// faded ([`Agent::in_memory`]): a derived agent comes off `for-each-ref`,
    /// so it always has a tip, and this one has no branch at all.
    ///
    /// **Its state is what `git_tree::state::classify` would answer about it**
    /// (bl-56c6): no lock observed and no completed step, flagged uncertain
    /// because nothing was probed at all — there is no inbox directory to hold a
    /// lock and no step to frame. It read `Live` until that ball, which claimed
    /// a driver held an executor yog had never looked at, and offered §8.2's
    /// `Stop` on a conversation no signal could reach — a control that fires and
    /// does nothing, silently.
    pub(crate) fn pending_conversation(&self, id: &str, name: &str) -> Agent {
        Agent {
            branch_name: format!("agents/{name}"),
            agent_id: id.to_owned(),
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
            state: AgentState::Stopped,
            state_uncertain: true,
            pending: self.deposits(),
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

/// One text as the deposit it already is. Its `name` is **empty**, and that is
/// the whole of what makes it read as pending rather than settled
/// ([`InboxEntry::in_memory`]): a deposit's name is its file, and this one has
/// no file. The seats paint it faded off that one fact and brighten when the
/// derivation replaces it (§11, the faded-send ruling).
fn deposit_of(text: &str, at_unix: i64) -> InboxEntry {
    InboxEntry {
        name: String::new(),
        raw: text.as_bytes().to_vec(),
        deposit: Deposit {
            sender: Some(SENDER.to_owned()),
            deposited_at: Some(crate::ui_state::iso8601_extended(at_unix)),
            body: text.to_owned(),
            ..Deposit::default()
        },
    }
}
