//! One JSON document on disk — the file mechanics [`UiState`](super::UiState)
//! is made of, now that it is made of **two** of them (REMOTE §7, bl-8bbc).
//!
//! Forgiving load, echo hash, write-through atomic save: the §4.1 discipline,
//! stated once and spent twice. It was `UiState`'s own body until the per-seat
//! split gave the handle a second document to keep, at which point "the
//! document" and "the pair of documents a seat reads through" became two
//! things and only one of them is a file.

use super::{content_hash, default_root, parse_or_default};
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::PathBuf;

/// A live handle on one JSON document: its path, its root map, and the hash of
/// what is on disk. The hash does double duty — echo suppression on read, and
/// write elision on mutate (identical bytes are already there, so there is
/// nothing to do).
pub(super) struct Doc {
    pub(super) path: PathBuf,
    pub(super) root: Map<String, Value>,
    pub(super) last_hash: Option<u64>,
}

impl Doc {
    /// Forgiving load (missing/unreadable/corrupt ⇒ default doc, never an error).
    pub(super) fn open(path: PathBuf) -> Self {
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
    pub(super) fn save(&mut self) {
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
        let tmp = crate::scratch::temp_in(dir, &name.to_string_lossy());
        fs::write(&tmp, bytes)?;
        fs::rename(&tmp, &self.path)
    }
}
