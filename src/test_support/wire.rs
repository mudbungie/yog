//! Wire key material, minted at test runtime (REMOTE §1.4; bl-b6fa).
//!
//! **No certificate is ever committed.** A fixture key in the tree is a private
//! key in a public repository — the exact class `make leak-scan` refuses — and
//! it would be one whether or not it guarded anything. So the suite performs
//! the same out-of-channel act the engine's own boot performs, through the same
//! function: [`provision::mint`](crate::wire::provision::mint), which is the
//! crate's one `openssl` recipe (bl-ae05). A second copy here would be a second
//! recipe, and two spellings of one act drift within a week.

use crate::wire::material::{ANCHORS, Material, Role};
use std::path::Path;

/// The address minted into the material: loopback on a kernel-chosen port. A
/// test never names a port, so two tests never collide on one; the bound
/// address comes back from [`Listener::address`](crate::wire::server::Listener::address).
///
/// It is also what self-provisioning itself requests since bl-dc14 —
/// bl-4c50's fixture seed (an `ephemeral` writer that pre-named this address
/// so a boot would not fall back to a fixed port) dissolved into the default:
/// a fixture world that names nothing now gets exactly this, the same way a
/// real box does.
pub(crate) const EPHEMERAL: &str = "127.0.0.1:0";

/// Mint a CA and every leaf into `dir`, plus the `address` file — the whole of
/// what a boot mints, in the shape [`crate::wire::material::read`] expects.
pub(crate) fn mint(dir: &Path) {
    crate::wire::provision::mint(dir, EPHEMERAL, &[], false).expect("the wire mint runs");
}

/// The material for `role` in `dir`, aimed at `address` — the read
/// [`crate::wire::material::read`] performs, with the bound port substituted in
/// (the file says `:0`, and only the listener knows what that became).
pub(crate) fn material(dir: &Path, role: Role, address: &str) -> Material {
    let leaf = role.leaf();
    Material {
        anchors: dir.join(ANCHORS),
        chain: dir.join(format!("{leaf}.pem")),
        key: dir.join(format!("{leaf}.key")),
        address: address.to_owned(),
    }
}
