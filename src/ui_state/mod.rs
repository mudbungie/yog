//! yog's converging UI state (DESIGN §4.1, §15 Y8) — the four attention `seen`
//! watermarks, `pinned`, `ceiling` and `prices`.
//!
//! **It is one document, and every key in it is a fact about the world**
//! (REMOTE §7 as amended, bl-f936): an acknowledgement, a pin, a spend
//! ceiling — assertions every seat shares, because attention answered on the
//! phone must clear on the desktop (I0).
//!
//! **A fact about a pane of glass is not stored here and never was reachable**
//! — how wide a panel was dragged, what is collapsed, how big the text is.
//! bl-8bbc gave those a second per-client document on the promise that they
//! would converge across one client's seats; the frame that read them left with
//! bl-7942 and no gesture ever replaced it, so for the whole of that document's
//! life nothing wrote a key and no reply carried one. A glass fact is each
//! seat's own local storage (§4.1), which costs a boundary act nothing and
//! keeps this document to the facts that genuinely converge.
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
mod prices;
/// The §3.6 workspace prune — a deleted workspace's keys leave the document.
mod prune;

pub(crate) use clock::format_iso8601;
pub use clock::{Clock, SystemClock, epoch_from_iso8601, iso8601_extended};
pub use fields::SeenKind;
use json::{default_root, descend, parse_or_default, string_array};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

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

/// The live UI-state handle: **one** document (REMOTE §7 as amended, bl-f936),
/// `ui.json`, holding the operator's facts about the world.
///
/// It was two until bl-f936. The second held one client's glass facts and was
/// answered to nobody — no `Action` wrote a key and no `Reply` carried one from
/// the moment the frame that read them left — so the promise it existed for
/// ("a fold answered on the phone clears on the desktop") was never kept by it.
/// A seat keeps its own glass facts; this file keeps what converges.
pub struct UiState {
    world: doc::Doc,
}

impl UiState {
    /// Open the world document at `path`. Missing or corrupt is the fold
    /// identity — all defaults, never an error (§4.1's forgiving read).
    pub fn open(path: PathBuf) -> Self {
        Self {
            world: doc::Doc::open(path),
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
