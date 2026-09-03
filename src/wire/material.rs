//! Where the wire's key material lives, and what its absence means (REMOTE
//! §1.4, §8; bl-b6fa).
//!
//! **yog never mints a certificate.** Provisioning is an act the operator
//! performs on the boxes, out-of-channel by ruling — so this module only ever
//! *reads*, and the three answers it can give are the whole trust bootstrap:
//!
//! - **Nothing provisioned** — `Ok(None)`. The wire is simply off: the engine
//!   listens on nothing and a seat has nowhere to dial. Removing the directory
//!   deletes config, not code (the severability test), which is why absence is
//!   an answer and not an error.
//! - **Partly provisioned** — `Err(remedy)`, naming what is missing and the
//!   target that mints it. Half a trust store is a misconfiguration, and a
//!   misconfiguration that silently degrades to *no encryption* is the failure
//!   mode this design exists to exclude.
//! - **Provisioned** — `Ok(Some(Material))`: the anchors, this role's leaf and
//!   key, and the one address.
//!
//! **An address is one fact with one home, and the home is the relationship**
//! (REMOTE §8 as amended, bl-aaec). A server binds its own `wire/address` and a
//! local seat dials it; a workspace held elsewhere names its host in its own
//! entry's `address` file ([`entries`](super::entries), REMOTE §8.2). Every
//! address still has exactly one file and no flag — two spellings of one
//! address is exactly the drift §8's name-resolution ruling removed from the
//! boundary — and one address per relationship is what a client of many
//! servers needs, no more.
//!
//! **It sits beside the world, not inside it** (`<yog-data-root>/wire`, the
//! sibling of `<yog-data-root>/world`). The world subtree is a *generated*
//! artifact — yog seeds it, and it is wiped and reseeded — while key material
//! is operator-provisioned and irreplaceable by anything yog can do. Nesting it
//! under a directory yog rebuilds would make a reseed a revocation. Entries sit
//! *inside* `wire/` ([`entries::ENTRIES`](super::entries::ENTRIES)) for that
//! same reason: an entry is the same operator-provisioned, irreplaceable class
//! of fact.

use crate::xdg::Env;
use std::path::{Path, PathBuf};

/// The material directory's leaf under the yog data root.
pub const DIR: &str = "wire";
/// The operator CA both ends verify against — one anchor set, both directions.
pub const ANCHORS: &str = "ca.pem";
/// The file naming the address the engine binds and a seat dials.
pub const ADDRESS: &str = "address";

/// The directory a **client** box files one host's material under, inside its
/// own [`DIR`] — `wire/workspaces/<workspace>/` (REMOTE §8.2). yog holds no
/// entries since bl-7942 (a seat does), but it still *issues* the leaf a
/// visiting box files there (`WIRE_LEAF`), and the instruction that goes with
/// the pair has to name the destination. One home for the word, so the mint's
/// sentence and the client that reads the directory cannot drift.
pub const ENTRIES: &str = "workspaces";

/// What that directory is **named for** — the workspace, never the leaf
/// (bl-686c). A seat holds one channel per directory under [`ENTRIES`] and
/// routes every gesture by the workspace it names, so material filed under the
/// common name `WIRE_LEAF` stated is a channel no gesture can address: present,
/// valid, correctly permissioned, unreachable. The common name inside the
/// certificate is the identity and the basename is a filing convenience (REMOTE
/// §2), which is exactly why the two are free to differ and why the
/// instruction may not spell one as the other.
///
/// It is a constant because the mint's sentence is otherwise assembled from
/// [`DIR`] and [`ENTRIES`] *"so the instruction and the directory a client
/// files it into cannot drift"* — and the one token still written by hand there
/// is the one that drifted.
pub const ENTRY: &str = "<workspace>";
/// The act that mints the lot, named in every refusal so a seat that cannot
/// start says how to make it start.
///
/// **The verb, never the make target** (bl-a0dd). It said `make wire-certs`,
/// which is a wrapper over `cargo run` and so wants a checkout: a deployed
/// engine is an installed binary or the OCI image (DESIGN §10.1) and has
/// neither. That is REMOTE §8's own argument for retiring `wire-certs.sh` —
/// *"an installed binary has no repository to find a script in"* — and it
/// condemns a Makefile exactly as far as it condemned a script. The one
/// spelling every box has is the verb.
pub const REMEDY: &str = "yog wire-certs";

