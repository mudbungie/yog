//! yog's converging UI state (DESIGN §4.1, §15 Y8) — the four attention `seen`
//! watermarks, `pinned`, `identity_last_used`, `ceiling`, `prices`, and the
//! pane's own `panels` / `collapsed` / knobs ([`knobs`]: the §11
//! transcript-density automatics, the zoom, the §6 escalation).
//!
//! **It is two documents, split on what the fact is about** (REMOTE §7,
//! bl-8bbc). `ui.json` holds facts about the **world** — an acknowledgement,
//! a pin, a spend ceiling — and every seat shares it, because attention
//! answered on the phone must clear on the desktop (I0). A **pane of glass**
//! fact — how wide a panel was dragged, what is collapsed, how big the text is
//! — belongs to the client whose glass it is, and lives in that client's own
//! document under [`registry`](crate::registry). Both are read through one
//! [`UiState`], so which file owns a key is stated exactly once, in the
//! accessor for that key, and no caller knows there are two.
//!
//! **Single source of truth:** each document is one [`serde_json::Value`]
//! object (`root`); every known field is a *query* over that map and every
//! unknown key round-trips for free — no parallel typed struct plus extra-map
//! to drift (the "one struct, flattened extra" discipline without a `serde`
//! derive dependency: `serde_json` only).
//!
//! Convergence (§4.1, I5) is last-writer-wins whole-file: forgiving load
//! (missing/corrupt ⇒ default doc, never an error), **write-through** atomic
//! writes (temp dotfile + `rename`, I3), echo suppression by content hash
//! ([`UiState::is_echo`]), and wholesale [`UiState::adopt`] otherwise.
//!
//! **No write is ever in flight.** Every mutator lands on disk before it
//! returns, so the document has no RAM window to lose — not to a SIGTERM
//! (`pkill`, what `make ux` does every iteration), not to a SIGKILL, not to a
//! crash. This dissolves the shutdown-hook problem instead of handling one
//! signal's worth of it (bl-b54e): there is no exit path to flush on, graceful
//! or otherwise. The coalescing a debounce used to buy is bought instead by
//! the same content hash that suppresses echoes — a mutation that does not
//! change the bytes writes nothing at all, so re-acknowledging an already-seen
//! agent is free and a held arrow key writes only on the steps that change
//! something.

/// The §3.5 spend ceiling — `ui.json`'s `ceiling` number (§4.1).
mod ceiling;
/// The crate's one injected time seam and its one calendar routine — split off
/// at §12's budget and re-exported here, so `ui_state::Clock` and
/// `ui_state::iso8601_extended` stay the one address they always were.
mod clock;
/// One JSON document's file mechanics — forgiving load, echo hash, atomic
/// write-through — spent twice since the REMOTE §7 split (bl-8bbc).
mod doc;
mod fields;
mod json;
mod knobs;
/// The §11 resizable panel sizes — `ui.json`'s `panels` object (§4.1).
mod panels;
mod prices;
/// The §3.6 workspace prune — a deleted workspace's keys leave the document.
mod prune;

pub(crate) use clock::format_iso8601;
pub use clock::{Clock, SystemClock, epoch_from_iso8601, iso8601_extended};
pub use fields::SeenKind;
use json::{default_root, descend, parse_or_default, string_array};
pub use panels::Panel;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// The two §11 transcript auto-expand knobs' `ui.json` keys (§4.1).
const EXPAND_RESPONSES: &str = "transcript_expand_responses";
const EXPAND_OTHERS: &str = "transcript_expand_others";

/// Stable content hash of file bytes — the echo-suppression identity (§4.1).
pub fn content_hash(bytes: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

/// Startup focus (§4.1, §6): first attention-bearing workspace in the caller's
/// derived roster order, else the first, else none. Pure over ids.
pub fn derive_startup_focus(roster: &[&str], attention: &[&str]) -> Option<String> {
    for &w in roster {
        if attention.contains(&w) {
            return Some(w.to_string());
        }
    }
    roster.first().map(|w| (*w).to_string())
}

/// The live UI-state handle: **two** documents (REMOTE §7, bl-8bbc), read and
/// written through one interface.
///
/// - `world` is `ui.json` — the operator's facts about the world, shared by
///   every seat as they always were. Attention answered on the phone must clear
///   on the desktop; that is I0's whole point.
/// - `pane` is that seat's client's own document — the facts about a pane of
///   glass, held server-side so a client that is stateless (REMOTE §6) still
///   finds its panel sizes, and so any two seats of one client converge.
///
/// The split is invisible to every caller: which document owns a key is a
/// property of the key, stated once in the accessor that reads it, so nothing
/// outside this module knows there are two files.
pub struct UiState {
    world: doc::Doc,
    pane: doc::Doc,
}

impl UiState {
    /// The window's own handle: the world document at `path`, with the
    /// **window client's** pane beside it (REMOTE §7 as amended, bl-ae05).
    ///
    /// The pane keys on [`WINDOW`](crate::registry::WINDOW) rather than on
    /// `local` because the window carries its own certificate now and is a
    /// client like any other — so the document the frame writes through here
    /// and the one a gesture the window sends over the wire lands in are the
    /// same file, which is the whole of what keying it on the leaf buys.
    ///
    /// **The pane path is derived, never stored** — `ui.json` lives at yog's
    /// state root and `clients/` is its sibling, so the layout answers where
    /// the pane is rather than a second field carrying it.
    pub fn open(path: PathBuf) -> Self {
        let state_root = path.parent().unwrap_or(&path).to_path_buf();
        let pane = crate::registry::pane(&state_root, &crate::registry::window());
        Self::open_at(path, pane)
    }

    /// The handle a seat reads through (REMOTE §4, §7): the shared world
    /// document at `path`, and the pane document at `pane`.
    ///
    /// The pane is named outright rather than derived here, because the caller
    /// that has a client to name — the wire's scoped intake — already holds the
    /// state root it reads that client's registrations out of, and deriving a
    /// second one off `path` would make one location two facts.
    pub fn open_at(path: PathBuf, pane: PathBuf) -> Self {
        Self {
            world: doc::Doc::open(path),
            pane: doc::Doc::open(pane),
        }
    }

    /// True iff `bytes` hash to the content we last wrote/read/adopted (§4.1).
    /// The **world** document: it is the one an external editor and the §7.2
    /// worker's watch both name, and the one a second face converges with.
    pub fn is_echo(&self, bytes: &[u8]) -> bool {
        self.world.last_hash == Some(content_hash(bytes))
    }

    /// Wholesale-adopt an external change to the world document (LWW
    /// whole-file, I5).
    pub fn adopt(&mut self, bytes: &[u8]) {
        self.world.root = parse_or_default(bytes);
        self.world.last_hash = Some(content_hash(bytes));
    }
}

#[cfg(test)]
mod tests;
