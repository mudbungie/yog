//! Wire key material, minted at test runtime (REMOTE §1.4; bl-b6fa).
//!
//! **No certificate is ever committed.** A fixture key in the tree is a private
//! key in a public repository — the exact class `make leak-scan` refuses — and
//! it would be one whether or not it guarded anything. So the suite performs
//! the same out-of-channel act the engine's own boot performs, through the same
//! function: [`provision::mint`](crate::wire::provision::mint), which is the
//! crate's one `openssl` recipe (bl-ae05). A second copy here would be a second
//! recipe, and two spellings of one act drift within a week.

use crate::wire::material::{ADDRESS, ANCHORS, Material, Role};
use crate::xdg::Env;
use std::path::Path;

/// The address minted into the material: loopback on a kernel-chosen port. A
/// test never names a port, so two tests never collide on one; the bound
/// address comes back from [`Listener::address`](crate::wire::server::Listener::address).
pub(crate) const EPHEMERAL: &str = "127.0.0.1:0";

/// **Aim a fixture world's wire at a kernel-chosen port — the one seed a test
/// that boots a REAL listener writes first** (bl-4c50).
///
/// The address is a fact of the world with one home (REMOTE §8), and
/// [`provision::ensure`](crate::wire::provision::ensure) mints *taking the
/// address the directory already names*; `127.0.0.1:<PORT>` is only what it
/// falls back to when nothing names one. A test that let it fall back asserted
/// on a shared machine resource: the operator's own running window holds that
/// one port, so the boot's `TcpListener::bind` answered `Address already in
/// use` on **every** run and the local gate could not pass while the app under
/// development was up.
///
/// It writes `address` and nothing else, so every certificate is still absent
/// and the boot's own mint is still the thing under test. The loopback default
/// itself is covered where nothing binds —
/// `provision::tests::an_unprovisioned_box_founds_its_own_loopback_trust_root`
/// reads it out of the file.
pub(crate) fn ephemeral(world: &Env) {
    let dir = crate::wire::material::dir(world);
    std::fs::create_dir_all(&dir).expect("the fixture world's wire dir");
    std::fs::write(dir.join(ADDRESS), format!("{EPHEMERAL}\n")).expect("the fixture address");
}

/// Mint a CA and every leaf into `dir`, plus the `address` file — the whole of
/// what a boot mints, in the shape [`crate::wire::material::read`] expects.
pub(crate) fn mint(dir: &Path) {
    let _guard = super::spawn_guard();
    crate::wire::provision::mint(dir, EPHEMERAL, false).expect("the wire mint runs");
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
