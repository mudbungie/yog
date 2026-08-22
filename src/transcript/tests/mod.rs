//! Tests for the transcript view-model.
//!
//! [`vm`] drives [`super::build`] end-to-end against tempdir-backed
//! `messages/` directories, exercising every origin-classification and
//! forgiving-parse branch through the public API. [`flow`] covers
//! enumeration order, skipping, the in-progress query, and the live tail.
//! [`compaction`] pins the one thing a readdir cannot see — the entries
//! lernie's compactor deleted, derived from the hole they left in the `NNN`
//! counter, and the `summary/` prose that replaced them.
//! [`rows`] pins the §11 one-line projection — classification, the derived
//! auto-state, the override flip — and [`turns`] the rollup over it: a
//! finished turn's machinery as one aggregate line, a live turn's steps
//! streaming visibly. [`spine`] pins the step spine drawn through the chat —
//! one clickable rule per operable commit, the cards born at it, and the pin
//! the click raises. [`render`] shape-walks the egui widget
//! headlessly per the git_tree render-test pattern, and [`tail`] does the same
//! walk on a viewport too short to hold the conversation, where the §11 tail
//! anchor decides which rows are on screen.

use std::collections::HashSet;
use std::path::Path;

use super::{AutoExpand, Reading, Transcript};

mod compaction;
mod flow;
mod folds;
mod legible;
mod parity;
mod render;
mod rows;
mod speaker;
mod spine;
mod tail;
mod turns;
mod vm;

/// Fixed agent id all fs-backed tests build under.
pub(super) const AGENT: &str = "20260427T120000Z-aaaa";

/// How the chat is read under the tests' one speaker.
pub(super) fn reading(raw: bool, auto: AutoExpand) -> Reading {
    Reading {
        speaker: rows::SPEAKER.to_owned(),
        raw,
        auto,
    }
}

/// Paint the chat with **no spine drawn through it** — the shape every test
/// about rows rather than rules wants: an empty spine has no rule to seat, so
/// the paint is the row projection and nothing else.
pub(super) fn plain(
    ui: &mut egui::Ui,
    t: &Transcript,
    raw: bool,
    auto: AutoExpand,
    folds: &mut HashSet<String>,
) {
    let _ = super::render(
        ui,
        t,
        &reading(raw, auto),
        folds,
        &crate::rail::Rail::default(),
        &mut None,
    );
}

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
