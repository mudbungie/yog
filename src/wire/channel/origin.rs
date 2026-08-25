//! **Which channel a workspace came from, and the one name mapping it spends**
//! (REMOTE §8.2, bl-aaec) — the value half of [`Channel`](super::Channel),
//! split from it at §12's pre-split band because it *is* a value: no link, no
//! socket, nothing live.
//!
//! §8.2: *"The mapping between the two names is spent at exactly one place, the
//! channel boundary, in both directions."* This is that place on the read path.
//! An entry's leaf is the client's name for the workspace and its
//! [`WORKSPACE`](crate::wire::entries::WORKSPACE) file is what the workspace
//! answers to on its host; a question crossing carries the host's name
//! ([`outbound`](Origin::outbound)) and a row landing back is labelled with the
//! leaf ([`inbound`](Origin::inbound)). Neither direction exists for
//! [`Local`](Origin::Local): there is no second namespace to map into.

use crate::boundary::codec;
use crate::boundary::reply::{Reply, WsRow};
use crate::wire::entries::Entry;
use serde_json::Value;

/// **Which channel a workspace came from** (§8.2) — a fact painted on the row,
/// never a mode the window is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// The engine in this window's own process. It renames nothing and wears
    /// no label: a local workspace is not *from* anywhere.
    Local,
    /// One §8.2 entry: the client's `leaf` for the workspace, and the `host`
    /// name it answers to there — equal to the leaf where the entry states
    /// none, which is the ordinary provisioning.
    Entry { leaf: String, host: String },
}

impl Origin {
    /// One [`Entry`] as an origin — **the one place an entry becomes a
    /// channel's identity**, so the leaf and the host-side name are read off
    /// the directory once and every consumer below reasons in this value.
    pub(crate) fn of(held: &Entry) -> Self {
        Self::Entry {
            leaf: held.leaf.clone(),
            host: held.workspace.clone(),
        }
    }

    /// The client's name for this channel — the entry leaf, `None` for
    /// [`Local`](Self::Local). The label a row wears, and the token a gesture
    /// resolves against.
    pub fn label(&self) -> Option<String> {
        match self {
            Self::Local => None,
            Self::Entry { leaf, .. } => Some(leaf.clone()),
        }
    }

    /// How a refusal names this channel — the [`label`](Self::label) as a
    /// sentence says it, so the collision refusal reads in words rather than in
    /// a variant.
    pub(crate) fn held_by(&self) -> String {
        match self.label() {
            None => "this window's own engine".to_owned(),
            Some(leaf) => format!("the entry {leaf:?}"),
        }
    }

    /// **A sentence this channel gave, attributed to it** — for a reader with
    /// several channels' answers in one list and no other way to tell which
    /// refused (the searcher's union). The local channel adds nothing, because
    /// an unattributed sentence has always meant this window's own engine and a
    /// box holding no entry must read byte for byte as it did.
    pub(crate) fn attributed(&self, said: &str) -> String {
        match self.label() {
            None => said.to_owned(),
            Some(_) => format!("{}: {said}", self.held_by()),
        }
    }

    /// `name` as the far side spells it, or `None` where the two agree and the
    /// gesture crosses byte for byte — which is every name but the one an entry
    /// renames.
    pub(super) fn outbound(&self, name: &str) -> Option<String> {
        match self {
            Self::Entry { leaf, host } if name == leaf && host != leaf => Some(host.clone()),
            _ => None,
        }
    }

    /// The inverse: `name` as this box spells it, `None` where they agree.
    fn inbound(&self, name: &str) -> Option<String> {
        match self {
            Self::Entry { leaf, host } if name == host && host != leaf => Some(leaf.clone()),
            _ => None,
        }
    }

    /// **`question` as this channel's far side reads it** — the outbound
    /// direction, spent. Undecodable, naming no workspace, and naming one this
    /// channel does not rename are one branch: nothing to rewrite, so the
    /// envelope crosses byte for byte as it was written.
    ///
    /// Every write path spends the mapping here and only here — the frame's
    /// standing reads ([`Channel::ask`](super::Channel::ask)), the window's
    /// acts and its follow lane ([`Dial`](crate::wire::dial::Dial)), and the
    /// `yog seat` verb ([`seat::channel`](crate::wire::seat)). One site, so a
    /// gesture cannot cross renamed down one path and unrenamed down another.
    pub(crate) fn carried(&self, question: &Value) -> Value {
        let rewritten = codec::decode(question).ok().and_then(|gesture| {
            let named = gesture.workspace()?;
            let host = self.outbound(&named)?;
            Some(codec::encode(&gesture.with_workspace(&host)))
        });
        rewritten.unwrap_or_else(|| question.clone())
    }

    /// **A landed reply in this box's spelling** — the inbound direction,
    /// spent. Two replies *identify* a workspace by name and both are renamed
    /// back to the leaf; every other kind lands as it was answered.
    ///
    /// The roster's rows are the obvious one. The §8.1
    /// [`Prepared`](crate::start::Prepared) is the other and it is
    /// load-bearing (bl-e349): the name it carries is handed straight back out
    /// as the next act's address (`Action::Prompt` names
    /// `prepared.workspace`), so a `Prepared` left in the HOST's spelling
    /// routes its own `Prompt` to a name no entry claims — this window's own
    /// engine, which is precisely the local misfire this mapping exists to
    /// prevent. One direction of one mapping, spent at the one boundary §8.2
    /// puts it at.
    pub(crate) fn labelled(&self, reply: Reply) -> Reply {
        match reply {
            Reply::Workspaces(mut view) => {
                for row in &mut view.rows {
                    if let Some(leaf) = self.inbound(&row.workspace) {
                        row.workspace = leaf;
                    }
                }
                Reply::Workspaces(view)
            }
            Reply::Prepared(mut prepared) => {
                if let Some(leaf) = self.inbound(&prepared.workspace) {
                    prepared.workspace = leaf;
                }
                Reply::Prepared(prepared)
            }
            other => other,
        }
    }
}

/// One workspace row with the channel it came from (§8.2) — the union roster's
/// element. A **client-side** composition: the origin is this box's fact about
/// its own material, so it is stamped where the reply lands and never crosses
/// the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterRow {
    /// The row as its engine answered it, named in this box's spelling.
    pub row: WsRow,
    /// Which channel answered it.
    pub origin: Origin,
}
