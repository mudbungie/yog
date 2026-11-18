//! **A capture is N ≥ 1 posts** (REMOTE §5.7, bl-d414): the vocabulary of the
//! streamed form, and its JSON spellings.
//!
//! Split from [`super`] at §12's per-file budget, on the seam the wire itself
//! has: [`super`] says what an invocation and a finished capture *are*, and
//! this says how one arrives in pieces — the piece a post appends, the offsets
//! an asker holds, and the slot as either end is answered it.
//!
//! **It is not a second form.** A foot that posts once, with a capture, and an
//! asker that states no offsets spend none of this and read exactly what they
//! read before it existed; every key here is one a reader may ignore, which is
//! what makes the three of them an edition and not a `PROTOCOL` bump (§3.2).

use serde_json::{Map, Value, json};

use super::{Capture, capture_of, capture_value};
use crate::boundary::codec::fields::{opt_val, str_of, u64_of};

/// The two envelope keys this file owns, named once for both directions.
pub const PARTIAL: &str = "partial";
pub const SINCE: &str = "since";

/// **Text on the two streams**, with no verdict: what one post appends, and
/// what an asker has not read yet. Either string may be empty, and two empty
/// ones say only *still running*.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Piece {
    pub stdout: String,
    pub stderr: String,
}

/// **How much an asker already holds** — its own byte count per stream. An
/// idempotent ask (REMOTE §3), and the acknowledgement that lets the server
/// drop what was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Offsets {
    pub stdout: u64,
    pub stderr: u64,
}

/// **What one `complete` carries**: text and *the run is not over*, or the
/// capture that ends it — whose text is what was not yet posted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Post {
    Partial(Piece),
    Capture(Capture),
}

/// **One slot as it stands after a call** — the body of
/// [`Reply::Routed`](crate::boundary::reply::Reply::Routed) beside the handle.
/// `partial` present, empty included, says a machine has posted for the slot;
/// `capture` present says the run is over. Both absent is *queued, or running
/// on a foot that has said nothing yet*.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Standing {
    pub partial: Option<Piece>,
    pub capture: Option<Capture>,
}

/// A piece as JSON — the one spelling, on the post and on the reply alike.
pub fn piece_value(piece: &Piece) -> Value {
    json!({ "stdout": piece.stdout, "stderr": piece.stderr })
}

/// [`piece_value`]'s inverse, strict: both streams are stated or it refuses.
pub fn piece_of(v: &Value) -> Result<Piece, String> {
    let o = v.as_object().ok_or("partial: not a JSON object")?;
    Ok(Piece {
        stdout: str_of(o, "stdout")?,
        stderr: str_of(o, "stderr")?,
    })
}

/// Offsets as JSON.
pub fn offsets_value(since: Offsets) -> Value {
    json!({ "stdout": since.stdout, "stderr": since.stderr })
}

/// [`offsets_value`]'s inverse, on the same strict terms.
pub fn offsets_of(v: &Value) -> Result<Offsets, String> {
    let o = v.as_object().ok_or("since: not a JSON object")?;
    Ok(Offsets {
        stdout: u64_of(o, "stdout")?,
        stderr: u64_of(o, "stderr")?,
    })
}

/// A post under its envelope key: `partial` or `capture`, never both.
pub fn post_into(map: &mut Map<String, Value>, post: &Post) {
    let (key, value) = match post {
        Post::Partial(piece) => (PARTIAL, piece_value(piece)),
        Post::Capture(capture) => ("capture", capture_value(capture)),
    };
    map.insert(key.to_owned(), value);
}

/// [`post_into`]'s inverse. `capture` is read first, so today's foot is read
/// exactly as it was; an envelope stating neither refuses in the old words.
pub fn post_of(o: &Map<String, Value>) -> Result<Post, String> {
    match (o.get("capture"), o.get(PARTIAL)) {
        (Some(capture), _) => capture_of(capture).map(Post::Capture),
        (None, Some(partial)) => piece_of(partial).map(Post::Partial),
        (None, None) => Err("complete: missing capture".to_owned()),
    }
}

/// A standing's keys, each **absent rather than empty** when it has nothing
/// to say — a reader must not have to tell "nobody posted" from "posted
/// nothing", nor "not finished" from "finished saying nothing".
pub fn standing_into(map: &mut Map<String, Value>, standing: &Standing) {
    if let Some(piece) = &standing.partial {
        map.insert(PARTIAL.to_owned(), piece_value(piece));
    }
    if let Some(capture) = &standing.capture {
        map.insert("capture".to_owned(), capture_value(capture));
    }
}

/// [`standing_into`]'s inverse.
pub fn standing_of(o: &Map<String, Value>) -> Result<Standing, String> {
    Ok(Standing {
        partial: opt_val(o, PARTIAL, piece_of)?,
        capture: opt_val(o, "capture", capture_of)?,
    })
}

/// The asker's offsets under their envelope key, when it states any.
pub fn since_into(map: &mut Map<String, Value>, since: Option<Offsets>) {
    if let Some(since) = since {
        map.insert(SINCE.to_owned(), offsets_value(since));
    }
}

/// [`since_into`]'s inverse: absent is the read exactly as it always was.
pub fn since_of(o: &Map<String, Value>) -> Result<Option<Offsets>, String> {
    opt_val(o, SINCE, offsets_of)
}
