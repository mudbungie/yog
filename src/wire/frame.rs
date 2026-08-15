//! The wire's framing (REMOTE §3, decided by bl-b6fa): **a big-endian `u32`
//! byte length followed by that many bytes of JSON, and a zero-length frame
//! ends a reply stream.**
//!
//! §3 left the choice to the implementation ball and named two candidates. This
//! is the length-delimited one, for three reasons and one that matters more
//! than all of them:
//!
//! - A reader never scans. It reads four bytes, then exactly that many — so a
//!   payload's own bytes can never be mistaken for a delimiter, and no property
//!   of the *encoder* (that `serde_json` escapes newlines, say) is load-bearing
//!   in the *framing*.
//! - The allocation is bounded before it is made. A length above [`MAX_FRAME`]
//!   is refused on its header, so a hostile peer on the open internet cannot
//!   make a reader grow to meet it.
//! - The terminator is unambiguous by construction: a zero-length frame is not
//!   a JSON value, so nothing a payload can say collides with it.
//!
//! **And the streaming form is not a second form.** §3 asks for "one streaming
//! form for follow-class reads, same envelope, chunked". Here *every* answer is
//! a stream: a request is one frame, and its answer is N ≥ 1 reply frames
//! followed by the terminator. Today N is always 1 because no [`Query`] is
//! follow-class — the seat polls (§3) — so a follow-class read is the general
//! path with more than one frame in it, not a case of its own, and landing one
//! needs no version, no flag and no second reader. That is the same
//! edge-case-dissolving move §8.5 makes for a gesture that names no workspace.

use serde_json::Value;
use std::io::{self, Read, Write};

/// The largest frame either end will write or read: 16 MiB. A reply is JSON
/// derived from a world's own files, so this is orders above anything yog
/// says, and it is the bound that makes an unauthenticated-but-handshaken
/// peer unable to dictate an allocation.
pub const MAX_FRAME: usize = 16 * 1024 * 1024;

/// The frame header's width — a big-endian `u32`.
const HEADER: usize = 4;

/// Write one JSON frame.
pub fn write_value(w: &mut dyn Write, v: &Value) -> io::Result<()> {
    write_bytes(w, serde_json::to_string(v)?.as_bytes())
}

/// Write the end-of-stream terminator: a zero-length frame.
pub fn write_end(w: &mut dyn Write) -> io::Result<()> {
    write_bytes(w, &[])
}

/// Read one JSON frame: `Some(value)` a frame, `None` the terminator. An
/// oversized length, a short stream or a body that is not JSON is an error —
/// the strict-decode discipline the codec already keeps, at the framing.
pub fn read_value(r: &mut dyn Read) -> io::Result<Option<Value>> {
    let Some(body) = read_bytes(r)? else {
        return Ok(None);
    };
    serde_json::from_slice(&body).map(Some).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame is not JSON: {e}"),
        )
    })
}

/// The length-prefixed write both spellings above share.
fn write_bytes(w: &mut dyn Write, body: &[u8]) -> io::Result<()> {
    if body.len() > MAX_FRAME {
        return Err(oversize(body.len()));
    }
    // Infallible after the bound above, and deliberately not a `try_from`: a
    // fallible conversion whose error arm cannot be reached is an untestable
    // branch, and this file has no untestable branches.
    let len = body.len() as u32;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(body)?;
    w.flush()
}

/// The length-prefixed read: `None` for the zero-length terminator.
fn read_bytes(r: &mut dyn Read) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; HEADER];
    r.read_exact(&mut header)?;
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 {
        return Ok(None);
    }
    if len > MAX_FRAME {
        return Err(oversize(len));
    }
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    Ok(Some(body))
}

/// The one refusal a length can earn, said the same way on both sides.
fn oversize(len: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("frame of {len} bytes exceeds the {MAX_FRAME}-byte limit"),
    )
}

#[cfg(test)]
mod tests;
