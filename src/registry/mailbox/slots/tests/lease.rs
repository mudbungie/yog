//! **The lease** (REMOTE §5.3, §5.6): a hand-off is not a delivery, so an
//! unanswered invocation goes out again — three times, and then the engine
//! answers it in doubt rather than handing a box a tool that is killing it.

use super::super::super::Capture;
use super::super::read::HAND_OFFS;
use super::{call, quick, ran, took};

/// **The hand-off is not the delivery** (bl-e658): a slot handed to a read that
/// never answered it goes back on the queue at the client's next read, under
/// the id it was first handed.
///
/// This is the defect's own shape. `taken` was a latch, so an invocation
/// drained into a parked read whose peer had already died was consumed by a
/// thread that could not deliver it, and no later read ever offered it again.
#[test]
fn work_handed_to_a_read_that_never_answered_is_offered_again() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    assert_eq!(took(&mail, "laptop").len(), 1, "handed over once");
    assert_eq!(
        took(&mail, "laptop").first().map(|i| i.id.clone()),
        Some(id),
        "the next read is the acknowledgement the first one never gave"
    );
}

/// **A completed slot is never re-run.** The redelivery above offers only work
/// this engine has no answer for, so a capture waiting to be collected is not
/// handed out a second time.
#[test]
fn an_answered_invocation_is_not_offered_again() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    assert_eq!(took(&mail, "laptop").len(), 1);
    assert_eq!(mail.complete("laptop", &id, &ran(0)), Ok(ran(0)));
    assert!(
        took(&mail, "laptop").is_empty(),
        "answered work is finished work"
    );
}
/// The acknowledgement is **per client**: one machine asking again says nothing
/// about what another machine is holding.
#[test]
fn a_read_acknowledges_only_its_own_clients_work() {
    let mail = quick();
    let theirs = mail.post(10, "local", &call("phone", "Bash"));
    assert_eq!(took(&mail, "phone").len(), 1);
    assert!(took(&mail, "laptop").is_empty());
    assert_eq!(
        took(&mail, "phone").first().map(|i| i.id.clone()),
        Some(theirs)
    );
}

/// **The bound, and the answer past it** (REMOTE §5.6, ruling 2): three
/// hand-offs is the whole lease — delivery, the redelivery bl-e658 bought, and
/// one more that separates a blip from a poison invocation. At the read that
/// would hand it a fourth time the engine writes the slot's capture itself:
/// non-zero, nothing on stdout, and one sentence naming the client, the count
/// and the instruction the gesture lane already gives.
#[test]
fn a_third_unanswered_hand_off_is_answered_in_doubt_naming_the_count() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    for _ in 0..HAND_OFFS {
        assert_eq!(
            took(&mail, "laptop").first().map(|i| i.id.clone()),
            Some(id.clone()),
            "every read up to the bound is handed it again"
        );
    }
    assert!(
        took(&mail, "laptop").is_empty(),
        "the fourth read is answered by the engine instead"
    );
    let said = mail
        .collect("local", &id)
        .expect("the asker's own handle")
        .expect("the engine wrote the capture itself");
    assert_eq!(said.stdout, String::new(), "nothing ran here");
    assert_ne!(said.exit_code, 0, "a failed tool result, and read as one");
    assert!(said.stderr.contains("\"laptop\""), "{}", said.stderr);
    assert!(said.stderr.contains("3 times"), "{}", said.stderr);
    assert!(
        said.stderr.contains("each hand-off may have run it"),
        "{}",
        said.stderr
    );
    assert!(
        said.stderr.contains("read the world before acting again"),
        "{}",
        said.stderr
    );
}

/// Once the engine has answered in doubt the slot is a capture like any other,
/// so nothing offers it again — the loop the hour sweep used to be the only end
/// of is over at the count instead.
#[test]
fn an_in_doubt_slot_is_not_offered_again() {
    let mail = quick();
    mail.post(10, "local", &call("laptop", "Bash"));
    for _ in 0..=HAND_OFFS {
        took(&mail, "laptop");
    }
    assert!(
        took(&mail, "laptop").is_empty(),
        "an answered slot is finished work, whoever answered it"
    );
}

/// **A real capture beats a doubt** (REMOTE §5.6): a `complete` that lands on
/// an in-doubt slot the driver has not collected yet overwrites it, because the
/// box did run the tool after all. One that lands after the collect is refused
/// as any spent handle is, which is what a swept slot has always done.
///
/// It is also a slot handed out three times, so what it overwrites the doubt
/// with is a *marked* capture — the box's own output, with the count in front
/// of it ([`redelivered`](crate::registry::mailbox::doubt)).
#[test]
fn a_late_completion_overwrites_an_uncollected_in_doubt_answer() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    for _ in 0..=HAND_OFFS {
        took(&mail, "laptop");
    }
    let landed = mail.complete("laptop", &id, &ran(0)).expect("in flight");
    assert_eq!(landed.stdout, "out", "the box's own capture, kept whole");
    assert_eq!(landed.exit_code, 0);
    assert!(landed.stderr.contains("3 times"), "{}", landed.stderr);
    assert_eq!(
        mail.collect("local", &id),
        Ok(Some(landed)),
        "the driver collects what the engine stored, mark and all"
    );
    assert!(
        mail.complete("laptop", &id, &ran(0)).is_err(),
        "after the collect the handle is spent"
    );
}

/// **bl-0655**: a foot killed and restarted mid-flight ran one command twice on
/// the box, and the capture the model was handed read as one clean run. The
/// engine held the hand-off count the whole time and spent it only on the
/// give-up case; at hand-off two — one restart, one deploy, one blip — it said
/// nothing.
///
/// The mark is a sentence on stderr because litany renders exit code, stdout
/// and stderr into the `tool_result` envelope and nothing else, so those three
/// are the whole of what reaches the model.
#[test]
fn a_capture_for_a_redelivered_invocation_carries_the_count() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    assert_eq!(took(&mail, "laptop").len(), 1);
    // The foot died holding the capture; the next read hands the slot out
    // again and the box runs the command a second time.
    assert_eq!(took(&mail, "laptop").len(), 1);
    let said = mail
        .complete(
            "laptop",
            &id,
            &Capture {
                stdout: "2".to_owned(),
                stderr: "warning: already applied".to_owned(),
                exit_code: 0,
            },
        )
        .expect("in flight");
    assert_eq!(said.stdout, "2", "the box's own output is untouched");
    assert_eq!(said.exit_code, 0, "and so is its verdict");
    assert!(said.stderr.contains("\"laptop\""), "{}", said.stderr);
    assert!(said.stderr.contains("2 times"), "{}", said.stderr);
    assert!(
        said.stderr
            .contains("may be the effect of more than one run"),
        "{}",
        said.stderr
    );
    assert!(
        said.stderr.ends_with("warning: already applied"),
        "the mark goes first, and the tool's own stderr stays: {}",
        said.stderr
    );
    assert_eq!(mail.collect("local", &id), Ok(Some(said)));
}

/// **One hand-off is not a redelivery.** The ordinary run — and Ruling 1's
/// re-post, which lands before that channel's first follow-class read and so
/// meets a slot still at one — says nothing at all.
#[test]
fn a_capture_delivered_once_carries_no_mark() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    assert_eq!(took(&mail, "laptop").len(), 1);
    assert_eq!(
        mail.complete("laptop", &id, &ran(0)),
        Ok(ran(0)),
        "the capture the box computed, verbatim"
    );
}
