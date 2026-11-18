//! **A capture is N ≥ 1 posts** (REMOTE §5.7, bl-d414): posts absorb in
//! order, an offset is an idempotent ask, a run lives while its asker listens,
//! and an acknowledged slot is answered in doubt rather than handed out again.

use super::super::super::doubt::{outside, unknown};
use super::super::super::{Capture, Offsets, Piece, Post, Standing};
use super::super::run::UNREAD_LIMIT;
use super::{Mailbox, NOW, call, poll, quick, took};

pub(super) fn piece(stdout: &str, stderr: &str) -> Post {
    Post::Partial(Piece {
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
    })
}

pub(super) fn ending(stdout: &str, exit_code: i32) -> Post {
    Post::Capture(Capture {
        stdout: stdout.to_owned(),
        stderr: String::new(),
        exit_code,
    })
}

pub(super) fn at(stdout: u64, stderr: u64) -> Offsets {
    Offsets { stdout, stderr }
}

/// What a live slot answers an asker holding offsets: the text from there on.
pub(super) fn live(stdout: &str, stderr: &str) -> Standing {
    Standing {
        partial: Some(Piece {
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        }),
        capture: None,
    }
}

/// A slot queued at [`NOW`] and handed to its machine once.
pub(super) fn running(mail: &Mailbox) -> String {
    let id = mail.post(NOW, "local", &call("laptop", "bz"));
    assert_eq!(took(mail, "laptop").len(), 1);
    id
}

/// **Posts absorb in order, and the one carrying a capture ends the run.** An
/// asker that states no offsets reads exactly what it always read: nothing
/// until the end, then the whole — so N posts and one post are one form.
#[test]
fn posts_absorb_in_order_and_the_capture_ends_the_run() {
    let mail = quick();
    let id = running(&mail);
    assert_eq!(
        mail.complete(NOW, "laptop", &id, &piece("a", "x")),
        Ok(live("", "")),
        "the receipt is the lease fact, not an echo"
    );
    assert_eq!(poll(&mail, "local", &id), Ok(None), "the run is not over");
    assert_eq!(
        mail.complete(NOW, "laptop", &id, &piece("b", "")),
        Ok(live("", ""))
    );
    let whole = Capture {
        stdout: "abc".to_owned(),
        stderr: "x".to_owned(),
        exit_code: 0,
    };
    assert_eq!(
        mail.complete(NOW, "laptop", &id, &ending("c", 0)),
        Ok(Standing {
            partial: None,
            capture: Some(whole.clone()),
        }),
        "the last post's text is what was not yet posted"
    );
    assert_eq!(poll(&mail, "local", &id), Ok(Some(whole)));
}

/// **An offset is an idempotent ask**: asked twice at one count the answer is
/// the same text, and `partial` is absent until a machine has posted.
#[test]
fn since_is_idempotent_and_partial_is_the_lease_fact() {
    let mail = quick();
    let id = running(&mail);
    assert_eq!(
        mail.collect(NOW, "local", &id, Some(at(0, 0))),
        Ok(Standing::default()),
        "nobody has posted for this slot"
    );
    mail.complete(NOW, "laptop", &id, &piece("", ""))
        .expect("still running");
    assert_eq!(
        mail.collect(NOW, "local", &id, Some(at(0, 0))),
        Ok(live("", ""))
    );
    mail.complete(NOW, "laptop", &id, &piece("ab", "e"))
        .expect("in flight");
    for _ in 0..2 {
        assert_eq!(
            mail.collect(NOW, "local", &id, Some(at(0, 0))),
            Ok(live("ab", "e"))
        );
    }
    mail.complete(NOW, "laptop", &id, &piece("cd", ""))
        .expect("in flight");
    for _ in 0..2 {
        assert_eq!(
            mail.collect(NOW, "local", &id, Some(at(2, 1))),
            Ok(live("cd", ""))
        );
    }
}

/// **The offset is the acknowledgement**, so the server has dropped what an
/// asker passed: a count before it, past what arrived, or inside a character
/// has no text to answer from and refuses in band.
#[test]
fn an_offset_no_stream_can_answer_from_is_refused() {
    let mail = quick();
    let id = running(&mail);
    mail.complete(NOW, "laptop", &id, &piece("aé", ""))
        .expect("in flight");
    assert_eq!(
        mail.collect(NOW, "local", &id, Some(at(1, 0))),
        Ok(live("é", ""))
    );
    for bad in [
        Some(at(0, 0)),
        Some(at(2, 0)),
        Some(at(9, 0)),
        Some(at(1, 1)),
    ] {
        assert_eq!(
            mail.collect(NOW, "local", &id, bad),
            Err(outside(&id)),
            "{bad:?}"
        );
    }
    assert_eq!(
        mail.collect(NOW, "local", &id, Some(at(3, 0))),
        Ok(live("", ""))
    );
}

