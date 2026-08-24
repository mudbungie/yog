//! **One channel the window holds** (REMOTE §8.2, bl-aaec): the slice of the
//! roster it feeds, the one name mapping it spends, and the link the answers
//! land on.
//!
//! The window is a client of the engine in its own process — over loopback, on
//! the window leaf ([`Origin::Local`]) — **plus one channel per
//! [entry](super::entries)**, each on that entry's own material. §8.2: *"The
//! roster is the union: a workspace is a workspace, and which engine hosts it
//! is a fact painted on it, never a mode the window is in."* So a channel is
//! not a mode and not a server noun; it is where a row came from.
//!
//! **The mapping is spent here, in both directions** ([`origin`]) — the same
//! one place, and the same two directions, as
//! [`seat::channel`](super::seat)'s write path, which is why nothing above this
//! line reasons in anything but the leaf.
//!
//! **A refusal is one entry's, never the set's.** An entry whose material will
//! not read answers its own sentence here, exactly as it does at
//! [`seat`](super::seat): an entry that exists is the answer to its name even
//! when it cannot be dialled, and falling through to another channel would send
//! the question to the wrong engine on the strength of a missing file. The
//! whole-shell refusal stays reserved for the one wire the window cannot exist
//! without: its own (bl-dc14), which is why [`Origin::Local`] carries none.
//!
//! **A channel nobody answers is the ordinary empty state**, not a case: a
//! [`Link`] whose far end was never taken lands nothing, which is the same code
//! path — and the same paint — as an answer that has not arrived yet. Every
//! entry channel is in exactly that posture until bl-670c attaches an asker to
//! it; what the roster shows for one meanwhile is [`Channel::rows`]' claim row.

/// The origin and its mapping — the value half, split off at §12's band.
mod origin;
pub use origin::{Origin, RosterRow};

use super::entries::Entry;
use super::link::{Landed, Link};
use crate::binding::WorkspaceKind;
use crate::boundary::reply::{Reply, WsRow};
use crate::boundary::{Gesture, Query, codec};
use serde_json::Value;

/// One channel: where it is, why it cannot be reached when it cannot, and the
/// slice that has landed on it.
pub struct Channel {
    origin: Origin,
    /// Why this channel has no material, when it has none — an entry's own
    /// sentence, answered in place of every ask. `None` on a channel whose
    /// material read clean, and always `None` on [`Origin::Local`].
    refusal: Option<String>,
    link: Link,
}

impl Channel {
    /// The window's own engine, over the link the engine minted for it.
    pub fn local(link: Link) -> Self {
        Self {
            origin: Origin::Local,
            refusal: None,
            link,
        }
    }

    /// One entry as a channel, over `link`. The link is handed in rather than
    /// minted here for [`local`](Self::local)'s reason: whoever owns the far
    /// end owns the mint. Today every caller hands one **nobody answers**
    /// ([`Channels::compose`](super::channels::Channels::compose)) because no
    /// thread asks an entry yet — that is bl-670c's. Its refusal is the
    /// entry's own.
    pub fn entry(held: Entry, link: Link) -> Self {
        let Entry {
            leaf,
            workspace,
            channel,
        } = held;
        Self {
            origin: Origin::Entry {
                leaf,
                host: workspace,
            },
            refusal: channel.err(),
            link,
        }
    }

    /// Whether this channel is the answer to `name` — an entry is the answer to
    /// its leaf, and the local channel claims nothing, being where every name
    /// no entry holds goes (§8.2).
    pub(crate) fn claims(&self, name: &str) -> bool {
        self.origin.label().is_some_and(|leaf| leaf == name)
    }

    /// Declare `question` standing on this channel and read whatever landed —
    /// carrying the host's name out, and this box's spelling back.
    pub(crate) fn ask(&mut self, question: &Value) -> Option<Landed> {
        if let Some(said) = &self.refusal {
            return Some(Err(said.clone()));
        }
        let carried = self.carried(question);
        let landed = self.link.ask(&carried)?;
        Some(landed.map(|reply| self.labelled(reply)))
    }

    /// This channel's slice of the union roster (§8.2), each row labelled.
    ///
    /// **An entry that has landed nothing still holds its leaf.** The entry IS
    /// a workspace this box participates in, so it wears a row from the moment
    /// the operator provisions it, carrying the zeros it honestly has — the
    /// §3.4 raise claim's shape one noun over (`AppModel::raised_rows`), and
    /// the row bl-670c's asker fills and bl-f29c paints unreachable.
    pub(crate) fn rows(&mut self) -> Vec<RosterRow> {
        let landed = self.ask(&codec::encode(&Gesture::Ask(Query::Workspaces)));
        let answered = match landed {
            Some(Ok(Reply::Workspaces(view))) => view.rows,
            _ => Vec::new(),
        };
        let mut rows: Vec<RosterRow> = answered
            .into_iter()
            .map(|row| RosterRow {
                row,
                origin: self.origin.clone(),
            })
            .collect();
        if let Some(leaf) = self.origin.label()
            && !rows.iter().any(|r| r.row.workspace == leaf)
        {
            rows.push(RosterRow {
                row: claimed(&leaf),
                origin: self.origin.clone(),
            });
        }
        rows
    }

    /// This channel's own frame duty — [`Link::settle`], per channel.
    pub(crate) fn settle(&mut self) {
        self.link.settle();
    }

    /// Whether anything standing on this channel is still unanswered.
    #[cfg(test)]
    pub(crate) fn awaiting(&self) -> bool {
        self.link.awaiting()
    }

    /// `question` as the far side reads it. Undecodable, naming no workspace,
    /// and naming one this channel does not rename are one branch: nothing to
    /// rewrite, so the envelope crosses as it was written.
    fn carried(&self, question: &Value) -> Value {
        let rewritten = codec::decode(question).ok().and_then(|gesture| {
            let named = gesture.workspace()?;
            let host = self.origin.outbound(&named)?;
            Some(codec::encode(&gesture.with_workspace(&host)))
        });
        rewritten.unwrap_or_else(|| question.clone())
    }

    /// A landed reply in this box's spelling — the roster's rows renamed back
    /// to the leaf. It is the one reply that *identifies* a workspace and also
    /// crosses an entry channel today; the board's and the monitor's do neither
    /// until the fan-out rung gives them a channel to cross.
    fn labelled(&self, reply: Reply) -> Reply {
        match reply {
            Reply::Workspaces(mut view) => {
                for row in &mut view.rows {
                    if let Some(leaf) = self.origin.inbound(&row.workspace) {
                        row.workspace = leaf;
                    }
                }
                Reply::Workspaces(view)
            }
            other => other,
        }
    }
}

/// The row an entry wears before its channel answers: named, and zero
/// everywhere else. Not a placeholder — a workspace nothing has been read from
/// holds nothing, exactly as a newborn wall does.
fn claimed(leaf: &str) -> WsRow {
    WsRow {
        workspace: leaf.to_owned(),
        kind: WorkspaceKind::Named {
            name: leaf.to_owned(),
        },
        attention: 0,
        agents: 0,
        running: false,
        pinned: None,
        config_tip: None,
    }
}

#[cfg(test)]
mod tests;
