//! Tests for the steps inspector.
//!
//! [`vm`] drives [`super::build`] against tempdir-backed `steps/<agent>/NNN/`
//! trees, covering enumeration order, the reused framing/attempts/token
//! derivations, forgiving parsing and the §7.1 login routing; [`detail`] drives
//! [`super::detail`] over the same trees for the drill-in — the two split at
//! §12's budget on the read seam the production modules are cut on.
//! [`logs`] drives the two capture-log seats (bl-83d6) — the derived row set
//! and the whole of a long log in the paint output.
//! [`render`] shape-walks the table headlessly per the transcript pattern with
//! [`drill`] doing the same for the record trees below it (the two cut at §12's
//! budget on the `render`/`drill` seam the paints themselves have),
//! [`raw`] holds the §11 Raw toggle's half of that walk (S7-T1: the record
//! file's bytes unaltered), and [`wound`] drives the §7.3 no-response state
//! from the shape the real substrate left behind, all the way to the painted
//! sentence, with [`truncation`] doing the same for the wound's other class
//! (the §4.4 output limit, bl-fb87). [`tail`] does the render walk on a viewport too short for the
//! table, where the §11 tail anchor decides which steps are on screen.
//! [`orphan`] and [`window`] drive the two shapes of the §7.3 orphaned-tail
//! state — delivered mail nobody answers (bl-ace6) and a tool window an
//! executor died inside (bl-abba) — split at §12's budget on that seam.
//! [`catch_up`] drives the §7.2 window the engine waits out before it states a
//! no-response wound at all (bl-776a), which is the one part of the derivation
//! that takes a clock.

use std::path::{Path, PathBuf};

mod catch_up;
mod detail;
mod logs;
mod orphan;
mod truncation;
mod vm;
mod window;
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
