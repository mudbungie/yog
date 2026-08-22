//! **The sends yog holds while a conversation has no address** (DESIGN §3.4,
//! §7.2; bl-56c6) — the start window, dissolved rather than patched per seat.
//!
//! Between Enter and the detached driver's first write the conversation has
//! only its minted §3.3 name, and that name resolves nowhere: every
//! agent-addressed act refuses at `boundary::address::agent`. So a second
//! message typed into the composer during the window was posted at an
//! unresolvable name and bounced back *"unknown conversation"* for the whole
//! window — while DESIGN §3.4's own ruling says the next Enter is **always the
//! second** message, never a second conversation.
//!
//! The reframe: **a send aimed at yog's own pending mint is held by yog and
//! posted when the start resolves.** Nothing addresses a name that resolves
//! nowhere, so the refusal has no site left to happen at, and the race where
//! such a send *did* land — overwriting the §3.4 claim with an echo that could
//! never retire — has no site either. It is not a queue beside the echo: a held
//! send is one more deposit on the same [`Echo`], painted in the same faded
//! §11 queue, released by the same event that spends the claim.

use super::{Echo, Target};
use crate::AppModel;
use crate::boundary::Action;
use std::path::Path;

/// One send made at a conversation that had no address yet: what was said, and
/// when — its `at`, exactly as a landed deposit's frontmatter carries one, so
/// the queue row it paints is the row it will keep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeldSend {
    pub(crate) text: String,
    pub(crate) at_unix: i64,
}

impl Echo {
    /// Whether `agent` names **this window's own unresolved mint** — a §3.4
    /// start whose branch does not exist, addressed by the name it minted.
    /// False the instant the claim resolves, which is what makes the hold below
    /// end without a second signal saying so.
    pub(crate) fn is_pending_mint(&self, agent: &str) -> bool {
        matches!(&self.target, Target::Conversation(name) if name == agent)
    }
}

impl AppModel {
    /// **Hold a send aimed at the conversation this window is still starting**
    /// (§3.4). `true` when it took the text — the caller then empties its draft
    /// exactly as a clean deposit would, because from the operator's side the
    /// words have left the box and joined the queue.
    ///
    /// `false` is every other send, which is nearly all of them: a target that
    /// is not this window's own pending mint is posted at the boundary in the
    /// ordinary way. The predicate is the echo's, so there is no flag to keep in
    /// step with it.
    pub(crate) fn hold_send(&mut self, ws: &Path, agent: &str, text: &str) -> bool {
        let at_unix = self.now_unix();
        let Some(echo) = self
            .started
            .as_mut()
            .filter(|echo| echo.ws == ws && echo.is_pending_mint(agent))
        else {
            return false;
        };
        echo.held.push(HeldSend {
            text: text.to_owned(),
            at_unix,
        });
        true
    }

    /// **Post everything held, in the order it was said** (§3.4) — the moment
    /// the claim resolves and the conversation has an id an act can address.
    ///
    /// Fired without a receipt ([`crate::shell::act`]'s first shape) because
    /// there is nothing left for one to gate: the draft these came out of was
    /// emptied when they were held, and each is already in the §11 queue as the
    /// undelivered deposit it is. What a refusal earns is the durable
    /// `ops.jsonl` line every act leaves (INV-2), read back by the §7.3 banner.
    ///
    /// Nothing held is nothing posted, which is every resolution but the ones
    /// the operator typed into.
    pub(in crate::app) fn release_held(&mut self, agent: &str) {
        let Some(echo) = self.started.as_mut() else {
            return;
        };
        let held = std::mem::take(&mut echo.held);
        let ws = echo.ws.clone();
        for send in held {
            let workspace = self.derived.ws_name(&ws);
            self.post_act(&Action::Message {
                workspace,
                agent: agent.to_owned(),
                content: send.text,
            });
        }
    }

    /// **The name→id swap a resolved §3.4 claim made**: `(minted name, agent
    /// id)` while the echo that made it is still alive, `None` otherwise.
    ///
    /// The composer's draft buffer is keyed by the identity it was typed
    /// against (§11, bl-a69a), so half-typed text vanished the frame the
    /// conversation swapped one for the other (bl-56c6 D3). This is the fact
    /// that carries it over, read off the echo rather than announced by a
    /// one-shot: it is true for the echo's whole remaining life, so the seat
    /// that acts on it is idempotent and needs no event to have caught.
    pub(crate) fn adopted_names(&self) -> Option<(String, String)> {
        let echo = self.started.as_ref()?;
        let name = echo.born.clone()?;
        match &echo.target {
            Target::Agent(id) => Some((name, id.clone())),
            Target::Conversation(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
