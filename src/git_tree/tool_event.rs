//! **One opening or closing of the tool window** (REMOTE §5.5, bl-5305): the
//! datum the follow lane carries beside the model's prose, read off the
//! per-call record litany lands, and its JSON spelling.
//!
//! It sits beside [`Stream`](super::Stream) and for that type's reason exactly:
//! the shape of a fact read off the workspace's disk is this module's
//! vocabulary, and the boundary names it rather than restating it. What the
//! *follow lane* adds — which calls a given read has already spoken about — is
//! that lane's own ([`boundary::follow::tools`](crate::boundary::follow)),
//! exactly as [`Open`](crate::boundary::follow) is the lane's half of the fold.
//!
//! **Two events per call come off disk, and both are transitions.** litany
//! lands `input.json` immediately before it dispatches a call and `output.json`
//! when the capture returns (litany ARCH §3.3), so the pair of file existences
//! *is* the window opening and closing. A call's standing is what the two files
//! say the moment it is asked; nothing is stored.
//!
//! **The third transition never reaches disk, because it is what stops the call
//! from being dispatched** (bl-58bb). When the capability control answers
//! `hold`, litany's seam parks the invocation *before* the executor is entered
//! — so no `input.json` is landed and the window's two files say nothing at
//! all. The park's one record is the hold mark
//! ([`control::hold`](crate::control::hold)), which carries the same three
//! facts an opening event does: the `tool_use` id, the tool the model named,
//! and the control's reason. [`parked`] is that mark read as an event, so the
//! lane has one vocabulary for *what this call is doing* rather than a second
//! carrier beside it.
//!
//! **The client the call ran on rides in the name and is not joined here.** A
//! loaded remote tool is presented as `<client>_<tool>`
//! ([`loaded::Entry::presented`](crate::tool_host::loaded::Entry::presented)),
//! always and never only when ambiguous (REMOTE §5.1), so the name litany
//! recorded already says which machine the call was routed to. Splitting that
//! composition back apart would be a second reader of a rule with one home, and
//! asking the registry instead would answer *where it would route now* rather
//! than where it ran.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::boundary::codec::fields::{opt_str_of, str_of};

/// Filenames of the per-call record (litany ARCH §3.3). [`super::tools`] names
/// them too and reads them for a different question — what the latest step is
/// running, on the worker's cadence, for every agent at once — so the two
/// readers share the directory convention and nothing else.
const INPUT_FILE: &str = "input.json";
const OUTPUT_FILE: &str = "output.json";
/// The step directory's tool subtree.
pub(crate) const TOOLS_SUBDIR: &str = "tools";

/// How much of a call's input one event carries. A tool input is arbitrary JSON
/// — a whole file body is an ordinary write argument — and this lane exists to
/// be read *while* it streams, so the summary is bounded exactly as the §4.2
/// trail's argv summary is, through [`crate::elide::middle`]: the cut keeps
/// both ends, because the head of a command line says which command and the
/// tail says which invocation of it.
const INPUT_MAX: usize = 240;

/// One opening or closing of the tool window (REMOTE §5.5).
///
/// **The capture's status is the exit code's presence, not a second field.**
/// Absent is a call in flight and present is one whose capture landed, so the
/// two readings cannot disagree and there is no arm for "complete, status
/// unknown".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolEvent {
    /// The provider's `tool_use` id — the call's identity, and the directory
    /// name the record was read from.
    pub tool_use: String,
    /// What the tool is called, verbatim as litany recorded it. A routed call's
    /// name carries its client (module doc); `None` for a record with no
    /// readable name, the same partial-write tolerance every other reader of
    /// these files has, and for the closing event, which restates nothing.
    pub name: Option<String>,
    /// The call's input, rendered on one line and bounded by [`INPUT_MAX`] —
    /// the command line for a shell-shaped tool, the argument object for any
    /// other. One rule and no per-tool table: a table saying which key is "the
    /// command" would be a second home for what each tool's schema already
    /// says, and it would be silent on the tool nobody thought of.
    pub input: Option<String>,
    /// The captured exit code, and the whole of what "closed" means.
    pub exit_code: Option<i32>,
    /// **Why the control parked this call** (bl-58bb) — the reason sentence off
    /// the hold mark, and, like [`exit_code`](ToolEvent::exit_code), a status
    /// by its presence rather than by a flag beside it. A held call has no
    /// `input.json`, so it is never also an opening; when the operator answers
    /// it, the call runs and its opening and closing follow under this same
    /// `tool_use` id.
    pub held: Option<String>,
}

