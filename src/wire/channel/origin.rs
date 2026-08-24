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

use crate::boundary::reply::WsRow;

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
    pub(super) fn inbound(&self, name: &str) -> Option<String> {
        match self {
            Self::Entry { leaf, host } if name == host && host != leaf => Some(leaf.clone()),
            _ => None,
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