/// **The final capture, asked with offsets, carries only the remainder** —
/// and the slot is released by that read, as a capture always was.
#[test]
fn a_final_capture_with_since_is_the_remainder() {
    let mail = quick();
    let id = running(&mail);
    mail.complete(NOW, "laptop", &id, &piece("ab", ""))
        .expect("in flight");
    assert_eq!(
        mail.collect(NOW, "local", &id, Some(at(0, 0))),
        Ok(live("ab", ""))
    );
    mail.complete(NOW, "laptop", &id, &ending("cd", 3))
        .expect("in flight");
    assert_eq!(
        mail.collect(NOW, "local", &id, Some(at(2, 0))),
        Ok(Standing {
            partial: None,
            capture: Some(Capture {
                stdout: "cd".to_owned(),
                stderr: String::new(),
                exit_code: 3,
            }),
        })
    );
    assert_eq!(poll(&mail, "local", &id), Err(unknown(&id)), "read once");
}

/// **A run that is over takes no more posts.** A capture re-posted after the
/// first one landed would be absorbed twice, so it is a spent handle.
#[test]
fn a_post_for_a_finished_run_is_a_spent_handle() {
    let mail = quick();
    let id = running(&mail);
    mail.complete(NOW, "laptop", &id, &ending("out", 0))
        .expect("in flight");
    for late in [ending("out", 0), piece("more", "")] {
        assert_eq!(mail.complete(NOW, "laptop", &id, &late), Err(unknown(&id)));
    }
    assert_eq!(
        poll(&mail, "local", &id).map(|c| c.map(|c| c.stdout)),
        Ok(Some("out".to_owned())),
        "and what landed first is what is collected"
    );
}

/// **The redelivery mark goes in front of the first text a run ever holds**
/// (REMOTE §5.6), on its own line when the run streams.
#[test]
fn a_redelivered_run_that_streams_is_marked_at_its_first_post() {
    let mail = quick();
    let id = running(&mail);
    assert_eq!(took(&mail, "laptop").len(), 1, "handed out a second time");
    mail.complete(NOW, "laptop", &id, &piece("a", ""))
        .expect("in flight");
    mail.complete(NOW, "laptop", &id, &piece("", "warn"))
        .expect("in flight");
    let read = mail
        .collect(NOW, "local", &id, Some(at(0, 0)))
        .expect("the asker's own handle")
        .partial
        .expect("posted");
    assert!(
        read.stderr.contains("\"laptop\" 2 times"),
        "{}",
        read.stderr
    );
    assert!(
        read.stderr.ends_with("\nwarn"),
        "said once: {}",
        read.stderr
    );
}

/// **The slot holds what is unread, bounded** at the foot's old per-stream
/// capture limit: past it the server keeps nothing more, cuts on a character,
/// and says what it dropped on stderr when the run ends. What an asker's
/// offset has passed is dropped, which is what makes room.
#[test]
fn unread_text_is_bounded_and_the_drop_is_said_on_stderr() {
    let mail = quick();
    let id = running(&mail);
    let flood = format!("{}éyyy", "x".repeat(UNREAD_LIMIT - 1));
    mail.complete(NOW, "laptop", &id, &piece(&flood, ""))
        .expect("in flight");
    mail.complete(NOW, "laptop", &id, &piece("more", ""))
        .expect("in flight");
    let held = (UNREAD_LIMIT - 1) as u64;
    assert_eq!(
        mail.collect(NOW, "local", &id, Some(at(held, 0))),
        Ok(live("m", "")),
        "the cut fell before the character that did not fit, leaving one byte of room"
    );
    mail.complete(NOW, "laptop", &id, &ending("fits", 0))
        .expect("in flight");
    let ended = mail
        .collect(NOW, "local", &id, Some(at(held + 1, 0)))
        .expect("the asker's own handle")
        .capture
        .expect("over");
    assert_eq!(ended.stdout, "fits", "an acknowledged read made room");
    assert_eq!(
        ended.stderr,
        format!(
            "\nyog: stdout exceeded this server's {UNREAD_LIMIT}-byte capture limit; \
             8 further bytes were dropped\n"
        )
    );
}