/// The call directories under one step's `tools/`, keyed by `tool_use` id and
/// ordered by it — the provider mints those ids in call order and `read_dir` is
/// unordered, which is the ordering rule [`super::tools`] already applies. A
/// directory with no `input.json` is a call still being landed, not a call; an
/// absent or unreadable `tools/` is an empty listing, which is the general path
/// with no input rather than an error with an opinion.
pub(crate) fn calls(step_dir: &Path) -> BTreeMap<String, PathBuf> {
    let Ok(entries) = std::fs::read_dir(step_dir.join(TOOLS_SUBDIR)) else {
        return BTreeMap::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let dir = entry.path();
            let id = entry.file_name().to_str()?.to_owned();
            dir.join(INPUT_FILE).exists().then_some((id, dir))
        })
        .collect()
}

/// Whether this call's capture has landed — the closing half of the window.
pub(crate) fn captured(dir: &Path) -> bool {
    dir.join(OUTPUT_FILE).exists()
}

/// **The window parked** (bl-58bb): the call the capability control held, off
/// the mark litany's seam wrote instead of dispatching it.
///
/// It carries the mark's three fields and adds nothing. The reason is the
/// control's own sentence — the tool, an input summary, the computed class and
/// the evidence — so this event says what an opening says *and* why nothing
/// ran, in the text the attention item and the agent row already show.
pub(crate) fn parked(held: &crate::control::hold::Held) -> ToolEvent {
    ToolEvent {
        tool_use: held.tool_use_id.clone(),
        name: Some(held.tool.clone()),
        held: Some(held.reason.clone()),
        ..ToolEvent::default()
    }
}

/// The window **opening**: what is about to run, and — through the name — where.
pub(crate) fn posted(tool_use: &str, dir: &Path) -> ToolEvent {
    let record = read_json(&dir.join(INPUT_FILE));
    ToolEvent {
        tool_use: tool_use.to_owned(),
        name: record
            .as_ref()
            .and_then(|v| v.get("name")?.as_str())
            .map(str::to_owned),
        input: record
            .as_ref()
            .and_then(|v| v.get("input"))
            .map(|input| crate::elide::middle(&input.to_string(), INPUT_MAX)),
        exit_code: None,
        held: None,
    }
}

/// The window **closing**: the same identity, and the status the capture landed
/// with. It restates neither the name nor the input — a follower holds the
/// opening event those rode on, keyed by the same id, and a second copy of one
/// fact is exactly what this lane's byte budget was cut for (bl-3655).
///
/// A record whose `exit_code` is unreadable answers `0`: the file's presence is
/// what "the capture landed" means, and a closing event carrying no status at
/// all would be indistinguishable from the opening one.
pub(crate) fn complete(tool_use: &str, dir: &Path) -> ToolEvent {
    let code = read_json(&dir.join(OUTPUT_FILE))
        .as_ref()
        .and_then(|v| v.get("exit_code")?.as_i64())
        .and_then(|c| i32::try_from(c).ok())
        .unwrap_or_default();
    ToolEvent {
        tool_use: tool_use.to_owned(),
        exit_code: Some(code),
        ..ToolEvent::default()
    }
}

/// One record, or `None` for a file that is absent, unreadable or caught
/// mid-write — the same tolerance every other reader of a litany record has.
fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

/// One event's JSON spelling. `tool` and `input` are absent on a closing event
/// and on a record that carried neither; `exit_code` is absent for a call in
/// flight, and `held` for one no control parked — each the fact itself and not
/// an omission.
pub fn event_value(event: &ToolEvent) -> Value {
    let mut map = Map::new();
    map.insert("tool_use".to_owned(), json!(event.tool_use));
    if let Some(name) = &event.name {
        map.insert("tool".to_owned(), json!(name));
    }
    if let Some(input) = &event.input {
        map.insert("input".to_owned(), json!(input));
    }
    if let Some(code) = event.exit_code {
        map.insert("exit_code".to_owned(), json!(code));
    }
    if let Some(why) = &event.held {
        map.insert("held".to_owned(), json!(why));
    }
    Value::Object(map)
}

/// Read one back. Strict on `tool_use`, which is the identity a follower keys
/// on, and on a present field of the wrong shape; forgiving of an absent one,
/// which is a reading (module doc).
pub fn event_of(v: &Value) -> Result<ToolEvent, String> {
    let o = v.as_object().ok_or("follow: tool event is not an object")?;
    Ok(ToolEvent {
        tool_use: str_of(o, "tool_use")?,
        name: opt_str_of(o, "tool")?,
        input: opt_str_of(o, "input")?,
        exit_code: match o.get("exit_code") {
            None | Some(Value::Null) => None,
            Some(v) => Some(
                v.as_i64()
                    .and_then(|c| i32::try_from(c).ok())
                    .ok_or("follow: exit_code is not an i32")?,
            ),
        },
        held: opt_str_of(o, "held")?,
    })
}

#[cfg(test)]
mod tests;
