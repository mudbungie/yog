//! **A workspace this box founds is seen by the seats this box minted**
//! (REMOTE §4.1 as amended; bl-bd48, bl-0fbc) — the rule both intakes spend,
//! in its own file beside the room they open onto ([`consume`](super)):
//! everything there is one pass over the inbox.
//!
//! **The defect it closes was a workspace nobody could see.** §4 auto-registers
//! *the creating client*, and the creator of everything made on the engine's own
//! box is the reserved in-world identity — `yog gesture`, the `gestures/` inbox
//! — which holds no certificate and is registered nowhere. So `yog gesture --ws
//! ops '/prepare'`, the act both READMEs teach on a headless box, founded a
//! workspace every seat then answered `unknown workspace` for, and no gesture
//! could repair it: `/enroll` mints, and the workspace namespace is one global
//! set, so the name was spent on something nothing could reach, rename or
//! delete.
//!
//! **What is registered is this box's own leaves, and that is the narrowest
//! answer.** The alternatives were wider: registering *every* client that
//! exists would hand a foot on another machine a workspace it was never
//! enrolled in, and a `/register` gesture would be a second registrar beside
//! `/enroll`. The `client` and `window` leaves are the material yog's own mint
//! writes into this box's `wire/` and never carries anywhere: they are the
//! seats OF the box that holds the CA, which is the operator standing at it.
//! §1.5's separation argument is about a leaf carried to another box, and this
//! is the one class of leaf that is not.
//!
//! **Founding only, never joining.** A registration is revoked by deleting the
//! file (§4), so seating one on every gesture that merely *names* a workspace
//! would undo an operator's `rm` on their next `/prepare`. The question asked
//! is therefore whether the name was addressable **before** the gesture ran.

use serde_json::Value;

use crate::boundary::Gesture;
use crate::boundary::dispatch::Deps;
use crate::registry::{self, Client};
use crate::wire::material;

/// The workspace this gesture would FOUND — the one it names, when nothing the
/// caller can address answers to that name yet.
///
/// Asked before the gesture runs, because afterwards every founding and every
/// ordinary act name a workspace that exists. A scoped caller's unaddressable
/// set includes another client's workspace, which is deliberate and harmless:
/// the resolver refuses to *join* one (`dispatch::resolve`), so the only
/// gesture that can both reach here and succeed is one that founded something.
pub(super) fn pending(deps: &Deps, gesture: &Gesture) -> Option<String> {
    let name = gesture.workspace()?;
    deps.snapshot.ws_path(&name).err().map(|_| name)
}

/// Seat this box's own leaves in the workspace [`pending`] named, once the
/// gesture that founded it has answered. A refusal seats nothing — the same
/// `kind`-is-present test §4's own auto-registration reads a success by.
pub(super) fn seat(deps: &Deps, founded: Option<&str>, answered: &Value) {
    let Some(workspace) = founded.filter(|_| answered.get("kind").is_some()) else {
        return;
    };
    for client in seats(&material::dir(&deps.world)) {
        let _ = registry::register(&deps.state_root, &client, workspace);
    }
}

/// The client identities this box's own mint issued for itself: the common
/// name of every [`SEATS`](material::SEATS) role whose leaf is actually on
/// disk. Present-only, so a box that never minted one — or an operator who
/// removed it — collects no registration for a certificate that does not
/// exist.
fn seats(wire: &std::path::Path) -> Vec<Client> {
    material::SEATS
        .iter()
        .filter(|role| wire.join(format!("{}.pem", role.leaf())).is_file())
        .filter_map(|role| Client::parse(&role.common_name()).ok())
        .collect()
}
