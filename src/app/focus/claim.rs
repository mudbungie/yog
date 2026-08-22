//! **The §3.4 start claim and the §7.2 echo it carries** — the one focus no
//! gesture can set outright, cut off [`super`] at §12's per-file budget on the
//! seam that module's own doc already draws (bl-78d8).
//!
//! Everything here is *one value with one lifetime*: a fire mints an [`Echo`],
//! the seats paint it, and [`adopt_started`](crate::AppModel::adopt_started)
//! carries it forward against each derivation until the world shows what it
//! stood in for. The selection surface next door is about rows that exist; this
//! is about the one that does not yet.

use super::super::echo::{Echo, Target};
use crate::AppModel;
use std::path::Path;

impl AppModel {
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
    ///
    /// **And it focuses that name at once** (bl-2e8f, DESIGN §3.4): the echo
    /// mints a row keyed by this very name and the seat folds it into the
    /// answered forest ahead of every reader, so the name is a selection like
    /// any other — the ordinary [`focus_agent`](Self::focus_agent), the same
    /// path ↓ takes onto that row by hand and the same one the spend takes a
    /// write later. Focusing only on the spend left the operator's own new
    /// conversation the one row nothing highlighted, behind the birth
    /// placeholder, for as long as the driver took. Nothing durable is written
    /// for a name: §6 records the evidence an agent *has*, and a conversation
    /// with no branch has none.
    pub(crate) fn await_conversation(&mut self, ws: &Path, conversation: &str, goal: &str) {
        self.started = Some(Echo::started(ws, conversation, goal, self.now_unix()));
        self.focus_agent(ws, conversation);
    }

    /// Hold the echo a §8.2 `message` leaves (§7.2): the same mechanism one
    /// door over — the deposit is piped and its `NNN-user.md` only appears on
    /// the driver's next step boundary, so the identical gap was open there.
    /// No focus claim rides on it: the operator was already looking at this
    /// conversation, and their own message landing must not yank them back from
    /// wherever they have since navigated.
    ///
    /// `queued` is the §11 queue seat's own baseline (§7.2, bl-78d8): how many
    /// deposits that seat could show when the act was **queued**. It is the
    /// caller's to supply and cannot be re-derived here — by the time this runs
    /// the piped verb has already written the file, so any count read now would
    /// include the very deposit the echo stands in for and retire it at birth.
    pub(crate) fn await_message(&mut self, ws: &Path, agent: &str, content: &str, queued: usize) {
        self.started = Some(Echo::messaged(
            &self.derived,
            ws,
            agent,
            content,
            queued,
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
    pub(in crate::app) fn adopt_started(&mut self) {
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
}
