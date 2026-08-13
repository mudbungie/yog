//! The streamed-piped class ([`Streamed`], §8): line buffering (partial lines,
//! blank lines, the bounded-length truncation), the per-stream tag both
//! consumers project through, and the terminal outcome. Driven deterministically
//! through [`Stream::from_rx`] — chunks sent by hand, no real process to race —
//! so every `poll` arm is covered without timing flake; the real
//! spawn→disconnect path is covered by the S0-T5 login story.

use super::*;

/// A stdout-tagged line.
fn out(text: &str) -> StreamedLine {
    StreamedLine {
        text: text.to_owned(),
        err: false,
    }
}

/// A stderr-tagged line.
fn err(text: &str) -> StreamedLine {
    StreamedLine {
        text: text.to_owned(),
        err: true,
    }
}

/// A [`Streamed`] over a fresh channel plus the sender to feed it by hand.
fn wired() -> (Streamed, mpsc::Sender<Chunk>) {
    let (tx, rx) = mpsc::channel();
    (Streamed::new(Stream::from_rx(rx)), tx)
}

#[test]
fn poll_is_pending_when_nothing_is_buffered() {
    let (mut s, _tx) = wired();
    assert_eq!(s.poll(), StreamedPoll::Pending);
}

#[test]
fn poll_buffers_a_partial_line_until_its_newline() {
    let (mut s, tx) = wired();
    // A line split across two reads: the first chunk holds no newline yet.
    tx.send(Chunk::Stdout(b"ab".to_vec())).unwrap();
    assert_eq!(s.poll(), StreamedPoll::Pending);
    tx.send(Chunk::Stdout(b"c\n".to_vec())).unwrap();
    assert_eq!(s.poll(), StreamedPoll::Lines(vec![out("abc")]));
}

#[test]
fn poll_splits_multiple_and_blank_lines_verbatim() {
    let (mut s, tx) = wired();
    tx.send(Chunk::Stdout(
        b"code: WXYZ\n\nopen https://x/device\n".to_vec(),
    ))
    .unwrap();
    assert_eq!(
        s.poll(),
        StreamedPoll::Lines(vec![
            out("code: WXYZ"),
            out(""),
            out("open https://x/device")
        ])
    );
}

#[test]
fn poll_truncates_an_overlong_line_at_the_bound() {
    let (mut s, tx) = wired();
    let mut giant = vec![b'a'; 5000];
    giant.push(b'\n');
    tx.send(Chunk::Stdout(giant)).unwrap();
    let StreamedPoll::Lines(lines) = s.poll() else {
        panic!("expected a truncated line");
    };
    // Capped at 4096 bytes, the rest dropped, with a visible marker appended.
    assert_eq!(
        lines,
        vec![out(&format!("{}…[truncated]", "a".repeat(4096)))]
    );
}

#[test]
fn poll_flushes_a_trailing_partial_line_at_eof_then_stays_pending() {
    let (mut s, tx) = wired();
    // A final device line with no trailing newline, then the child exits (both
    // pump senders drop → the channel disconnects → a synthesized Exited).
    tx.send(Chunk::Stdout(b"tail".to_vec())).unwrap();
    drop(tx);
    let done = s.poll();
    assert_eq!(
        done,
        StreamedPoll::Done(StreamedOutcome {
            lines: vec![out("tail")],
            exit: -1, // a child-less from_rx has no waitable status (Unknown → -1)
        })
    );
    // Idempotent after Done: no second outcome row, just Pending.
    assert_eq!(s.poll(), StreamedPoll::Pending);
}

#[test]
fn poll_streams_stderr_lines_live_tagged_as_such() {
    let (mut s, tx) = wired();
    // bz --login writes its whole flow here: this is the pane's only content
    // (bl-b4e5 defect 3 — a stdout-only class rendered nothing).
    tx.send(Chunk::Stderr(
        b"To authorize, open https://x/auth\n".to_vec(),
    ))
    .unwrap();
    assert_eq!(
        s.poll(),
        StreamedPoll::Lines(vec![err("To authorize, open https://x/auth")])
    );
}

#[test]
fn poll_flushes_both_partial_lines_and_the_exit_into_the_outcome_row() {
    let (mut s, tx) = wired();
    tx.send(Chunk::Stdout(b"payload".to_vec())).unwrap();
    tx.send(Chunk::Stderr(b"78: use `--browser`".to_vec()))
        .unwrap();
    // An explicit terminal chunk exercises the Done fold with a chosen exit code
    // (the real process delivers Exited via disconnect — see the S0-T5 story).
    tx.send(Chunk::Exited(ExitInfo::Code(2))).unwrap();
    assert_eq!(
        s.poll(),
        StreamedPoll::Done(StreamedOutcome {
            lines: vec![out("payload"), err("78: use `--browser`")],
            exit: 2,
        })
    );
}

#[test]
fn the_two_projections_split_the_tagged_lines() {
    let lines = [out("{\"models\":[]}"), err("warning"), err("78: nope")];
    assert_eq!(stdout_text(&lines), "{\"models\":[]}");
    assert_eq!(stderr_text(&lines), "warning\n78: nope");
    assert_eq!(stdout_text(&[]), "");
}

#[test]
fn try_next_emits_exit_once_then_stays_pending() {
    // Directly cover the non-blocking peer's post-exit guard: after the single
    // synthesized Exited, every further poll is Pending (no double outcome).
    let (tx, rx) = mpsc::channel::<Chunk>();
    drop(tx);
    let mut stream = Stream::from_rx(rx);
    assert!(matches!(
        stream.try_next(),
        StreamPoll::Ready(Chunk::Exited(_))
    ));
    assert_eq!(stream.try_next(), StreamPoll::Pending);
}
