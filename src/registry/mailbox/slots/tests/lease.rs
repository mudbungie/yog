//! **The lease** (REMOTE §5.3, §5.6): a hand-off is not a delivery, so an
//! unanswered invocation goes out again — three times, and then the engine
//! answers it in doubt rather than handing a box a tool that is killing it.

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
#[test]
fn a_late_completion_overwrites_an_uncollected_in_doubt_answer() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    for _ in 0..=HAND_OFFS {
        took(&mail, "laptop");
    }
    assert_eq!(mail.complete("laptop", &id, &ran(0)), Ok(ran(0)));
    assert_eq!(
        mail.collect("local", &id),
        Ok(Some(ran(0))),
        "the driver collects what the box actually captured"
    );
    assert!(
        mail.complete("laptop", &id, &ran(0)).is_err(),
        "after the collect the handle is spent"
    );
}
