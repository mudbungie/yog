//! **Seating the window** (REMOTE §4, §4.1; bl-ae05): the one act the
//! [`Asker`](super::Asker) performs that is not a question, and the one it
//! performs on the loopback channel alone (bl-670c).
//!
//! Authorization is registration (REMOTE §4) and the window carries a
//! certificate, so it is scoped like any other client — it would see nothing at
//! all unless something wrote its registrations. The engine that enumerates the
//! workspaces is what knows them, so it seats its own window's leaf in each and
//! re-seats as the enumeration grows: a workspace founded while the window is up
//! is registered within one pass, with no create to detect.
//!
//! **An entry channel has no [`Seating`] at all**, which is REMOTE §1.4 rather
//! than a limitation. A registration is a file on the *host's* disk, written by
//! the operator who owns that box; nothing on this side of the wire may write
//! one, and the window's leaf reaches a host the way its certificate did — by
//! hand, out of channel.

use crate::state::{SnapshotCell, latest_snapshot};
use std::path::PathBuf;

/// What the loopback asker seats its window in: the enumeration this engine
/// publishes, and the registry root it writes into. Held together because they
/// are one act's two halves — an entry channel has neither, the enumeration
/// being the host's and so being the registry.
pub(super) struct Seating {
    snap: SnapshotCell,
    state_root: PathBuf,
}

impl Seating {
    pub(super) fn new(snap: SnapshotCell, state_root: PathBuf) -> Self {
        Self { snap, state_root }
    }

    /// Register the window's identity in every workspace the published
    /// derivation enumerates. Idempotent and quiet: a registration that is
    /// already there is one directory read, and one that cannot be written is a
    /// state root broken in ways this pass cannot answer for.
    pub(super) fn seat_window(&self) {
        let client = crate::registry::window();
        let snap = latest_snapshot(&self.snap);
        let seated = crate::registry::registered(&self.state_root, &client);
        for workspace in &snap.workspaces {
            let name = snap.ws_name(&workspace.path);
            if !seated.contains(&name) {
                let _ = crate::registry::register(&self.state_root, &client, &name);
            }
        }
    }
}
