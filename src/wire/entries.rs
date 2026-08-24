//! **The client-side workspace** (REMOTE §8.2, bl-aaec): what this box holds
//! so it can participate in a workspace hosted on another box.
//!
//! An **entry** is a directory under `<yog-data-root>/wire/workspaces/<leaf>/`
//! carrying the channel facts that reach one workspace — the host engine's
//! anchors, this box's leaf and key for it, the host's `address`, and
//! optionally the name the workspace bears *there*. It is the client's half of
//! the pair a server-side registration is the other half of: possession, where
//! registration is permission.
//!
//! **It is not a second noun, and there is no server object.** A workspace is
//! one word at both ends; "entry" names a *spelling* of it. The client-side
//! unit is the (server, workspace) participation and never the server, so
//! nothing here enumerates a server or holds a fact about one — a server is
//! the [`ADDRESS`](super::material::ADDRESS) inside an entry, entire. Two
//! entries naming one address are two trust relationships that happen to
//! terminate at one listener.
//!
//! **The shape already existed.** Four of the five files are exactly what a
//! pure-client box holds flat, so
//! [`material::read_dir`](super::material::read_dir) with
//! [`Role::Client`](super::material::Role::Client) reads an entry unchanged:
//! an entry is that directory, one level down, named. That is also the whole
//! migration path for a box whose flat directory was really a client set aimed
//! at another machine — one `mkdir` and one `mv`.
//!
//! **Separation is the absence of a mechanism.** Entries share nothing — not
//! anchors (two servers are two operators' trust roots), not leaves (one
//! certificate is one client identity), not addresses, not conversations. So
//! there is no inheritance from the flat root and no path by which one entry
//! can be read through another; the only structure below is a `readdir` and a
//! read per directory.
//!
//! **A refusal is one entry's, never the set's.** [`Entry::channel`] carries
//! its own `Result`, so a half-provisioned entry is painted unreachable by its
//! own consumers while every other entry stands. The whole-shell refusal stays
//! reserved for the one wire the window cannot exist without: its own.
//!
//! **Migration: none.** A box with a flat `wire/` and no `workspaces/`
//! directory is this module answering with zero entries — the general path
//! with empty inputs, not a case tested for. Nothing here writes anything:
//! material reaches an entry by the operator's hand, out of channel, forever
//! (REMOTE §1.4).

use super::material::{self, Material, REMEDY, Role};
use crate::xdg::Env;
use std::path::Path;

/// The entries directory's leaf under [`material::DIR`] — the one level of
/// naming that turns the flat client shape into a workspace this box holds.
pub const ENTRIES: &str = "workspaces";

/// The optional file naming the workspace **on its host**, for when that
/// differs from the entry's leaf. A host's namespace is the host's fact and
/// two hosts may both call something `home`; the remedy for a collision is a
/// local rename, which is `mv`, never a server-side rewrite.
pub const WORKSPACE: &str = "workspace";

/// One workspace this box participates in elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The directory name — the *client's* name for the workspace, which the
    /// roster paints and every gesture resolves.
    pub leaf: String,
    /// The name the workspace answers to on its host. The [`WORKSPACE`] file
    /// when it states one, the leaf otherwise. The mapping between the two is
    /// spent at exactly one place, the channel boundary, in both directions.
    pub workspace: String,
    /// This entry's channel material, or the sentence saying why it has none.
    pub channel: Result<Material, String>,
}

/// Every entry a composed world holds, sorted by [`leaf`](Entry::leaf).
pub fn entries(world: &Env) -> Vec<Entry> {
    read_dir(&material::dir(world).join(ENTRIES))
}

/// [`entries`] against the entries directory outright — the world-free core,
/// so a test names its own scratch tree the way the folds elsewhere do.
///
/// **A directory that will not read is zero entries, not a refusal.** Absent,
/// unreadable and empty are one fact — this box holds no workspace elsewhere —
/// and that fact is the shape every box had before §8.2 existed.
pub fn read_dir(dir: &Path) -> Vec<Entry> {
    let Ok(listing) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut held: Vec<Entry> = listing
        .flatten()
        .map(|found| found.path())
        // An entry *is* a directory (§8.2). A stray file beside them names no
        // intent and is not an entry with a problem.
        .filter(|path| path.is_dir())
        .map(|path| entry(&path))
        .collect();
    held.sort_by(|a, b| a.leaf.cmp(&b.leaf));
    held
}

/// One directory read as the entry it claims to be.
fn entry(dir: &Path) -> Entry {
    let leaf = dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let channel = match material::read_dir(dir, Role::Client) {
        Ok(Some(held)) => Ok(held),
        // Nothing provisioned is silence at the flat root, where absence is
        // the off switch. Here it is a refusal: a directory somebody made
        // names an intent, and an intent with no material behind it is the
        // half-provisioned failure one step earlier.
        Ok(None) => Err(format!(
            "{} is an empty entry: its material is minted on the host that \
             issued it (`{REMEDY}` there) and carried here by hand",
            dir.display()
        )),
        Err(refusal) => Err(refusal),
    };
    let workspace = named(dir, &leaf);
    Entry {
        leaf,
        workspace,
        channel,
    }
}

/// The name this workspace bears on its host. Absent, unreadable and empty are
/// one branch for the reason `material`'s `address` read has one: they are one
/// fact — the entry states no host-side name — and the leaf is then the name.
fn named(dir: &Path, leaf: &str) -> String {
    let stated = std::fs::read_to_string(dir.join(WORKSPACE))
        .unwrap_or_default()
        .trim()
        .to_owned();
    if stated.is_empty() {
        leaf.to_owned()
    } else {
        stated
    }
}

#[cfg(test)]
mod tests;
