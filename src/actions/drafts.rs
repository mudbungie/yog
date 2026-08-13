//! Composer drafts, **keyed by the target they were typed for** (§11, §5.3,
//! bl-a69a).
//!
//! One text box that re-labels its verb with the selection is not one draft: a
//! goal typed for a new conversation and a message typed to an agent are
//! different things said to different addressees, and a single buffer carried
//! the first into the second's box — where Enter would have deposited a fresh
//! start's text as a message to an unrelated agent. Every chat app answers this
//! the same way, and so does yog: a draft belongs to its target, switching the
//! selection shows that target's own, and switching back restores what was
//! there.
//!
//! Still RAM until sent (§5.3) — this is the *key*, not persistence. Nothing
//! here reaches disk, and the map dies with the process like the single buffer
//! it replaces. It is the same shape [`StartState::new_ball`] already uses for
//! the ball form's per-project drafts: the context is the key.
//!
//! **Absence is emptiness.** A key with no entry reads as `""` and writing `""`
//! removes the entry, so "no draft" has one representation rather than two, and
//! a send clears exactly its own key by writing the empty string.

use std::collections::HashMap;
use std::path::PathBuf;

/// Which composer a draft belongs to — the target *and* the verb, which is one
/// fact: the composer's verb follows its target (§11).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DraftKey {
    /// A new conversation in a workspace (§3.4). `None` is the empty world's
    /// bootstrap box (§3.4 S0), which has no workspace yet — the general path
    /// with the workspace absent, not a third case.
    NewConversation(Option<PathBuf>),
    /// A message to the selected agent, by id (§2.3: the id is the address).
    Message(String),
}

impl DraftKey {
    /// The composer's key for what it is currently pointed at: a selected agent
    /// is a message, no selection is a new conversation in the focused
    /// workspace. The one derivation both composers ([`crate::shell`]'s docked
    /// bar and the empty world's bootstrap box) read, so the two surfaces cannot
    /// disagree about which draft is theirs.
    pub fn composer(workspace: Option<PathBuf>, selected: Option<String>) -> Self {
        match selected {
            Some(agent) => Self::Message(agent),
            None => Self::NewConversation(workspace),
        }
    }
}

/// Every in-flight composer draft, one per target (RAM, §5.3).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Drafts {
    by_target: HashMap<DraftKey, String>,
}

impl Drafts {
    /// `key`'s draft — the empty string when nothing has been typed for it.
    /// Owned, so the frame edits a local buffer and writes it back through
    /// [`set`](Self::set) rather than holding a borrow across a paint.
    pub fn text(&self, key: &DraftKey) -> String {
        self.by_target.get(key).cloned().unwrap_or_default()
    }

    /// Write `key`'s draft. Empty text removes the entry — a cleared draft and a
    /// never-typed one are the same thing, and this is also how a clean send
    /// clears **only** its own target (§5.3).
    pub fn set(&mut self, key: DraftKey, text: String) {
        if text.is_empty() {
            self.by_target.remove(&key);
        } else {
            self.by_target.insert(key, text);
        }
    }

    /// Whether no target holds a draft at all.
    pub fn is_empty(&self) -> bool {
        self.by_target.is_empty()
    }
}

#[cfg(test)]
mod tests;
