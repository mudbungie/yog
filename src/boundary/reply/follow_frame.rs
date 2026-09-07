//! **One frame of the live tail**, and its spelling both ways (REMOTE §3,
//! §5.5; bl-73e7, bl-3655, bl-5305) — its own file on `config_view`'s seam:
//! the one reply whose body is two folds rather than one value, said where the
//! two are named together.
//!
//! **Two halves, one rule.** The prose half is
//! [`Stream`](crate::git_tree::Stream), the fold of what the model wrote; the
//! other is the **tool window** — [`ToolEvent`](crate::git_tree::ToolEvent)s
//! as a call is posted and as its capture lands. Both are *appends* under
//! REMOTE §5.5's ruling, which this file implements and does not restate:
//! absorb every frame of a read, in order, onto an empty fold, and concatenate
//! the event lists the same way.
//!
//! **Why the second half exists** (bl-5305). The lane carried the model's prose
//! alone, so a conversation that spent its life running commands on two enrolled
//! boxes produced a thinking marker and two sentences while eight commands ran.
//! For a conversation editing its own worktree that omission is defensible — the
//! work is on disk and a diff is a read away. For one administering a MACHINE it
//! is the wrong one: the box is precisely the thing an operator cannot inspect
//! afterwards, and the tool call is the only record that it was touched.
//! `Query::Transcript` recorded every capture in full the whole time; it was the
//! live view that dropped them, and the live view is the one an operator has
//! open when a command they did not expect is about to run on their server.

use serde_json::{Map, Value, json};

use crate::git_tree::{Stream, ToolEvent, stream_wire, tool_event};

/// What one follow frame carries. Both fields default to empty, which is the
/// honest answer for a conversation with nothing in flight rather than a case
/// of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FollowFrame {
    /// The prose that landed since the read's previous frame.
    pub stream: Stream,
    /// The tool window's transitions discovered since it.
    pub tools: Vec<ToolEvent>,
}

/// The whole reply value. **`tools` is written always, empty included**: a
/// frame that omitted it would make "this build has no tool window" and
/// "nothing ran since the last frame" one shape — `reply/advertised`'s `wrote`
/// argument at PROTOCOL 8, said again.
pub(super) fn reply(frame: &FollowFrame) -> Value {
    json!({
        "ok": true,
        "kind": super::encode::FOLLOW,
        "stream": stream_wire::stream_value(&frame.stream),
        "tools": frame.tools.iter().map(tool_event::event_value).collect::<Vec<_>>(),
    })
}

/// Read one back. A frame with no `stream` object at all is a codec that has
/// drifted, not an empty tail — an empty tail is an empty object, which reads
/// as [`Stream::default`] — and `tools` is required for that reason exactly:
/// absent would read as *nothing ran* on precisely the build that cannot say.
pub(super) fn frame_of(o: &Map<String, Value>) -> Result<FollowFrame, String> {
    let body = o.get("stream").ok_or("follow: missing stream")?;
    let body = body.as_object().ok_or("follow: stream is not an object")?;
    Ok(FollowFrame {
        stream: stream_wire::stream_of(body)?,
        tools: crate::boundary::codec::fields::list_of(o, "tools", tool_event::event_of)?,
    })
}