/// Which end of the wire is asking. One certificate is one client identity
/// (REMOTE §2), and the server's is its own — so the leaf names differ and
/// nothing else does.
///
/// **[`Window`](Role::Window) is the local window's own end** (REMOTE §1.2 as
/// executed, bl-ae05). The window is a wire client of loopback like any other
/// client: it presents a leaf, is identified by that leaf's common name, and is
/// scoped by its registrations. It is a role of its own rather than the
/// [`Client`](Role::Client) leaf because one certificate is one identity — the
/// window and a terminal seat sharing a leaf would be one client holding two
/// pane documents' worth of facts under one name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Server,
    Client,
    Window,
}

/// Every role a mint issues a leaf for, in the order it issues them.
pub const LEAVES: [Role; 3] = [Role::Server, Role::Client, Role::Window];

impl Role {
    /// This role's leaf basename: `server`, `client` or `window`.
    pub fn leaf(self) -> String {
        match self {
            Role::Server => "server".to_owned(),
            Role::Client => "client".to_owned(),
            Role::Window => "window".to_owned(),
        }
    }

    /// The subject common name this role's certificate carries — which **is**
    /// the client identity the engine reads off it (REMOTE §2,
    /// [`leaf::common_name`](crate::registry::leaf::common_name)). Spelled here
    /// so the mint and the identity that seats a registration are one fact: the
    /// window's is [`registry::WINDOW`](crate::registry::WINDOW), because a
    /// client identity's home is the registry.
    pub fn common_name(self) -> String {
        match self {
            Role::Server => "yog-server".to_owned(),
            Role::Client => "yog-client".to_owned(),
            Role::Window => crate::registry::WINDOW.to_owned(),
        }
    }
}

/// One end's provisioned material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    /// The operator CA, PEM.
    pub anchors: PathBuf,
    /// This end's certificate chain, PEM.
    pub chain: PathBuf,
    /// This end's private key, PEM.
    pub key: PathBuf,
    /// `host:port` — bound by the engine, dialled by a seat.
    pub address: String,
}

/// The material directory for a composed world.
pub fn dir(world: &Env) -> PathBuf {
    world.yog_data_root().join(DIR)
}

/// Read `role`'s material out of `world`. See the module doc for the three
/// answers; the `Err` names every missing file at once, because a remedy that
/// reveals one gap per run is a remedy run four times.
pub fn read(world: &Env, role: Role) -> Result<Option<Material>, String> {
    read_dir(&dir(world), role)
}

/// [`read`] against a directory outright — the world-free core, so a test names
/// its own scratch tree the way the folds elsewhere do.
pub fn read_dir(dir: &Path, role: Role) -> Result<Option<Material>, String> {
    let leaf = role.leaf();
    let wanted = [
        ANCHORS.to_owned(),
        format!("{leaf}.pem"),
        format!("{leaf}.key"),
        ADDRESS.to_owned(),
    ];
    let missing: Vec<&String> = wanted.iter().filter(|f| !dir.join(f).is_file()).collect();
    if missing.len() == wanted.len() {
        return Ok(None);
    }
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|f| f.as_str()).collect();
        return Err(format!(
            "the wire is half-provisioned at {}: missing {} — run `{REMEDY}`",
            dir.display(),
            names.join(", ")
        ));
    }
    // A file that will not read yields no address, and no address is the same
    // refusal an empty one earns: one branch, because "unreadable" and "empty"
    // are one fact about what this box can be told to dial.
    let address = std::fs::read_to_string(dir.join(ADDRESS))
        .unwrap_or_default()
        .trim()
        .to_owned();
    if address.is_empty() {
        return Err(format!(
            "{} names no address; it must hold one host:port — run `{REMEDY}`",
            dir.join(ADDRESS).display()
        ));
    }
    Ok(Some(Material {
        anchors: dir.join(ANCHORS),
        chain: dir.join(format!("{leaf}.pem")),
        key: dir.join(format!("{leaf}.key")),
        address,
    }))
}

#[cfg(test)]
mod tests;
