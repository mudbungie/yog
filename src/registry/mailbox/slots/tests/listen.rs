//! **A run lives while its asker listens, and an acknowledged run is never
//! handed out again** (REMOTE §5.7, bl-d414): the two silences that end a
//! streamed run, and the post that is refused because nobody is waiting.

use super::super::super::doubt::{unheard, unknown};
use super::super::super::{Capture, Standing};
use super::super::run::LISTEN_SECONDS;
use super::stream::{at, ending, live, piece, running};
use super::{NOW, finish, poll, quick, took};

/// **An acknowledged slot is never handed out again.** Its machine asking for
/// work again means the run will not finish, so the server answers it in doubt
/// at once: what arrived is kept, the verdict is a failure, and the sentence
/// names how much had arrived.
#[test]
fn an_acknowledged_slot_is_answered_in_doubt_when_its_machine_asks_again() {
    let mail = quick();
    let id = running(&mail);
    mail.complete(NOW, "laptop", &id, &piece("half", "warn"))
        .expect("in flight");
    assert!(took(&mail, "laptop").is_empty(), "not handed out again");
    assert!(took(&mail, "laptop").is_empty(), "nor on any later read");
    let said = poll(&mail, "local", &id)
        .expect("the asker's own handle")
        .expect("the server ended the run itself");
    assert_eq!(said.stdout, "half", "what arrived is real output");
    assert_ne!(said.exit_code, 0);
    assert!(said.stderr.starts_with("warn\n"), "{}", said.stderr);
    for clause in [
        "\"laptop\" 1 times",
        "asked for work again without finishing it",
        "4 bytes of stdout and 4 of stderr had arrived",
        "read the world before acting again",
    ] {
        assert!(said.stderr.contains(clause), "{}", said.stderr);
    }
}

/// **…and when it goes thirty seconds without a post** — judged lazily, at the
/// asker's own poll, and answered as the remainder to an asker holding offsets.
#[test]
fn an_acknowledged_slot_is_answered_in_doubt_on_silence() {
    let mail = quick();
    let id = running(&mail);
    mail.complete(NOW, "laptop", &id, &piece("half", ""))
        .expect("in flight");
    let edge = NOW + LISTEN_SECONDS;
    assert_eq!(
        mail.collect(edge, "local", &id, Some(at(4, 0))),
        Ok(live("", "")),
        "thirty seconds is still a run"
    );
    let ended = mail
        .collect(edge + 1, "local", &id, Some(at(4, 0)))
        .expect("the asker's own handle")
        .capture
        .expect("in doubt");
    assert_eq!(ended.stdout, "", "the remainder, and there was none");
    assert_eq!(ended.exit_code, 1);
    assert!(
        ended.stderr.contains("posted nothing for thirty seconds"),
        "{}",
        ended.stderr
    );
    // An unacknowledged slot is not judged by this clock at all: it keeps the
    // three hand-offs (REMOTE §5.6).
    let quiet = running(&mail);
    assert_eq!(
        mail.collect(edge + 99, "local", &quiet, None),
        Ok(Standing::default())
    );
    assert_eq!(took(&mail, "laptop").len(), 1, "handed out again");
}

/// **A run lives while its asker listens.** A piece for a slot nobody polled
/// in thirty seconds is refused, the refusal is the stop, and the slot is
/// ended in doubt so no later read hands it out. Every poll stamps the slot.
#[test]
fn a_piece_for_an_unlistened_slot_is_refused_and_the_slot_is_ended() {
    let mail = quick();
    let id = running(&mail);
    let edge = NOW + LISTEN_SECONDS;
    assert_eq!(
        mail.complete(edge, "laptop", &id, &piece("a", "")),
        Ok(live("", "")),
        "thirty seconds is still listening"
    );
    assert_eq!(
        mail.collect(edge + 5, "local", &id, None),
        Ok(Standing::default())
    );
    assert_eq!(
        mail.complete(edge + 35, "laptop", &id, &piece("b", "")),
        Ok(live("", "")),
        "the poll stamped it"
    );
    assert_eq!(
        mail.complete(edge + 36, "laptop", &id, &piece("c", "")),
        Err(unheard(&id))
    );
    assert!(took(&mail, "laptop").is_empty(), "never handed out again");
    let said = poll(&mail, "local", &id)
        .expect("the asker's own handle")
        .expect("ended in doubt");
    assert_eq!(said.stdout, "ab", "the refused piece was not absorbed");
    assert!(
        said.stderr.contains("nobody had asked for its capture"),
        "{}",
        said.stderr
    );
}

/// **The post carrying the capture is exempt**: there is no child left to
/// stop, and a finished answer is worth more to a late asker than a doubt —
/// which is also what keeps a foot that posts once exactly as it was.
#[test]
fn a_capture_for_an_unlistened_slot_still_lands() {
    let mail = quick();
    let id = running(&mail);
    let late = NOW + LISTEN_SECONDS + 600;
    assert!(
        mail.complete(late, "laptop", &id, &ending("out", 0))
            .is_ok()
    );
    assert_eq!(
        finish(&mail, "laptop", &id, &Capture::default()),
        Err(unknown(&id))
    );
    assert_eq!(
        poll(&mail, "local", &id).map(|c| c.map(|c| c.stdout)),
        Ok(Some("out".to_owned()))
    );
    // And it reopens a slot the server itself ended with nothing posted: a
    // refused first piece leaves a doubt, and the real capture replaces it.
    let id = running(&mail);
    assert_eq!(
        mail.complete(late, "laptop", &id, &piece("a", "")),
        Err(unheard(&id))
    );
    assert_eq!(
        mail.complete(late, "laptop", &id, &piece("a", "")),
        Err(unheard(&id)),
        "still nobody listening"
    );
    assert!(
        mail.complete(late, "laptop", &id, &ending("out", 0))
            .is_ok()
    );
    assert_eq!(
        poll(&mail, "local", &id),
        Ok(Some(Capture {
            stdout: "out".to_owned(),
            ..Capture::default()
        }))
    );
}
