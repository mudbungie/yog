//! The on-demand drill-in tier (DESIGN §11 Altitude 2): one selected step's
//! record files, parsed into [`Doc`]s for the jsonview trees. Split from
//! [`super`] — the cheap per-step summary list — along the boundary that
//! module's own doc already names, so neither tier approaches the 300-line cap
//! (§12).

use std::path::Path;

use serde_json::Value;

use super::records::{DRIVER_LOG_FILE, STDERR_FILE};
use super::{
    INPUT_FILE, META_FILE, OUTPUT_FILE, REQUEST_FILE, RESPONSE_FILE, STAGING_FILE, STEPS_DIR,
    TOOLS_SUBDIR,
};
use crate::files_view::Preview;

/// The sentence yog renders above an unparseable record — the §11 "error row"
/// for this class, held beside [`NO_RESPONSE`] for the same reason: one
/// wording, rendered verbatim, assertable by test.
pub const UNPARSED: &str = "unparseable JSON — bytes verbatim below";

/// A drill-in document, derived at read time and stored nowhere (§3.5): the
/// three things a record file can be. Nothing is summarized away — the bytes
/// of an [`Unparsed`](Doc::Unparsed) doc still render verbatim (§11) — but the
/// reader is *told* it is malformed, because rendered bare it is
/// indistinguishable from a file whose content happens to be that text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Doc {
    /// Parsed — rendered as a jsonview tree, with the bytes it parsed from
    /// kept beside it for the §11 Raw toggle. A `serde_json::Value` is not a
    /// lossless record of its source (key order, spacing and number spelling
    /// all go), so the tree alone could never answer "what does the file
    /// say" — the promise is the file's bytes *unaltered* (S7-T1).
    Json { value: Value, raw: Vec<u8> },
    /// No bytes at all: the file is missing, unreadable, or empty.
    Absent,
    /// Bytes that are not JSON — kept verbatim under the [`UNPARSED`] row.
    Unparsed(Vec<u8>),
}

impl Doc {
    /// Classify bytes: empty is [`Absent`](Doc::Absent), parsing is
    /// [`Json`](Doc::Json), anything else is [`Unparsed`](Doc::Unparsed). The
    /// emptiness split lives here, not in the renderer, so "absent" and
    /// "malformed" are distinct facts in the view-model rather than a shape
    /// the paint code re-derives.
    pub(super) fn of_bytes(bytes: Vec<u8>) -> Doc {
        if bytes.is_empty() {
            return Doc::Absent;
        }
        match serde_json::from_slice(&bytes) {
            Ok(value) => Doc::Json { value, raw: bytes },
            Err(_) => Doc::Unparsed(bytes),
        }
    }

    /// The record's backing bytes — what the §11 Raw toggle shows. Empty iff
    /// the record is [`Absent`](Doc::Absent), which is the same fact said
    /// twice only if you spell it twice: the renderer reads emptiness here
    /// rather than re-matching the variant.
    pub(super) fn raw(&self) -> &[u8] {
        match self {
            Doc::Json { raw, .. } | Doc::Unparsed(raw) => raw,
            Doc::Absent => &[],
        }
    }

    /// Read a file into a [`Doc`]; a missing/unreadable file is
    /// [`Absent`](Doc::Absent).
    fn of_file(path: &Path) -> Doc {
        Doc::of_bytes(std::fs::read(path).unwrap_or_default())
    }
}

/// One tool call's on-disk records (ARCH §3.3). `is_error` mirrors lernie's
/// own `is_error = exit_code != 0` (`spawn.rs`) — derived from
/// `output.json`'s `exit_code`, `false` when output is absent or carries no
/// exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolIo {
    pub tool_id: String,
    pub input: Doc,
    pub output: Doc,
    pub is_error: bool,
}

