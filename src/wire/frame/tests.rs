//! The framing, both directions and both refusals.

use super::*;
use serde_json::json;

/// A frame round-trips its value, and the terminator reads as the end.
#[test]
fn a_stream_is_frames_then_a_terminator() {
    let mut buf: Vec<u8> = Vec::new();
    write_value(&mut buf, &json!({"ok": true})).expect("write");
    write_value(&mut buf, &json!({"ok": false, "error": "no"})).expect("write");
    write_end(&mut buf).expect("end");
    let mut read: &[u8] = &buf;
    assert_eq!(
        read_value(&mut read).expect("first"),
        Some(json!({"ok": true}))
    );
    assert_eq!(
        read_value(&mut read).expect("second"),
        Some(json!({"ok": false, "error": "no"}))
    );
    assert_eq!(read_value(&mut read).expect("terminator"), None);
}

/// The header is a big-endian `u32` byte count: the bytes on the wire say so,
/// not a comment about them.
#[test]
fn the_header_is_a_big_endian_byte_count() {
    let mut buf: Vec<u8> = Vec::new();
    write_value(&mut buf, &json!(1)).expect("write");
    assert_eq!(buf, vec![0, 0, 0, 1, b'1']);
}

/// A payload's own bytes are never a delimiter — the property a scanning
/// reader cannot have. A JSON string full of newlines and NULs frames exactly.
#[test]
fn a_payload_cannot_forge_a_frame() {
    let hostile = json!({"text": "\n\u{0}\n{\"ok\":true}\n"});
    let mut buf: Vec<u8> = Vec::new();
    write_value(&mut buf, &hostile).expect("write");
    write_end(&mut buf).expect("end");
    let mut read: &[u8] = &buf;
    assert_eq!(read_value(&mut read).expect("frame"), Some(hostile));
    assert_eq!(read_value(&mut read).expect("terminator"), None);
}

/// A length above the limit is refused on its header — before the allocation
/// it asks for is made.
#[test]
fn an_oversized_length_is_refused_on_its_header() {
    let mut header: Vec<u8> = Vec::new();
    header.extend_from_slice(&u32::MAX.to_be_bytes());
    let mut read: &[u8] = &header;
    let err = read_value(&mut read).expect_err("refused");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("exceeds"), "{err}");
}

/// And a body too large to write is refused rather than written.
#[test]
fn an_oversized_body_is_never_written() {
    let mut buf: Vec<u8> = Vec::new();
    let err = write_bytes(&mut buf, &vec![b'x'; MAX_FRAME + 1]).expect_err("refused");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(buf.is_empty(), "nothing was written");
}

/// A frame whose body is not JSON is an error, not a silent skip: the strict
/// decode discipline the codec keeps, at the framing.
#[test]
fn a_body_that_is_not_json_refuses() {
    let mut buf: Vec<u8> = Vec::new();
    write_bytes(&mut buf, b"{").expect("write");
    let mut read: &[u8] = &buf;
    let err = read_value(&mut read).expect_err("refused");
    assert!(err.to_string().contains("not JSON"), "{err}");
}

/// A truncated stream is an error, never a short value.
#[test]
fn a_truncated_stream_refuses() {
    let mut read: &[u8] = &[0, 0, 0, 4, b'{'];
    read_value(&mut read).expect_err("refused");
    let mut short: &[u8] = &[0, 0];
    read_value(&mut short).expect_err("refused");
}
