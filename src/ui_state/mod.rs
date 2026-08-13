//! The `ui.json` document (DESIGN §4.1, §15 Y8): yog's one converging UI-state
//! artifact — the four attention `seen` watermarks, `pinned`, `collapsed`,
//! `identity_last_used`, and the boolean knobs ([`knobs`]: the §11
//! transcript-density automatics).
//!
//! **Single source of truth:** the whole document is one [`serde_json::Value`]
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

use serde_json::{Map, Value};

/// The §3.5 spend ceiling — `ui.json`'s `ceiling` number (§4.1).
mod ceiling;
mod fields;
mod json;
mod knobs;
/// The §11 resizable panel sizes — `ui.json`'s `panels` object (§4.1).
mod panels;
mod prices;
/// The §3.6 workspace prune — a deleted workspace's keys leave the document.
mod prune;

pub use fields::SeenKind;
use json::{default_root, descend, parse_or_default, string_array};
pub use panels::Panel;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::PathBuf;
use std::time::Instant;

/// The two §11 transcript auto-expand knobs' `ui.json` keys (§4.1).
const EXPAND_RESPONSES: &str = "transcript_expand_responses";
const EXPAND_OTHERS: &str = "transcript_expand_others";

/// Injected time (§7.2: "all timing is clock-injected"). `ui.json` itself is
/// untimed (write-through, above); the seam lives here as the crate's **one**
/// time injection, consumed by the §7.2 derivation worker, its sweep schedule
/// and the §10 probe TTL cache.
///
/// Two readings, one source. [`now`](Clock::now) is monotonic — only
/// differences between calls matter (debounce windows, sweep deadlines,
/// snapshot age). [`stamp`](Clock::stamp) is the wall-clock `ops.jsonl` field
/// (§4.2), opaque to `opslog`: it exists here because §7.2's worker writes its
/// own drift lines off the frame thread, and a second time seam for the string
/// would be a second thing to inject and fake.
///
/// `Send + Sync` because the worker thread holds the same `Arc<dyn Clock>` the
/// frame injected (§7.2) — the schedule it gates and the test that advances it
/// are on different threads by construction.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
    /// The wall-clock `ops.jsonl` timestamp (§4.2) — unix seconds as a string,
    /// the crate's timestamp convention.
    fn stamp(&self) -> String;
}

#[derive(Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
    fn stamp(&self) -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
            .to_string()
    }
}
/// The crate's **one** human-timestamp spelling, ISO 8601 extended:
/// `YYYY-MM-DD HH:MM:SSZ`. Assembled from already-decomposed calendar fields
/// so every caller — the chat header's when-seat (bl-16da, whose id already
/// carries `y/mo/d/h/mi/s` as digit groups) and the activity row's leading
/// column (bl-61db, whose `ts` is raw epoch seconds) — renders through this
/// one line rather than two independently-written format strings that could
/// drift apart.
pub(crate) fn format_iso8601(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> String {
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}Z")
}

/// Unix epoch seconds → [`format_iso8601`] (bl-61db: the activity row's raw
/// `1785630266` rendered as `2026-08-02 00:24:26Z`). Proleptic Gregorian, UTC,
/// no leap seconds — Howard Hinnant's `civil_from_days`
/// (<https://howardhinnant.github.io/date_algorithms.html>), the crate's one
/// calendar routine so this stays free of a `chrono`/`time` dependency.
pub fn iso8601_extended(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let secs_of_day = epoch_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format_iso8601(
        year,
        month,
        day,
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    )
}

/// Days since the Unix epoch (1970-01-01) → `(year, month, day)`, proleptic
/// Gregorian. Ported verbatim from Hinnant's `civil_from_days` (public
/// domain), which is exact for the whole `i64` range this crate ever sees.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day)
}

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

/// The live `ui.json` handle: document, path, and last-known on-disk hash. The
/// hash does double duty — echo suppression on read, and write elision on
/// mutate (identical bytes are already on disk, so there is nothing to do).
pub struct UiState {
    path: PathBuf,
    root: Map<String, Value>,
    last_hash: Option<u64>,
}

impl UiState {
    /// Forgiving load (missing/unreadable/corrupt ⇒ default doc, never an error).
    pub fn open(path: PathBuf) -> Self {
        let (root, last_hash) = match fs::read(&path) {
            Ok(bytes) => (parse_or_default(&bytes), Some(content_hash(&bytes))),
            Err(_) => (default_root(), None),
        };
        Self {
            path,
            root,
            last_hash,
        }
    }

    /// True iff `bytes` hash to the content we last wrote/read/adopted (§4.1).
    pub fn is_echo(&self, bytes: &[u8]) -> bool {
        self.last_hash == Some(content_hash(bytes))
    }

    /// Wholesale-adopt an external change (LWW whole-file, I5).
    pub fn adopt(&mut self, bytes: &[u8]) {
        self.root = parse_or_default(bytes);
        self.last_hash = Some(content_hash(bytes));
    }

    /// Land the document on disk, now — the write-through every mutator ends
    /// on. Elided when the bytes already hash to what is on disk (the no-op
    /// gesture: re-acknowledging a seen agent, re-collapsing a collapsed
    /// section), which is what keeps a held key from writing per repeat.
    ///
    /// Infallible by construction: a failed write leaves `last_hash` alone, so
    /// the next mutation retries the whole document — this is last-writer-wins
    /// whole-file state (§4.1), never a delta that could be half-applied. There
    /// is no caller who could do better with an `io::Error` (both former flush
    /// sites discarded it), and no `ui.json` write failure is worth taking a
    /// gesture down.
    fn save(&mut self) {
        let bytes = self.serialize();
        let hash = content_hash(&bytes);
        if self.last_hash == Some(hash) {
            return;
        }
        if self.write_atomic(&bytes).is_ok() {
            self.last_hash = Some(hash);
        }
    }

    /// Byte-deterministic serialization (`Map` sorts keys; arrays are canonical).
    fn serialize(&self) -> Vec<u8> {
        // A JSON object of strings/maps always serializes; empty on the impossible error.
        serde_json::to_vec_pretty(&self.root).unwrap_or_default()
    }

    /// Temp dotfile in the destination dir + `rename` (I3); creates the dir.
    fn write_atomic(&self, bytes: &[u8]) -> io::Result<()> {
        let dir = self.path.parent().ok_or(io::Error::other("no parent"))?;
        fs::create_dir_all(dir)?;
        let name = self
            .path
            .file_name()
            .ok_or(io::Error::other("no file name"))?;
        let tmp_name = format!(".{}.yog-tmp-{}", name.to_string_lossy(), std::process::id());
        let tmp = dir.join(tmp_name);
        fs::write(&tmp, bytes)?;
        fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests;