/// One step's drill-in: the four record files as jsonview docs, `response
/// .json` split per JSONL event, every tool call's input/output, and the two
/// capture logs when they carry anything. Built on demand for the selected step
/// only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepDetail {
    pub seq: String,
    pub meta: Doc,
    pub request: Doc,
    pub staging: Doc,
    /// `response.json` per line — each event a jsonview tree, a malformed
    /// line kept raw.
    pub response: Vec<Doc>,
    pub tools: Vec<ToolIo>,
    /// This step's `stderr.log` as bounded bytes — the adapter's own words in
    /// full, where the §7.3 wound banner quotes three lines of them (bl-83d6).
    /// `None` is a file with nothing in it, which is the ordinary run: the
    /// picker then seats no `stderr.log` row at all, so the absence is one fact
    /// with one encoding (`Some(Preview::Text(""))` would be a second).
    pub stderr: Option<Preview>,
    /// The **agent's** `driver.log` as bounded bytes — not this step's file, the
    /// conversation's, read here because the drill-in is the surface that shows
    /// a whole file and this is the only tier built on demand. `None` when it
    /// has nothing in it, exactly as `stderr` above.
    pub driver: Option<Preview>,
}

/// Build the drill-in for one step of `agent_id`. Every file is read
/// forgivingly — malformed content keeps its bytes and is *framed* as
/// malformed ([`UNPARSED`]), absent content says so; neither aborts the build,
/// so the sibling tabs still render (S7-T2).
pub fn detail(workspace: &Path, agent_id: &str, seq: &str) -> StepDetail {
    let agent = workspace.join(STEPS_DIR).join(agent_id);
    let step = agent.join(seq);
    StepDetail {
        seq: seq.to_string(),
        meta: Doc::of_file(&step.join(META_FILE)),
        request: Doc::of_file(&step.join(REQUEST_FILE)),
        staging: Doc::of_file(&step.join(STAGING_FILE)),
        response: response_events(&step.join(RESPONSE_FILE)),
        tools: tool_ios(&step.join(TOOLS_SUBDIR)),
        stderr: log(&step.join(STDERR_FILE)),
        driver: log(&agent.join(DRIVER_LOG_FILE)),
    }
}

/// A capture log as bounded bytes, or `None` when there is nothing to read —
/// absent, unstattable, or empty, which are one fact for a file nobody promised
/// to write (bl-83d6). This single read **is** the picker's presence rule
/// ([`super::records::seats`]): nothing stats these files twice, so a seat can
/// never be offered over bytes that are not there.
///
/// The bound is [`crate::files_view::preview`]'s, not a new one: 64 KiB of a
/// file with the cap said outright, a NUL-bearing capture declared binary
/// rather than mangled. A driver that chattered for hours is exactly the case
/// that bound exists for, and the §7.3 banners' three-line tail
/// ([`crate::opslog::rows::stderr_tail`]) is the *other* bound — how much a
/// one-line sentence quotes, not how much a reading surface shows.
fn log(path: &Path) -> Option<Preview> {
    let size = std::fs::symlink_metadata(path).ok()?.len();
    (size > 0).then(|| crate::files_view::preview(path))
}

/// Split `response.json` into per-line docs (empty lines dropped). A missing
/// file yields no events.
fn response_events(path: &Path) -> Vec<Doc> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    bytes
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| Doc::of_bytes(line.to_vec()))
        .collect()
}

/// One [`ToolIo`] per `<tool-id>/` subdir, sorted by tool-id (the wire id is
/// monotone in call order). A missing `tools/` dir yields none.
fn tool_ios(tools_dir: &Path) -> Vec<ToolIo> {
    let Ok(entries) = std::fs::read_dir(tools_dir) else {
        return Vec::new();
    };
    let mut tools: Vec<ToolIo> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let tool_id = entry.file_name().to_str()?.to_string();
            let output = Doc::of_file(&path.join(OUTPUT_FILE));
            let is_error = output_is_error(&output);
            Some(ToolIo {
                tool_id,
                input: Doc::of_file(&path.join(INPUT_FILE)),
                output,
                is_error,
            })
        })
        .collect();
    tools.sort_by(|a, b| a.tool_id.cmp(&b.tool_id));
    tools
}

/// Did the tool exit non-zero? Reads `output.json`'s `exit_code` (§3.3
/// `{stdout, stderr, exit_code, …}`); a raw/absent output or missing code is
/// not an error.
fn output_is_error(output: &Doc) -> bool {
    match output {
        Doc::Json { value, .. } => value
            .get("exit_code")
            .and_then(Value::as_i64)
            .is_some_and(|code| code != 0),
        Doc::Absent | Doc::Unparsed(_) => false,
    }
}
