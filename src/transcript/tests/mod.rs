//! Tests for the transcript view-model.
//!
//! [`vm`] drives [`super::build`] end-to-end against tempdir-backed
//! `messages/` directories, exercising every origin-classification and
//! forgiving-parse branch through the public API. [`flow`] covers
//! enumeration order, skipping, the in-progress query, and the live tail.
//! [`compaction`] pins the one thing a readdir cannot see — the entries
//! litany's compactor deleted, derived from the hole they left in the `NNN`
//! counter, and the `summary/` prose that replaced them. [`wound`] covers the
//! other virtual entry a caller folds on: the settled-failure notice.
//!
//! What a seat makes of the record — the one-line row projection, its folds,
//! its speaker seat and the step spine drawn through it — went with the seat
//! (bl-7942), and so did every claim about glyphs.

use std::path::Path;

mod compaction;
mod flow;
mod vm;
mod wound;

/// Fixed agent id all fs-backed tests build under.
pub(super) const AGENT: &str = "20260427T120000Z-aaaa";

/// Write one message file into `<ws>/agents/<AGENT>/messages/`.
pub(super) fn write_msg(ws: &Path, name: &str, bytes: &[u8]) {
    let dir = ws.join("agents").join(AGENT).join("messages");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), bytes).unwrap();
}

/// Write one compactor summary into `<ws>/agents/<AGENT>/summary/` — a
/// **sibling** of `messages/`, which is the whole reason it needs its own
/// helper (bl-7bd2).
pub(super) fn write_summary(ws: &Path, name: &str, text: &str) {
    let dir = ws.join("agents").join(AGENT).join("summary");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), text).unwrap();
}

/// Write the latest step's `response.json` the live-tail fold reads.
pub(super) fn write_response(ws: &Path, seq: u32, bytes: &[u8]) {
    let dir = ws.join("steps").join(AGENT).join(format!("{seq:03}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("response.json"), bytes).unwrap();
}
