//! Prompt recall (DESIGN §11 inbox-composer, bl-f908): ↑ at the box's top row
//! steps back through **what you already said in this conversation**, ↓ at its
//! bottom row steps forward, and forward past the newest hands back the draft
//! you were typing. The gesture codex and Claude Code bind, spelled in yog's
//! own terms.
//!
//! **The history is derived, never stored** (PRINCIPLES: single source of
//! truth). It is the target's own operator messages, read from the two seats
//! the composer already reads — the snapshot's pending deposits (§5.1 #11, the
//! rows painted above the box) and the delivered transcript (§5.1 #12) — each
//! filtered through the one role derivation ([`crate::theme::message_role`]),
//! never a second reading of the bytes. There is no session log and no
//! `ui.json` key: a restart pages the same history, because the history *is*
//! the conversation. A new conversation has no chat and therefore no prompts —
//! the general path at zero items, not a special case.
//!
//! **The caret gate is the whole rule, and it holds on entry and continuation
//! alike**, so there is no browse mode and no mode flag. A recall parks the
//! caret at the end of what it recalled, so a one-row prompt sits on the top
//! row *and* the bottom row (one key per step, as in codex) while a multi-row
//! prompt walks the caret up through itself first and pages only once the
//! caret is at the top — readline's own behaviour, and the reason the arrows
//! stay usable *inside* a recalled prompt.
//!
//! **Leaving the recall is a derivation too, not an event** ([`Recall::settle`]):
//! when the draft is no longer the entry we put there, the operator has edited
//! it. That one check covers every exit at once — typing over a recalled prompt
//! (it becomes the draft, and the stashed one is rightly forgotten), sending
//! (the draft clears), switching targets (a different draft), and the history
//! shrinking under a landed send — so no call site resets anything.

use crate::inboxview::InboxEntry;
use crate::theme::{Role, message_role};
use crate::transcript::{EntryKind, Transcript};

/// Which way a press steps through the recalled prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// ↑ — one prompt further back.
    Back,
    /// ↓ — one prompt forward, and past the newest back to the draft.
    Forward,
}

/// Where the caret sat when the box last painted: its **visual** row in the
/// galley and how many rows that galley had. Visual, not logical, so a long
/// wrapped draft does not swallow the gesture. A galley exists only at paint
/// time, so the gesture reads the previous frame's answer — one key event per
/// frame, so it cannot be stale.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Caret {
    pub row: usize,
    pub rows: usize,
}

impl Caret {
    /// Is this direction the box's to give up? ↑ only from the top row, ↓ only
    /// from the bottom one; anywhere else the key is the caret's, as always.
    /// An empty box has no rows and is both at once.
    pub fn open(&self, dir: Step) -> bool {
        match dir {
            Step::Back => self.row == 0,
            Step::Forward => self.row + 1 >= self.rows,
        }
    }
}

/// Everything the operator has said to this target, newest first: the pending
/// deposits (not yet delivered) ahead of the delivered transcript's messages,
/// each in reverse order — which is chronological, since pending is by
/// construction newer than delivered.
pub fn prompts(pending: &[InboxEntry], transcript: &Transcript) -> Vec<String> {
    let mut said: Vec<String> = transcript
        .entries
        .iter()
        .filter_map(|entry| match &entry.kind {
            EntryKind::Delivered {
                sender,
                epitaph,
                body,
            } => mine(sender, epitaph.is_some()).then(|| body.clone()),
            _ => None,
        })
        .collect();
    said.extend(pending.iter().filter_map(|entry| {
        let deposit = &entry.deposit;
        mine(
            deposit.sender.as_deref().unwrap_or_default(),
            deposit.epitaph.is_some(),
        )
        .then(|| deposit.body.clone())
    }));
    said.reverse();
    said
}

/// The operator's own, by the one §11 role derivation — a peer's message and a
/// child's result deposit are things said *to* you, and are not offered back.
fn mine(sender: &str, has_epitaph: bool) -> bool {
    message_role(sender, has_epitaph) == Role::User
}

/// How far back the box is currently showing, and the draft it displaced
/// (§5.3 RAM: a draft you have not sent yet, one step further from being
/// sent). Depth 0 is the live draft; depth *n* is the *n*th-newest prompt.
#[derive(Debug, Default)]
pub struct Recall {
    stash: Option<String>,
    depth: usize,
}

impl Recall {
    /// Leave the recall the moment the draft stops being the entry we put
    /// there — the one exit, covering every way out (see the module doc).
    pub fn settle(&mut self, draft: &str, prompts: &[String]) {
        let shown = self.depth.checked_sub(1).and_then(|at| prompts.get(at));
        if self.depth > 0 && shown.map(String::as_str) != Some(draft) {
            *self = Self::default();
        }
    }

    /// Step one prompt in `dir`, returning what the box should show. `None`
    /// leaves the key to the widget — the caret is not at that edge, or there
    /// is nothing that way — which is exactly today's behaviour.
    pub fn step(
        &mut self,
        dir: Step,
        caret: Caret,
        draft: &str,
        prompts: &[String],
    ) -> Option<String> {
        if !caret.open(dir) {
            return None;
        }
        match dir {
            Step::Back => {
                let further = prompts.get(self.depth)?.clone();
                if self.depth == 0 {
                    self.stash = Some(draft.to_owned());
                }
                self.depth += 1;
                Some(further)
            }
            Step::Forward => {
                self.depth = self.depth.checked_sub(1)?;
                Some(match self.depth.checked_sub(1) {
                    Some(at) => prompts.get(at).cloned().unwrap_or_default(),
                    None => self.stash.take().unwrap_or_default(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests;
