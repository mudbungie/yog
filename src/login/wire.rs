//! The sign-in standing's **one JSON spelling**, both directions (REMOTE §8.3,
//! bl-c285) — beside its own type, for the reason every other `wire` module is:
//! a [`LoginView`]'s shape *is* this module's vocabulary, and the boundary's
//! reply codec names the encoder rather than keeping a second copy of it.
//!
//! One shape serves the act's receipt and every lane frame, because they are
//! the same value read at two moments: the receipt is the standing just after
//! the spawn, and a frame is what the standing gained since the frame before it
//! (REMOTE §5.5's append rule, at this subject). So a seat folds frames by
//! extending `lines`, and the frame carrying an `outcome` is the last one.
//!
//! `outcome` and `fallback` are **absent rather than null** while the run is
//! live: a reader must not have to tell "still signing in" from "finished
//! saying nothing", and the fallback exists only for a non-zero exit (§8.3).

use serde_json::{Map, Value, json};

use super::LoginView;
use crate::boundary::codec::fields::{i64_of, list_of, opt, opt_str_of, str_of};
use crate::cli_outbound::StreamedLine;

/// The reply body: the lines this frame carries, each tagged with the stream it
/// came from (§8.3 rule 3 — bz writes the authorize URL to stderr), and the
/// terminal facts once there are any.
pub(crate) fn body(view: &LoginView) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert(
        "lines".to_owned(),
        json!(
            view.lines
                .iter()
                .map(|line| json!({ "text": line.text, "err": line.err }))
                .collect::<Vec<Value>>()
        ),
    );
    if let Some(exit) = view.outcome {
        map.insert("outcome".to_owned(), json!(exit));
    }
    if let Some(command) = &view.fallback {
        map.insert("fallback".to_owned(), json!(command));
    }
    map
}

/// The same body read back — strict, like every reply decode: a line that is
/// not an object, or an exit no `i32` can hold, refuses by name rather than
/// shortening the frame.
pub(crate) fn view_of(o: &Map<String, Value>) -> Result<LoginView, String> {
    Ok(LoginView {
        lines: list_of(o, "lines", line_of)?,
        outcome: opt(o, "outcome", exit_of)?,
        fallback: opt_str_of(o, "fallback")?,
    })
}

fn line_of(v: &Value) -> Result<StreamedLine, String> {
    let o = v.as_object().ok_or("login: line is not an object")?;
    Ok(StreamedLine {
        text: str_of(o, "text")?,
        err: o
            .get("err")
            .and_then(Value::as_bool)
            .ok_or("login: missing or non-boolean field \"err\"")?,
    })
}

fn exit_of(o: &Map<String, Value>, key: &str) -> Result<i32, String> {
    i32::try_from(i64_of(o, key)?).map_err(|_| format!("field {key:?} out of range"))
}
