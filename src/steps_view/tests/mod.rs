//! Tests for the steps inspector.
//!
//! [`vm`] drives [`super::build`] / [`super::detail`] against tempdir-backed
//! `steps/<agent>/NNN/` trees, covering enumeration order, the reused
//! framing/attempts/token derivations, forgiving parsing, and the drill-in.
//! [`render`] shape-walks the widget headlessly per the transcript pattern,
//! [`raw`] holds the §11 Raw toggle's half of that walk (S7-T1: the record
//! file's bytes unaltered), and [`wound`] drives the §7.3 no-response state
//! from the shape the real substrate left behind, all the way to the painted
//! sentence. [`tail`] does the render walk on a viewport too short for the
//! table, where the §11 tail anchor decides which steps are on screen.

use std::path::{Path, PathBuf};

mod raw;
mod render;
mod tail;
mod vm;
mod wound;

/// Fixed agent id the fs-backed tests build under.
pub(super) const AGENT: &str = "20260427T120000Z-aaaa";

pub(super) fn step_dir(ws: &Path, seq: &str) -> PathBuf {
    ws.join("steps").join(AGENT).join(seq)
}

/// Write one per-step record file, creating the step dir.
pub(super) fn write_file(ws: &Path, seq: &str, name: &str, bytes: &[u8]) {
    let dir = step_dir(ws, seq);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), bytes).unwrap();
}

/// Write one tool call's `input.json` (always) and `output.json` (when given).
pub(super) fn write_tool(ws: &Path, seq: &str, tool_id: &str, input: &[u8], output: Option<&[u8]>) {
    let dir = step_dir(ws, seq).join("tools").join(tool_id);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("input.json"), input).unwrap();
    if let Some(bytes) = output {
        std::fs::write(dir.join("output.json"), bytes).unwrap();
    }
}
