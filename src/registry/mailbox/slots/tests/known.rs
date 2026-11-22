//! **A machine that is working, and what it last answered** (REMOTE §5.7,
//! bl-3b4b, bl-d9ad): `busy` is the job the mailbox handed out and still
//! believes in; a read is asked once and its answer kept.

use super::super::TTL_SECONDS;
use super::super::run::LISTEN_SECONDS;
use super::stream::{ending, piece, running};
use super::{NOW, call, finish, quick, ran, took};

/// **Queued is not working, and handed is.** A job nobody took says nothing
/// about its machine; one a read was handed makes that machine busy — and no
/// other machine.
#[test]
fn a_machine_is_busy_from_the_hand_off() {
    let mail = quick();
    mail.post(NOW, "local", &call("laptop", "bz"));
    assert!(!mail.busy(NOW, "laptop"), "nobody has taken it");
    assert_eq!(took(&mail, "laptop").len(), 1);
    assert!(mail.busy(NOW, "laptop"));
    assert!(!mail.busy(NOW, "desktop"), "one machine's job is its own");
}

/// **Busy is believed as long as the job is, and no longer.** The capture ends
/// it; so does an asker that stopped listening; so does a run that posted and
/// then said nothing for thirty seconds — the death of a machine mid-job, as
/// far as this server can see one.
#[test]
fn busy_ends_by_the_rules_that_end_a_run() {
    let mail = quick();
    let id = running(&mail);
    assert!(mail.busy(NOW + LISTEN_SECONDS, "laptop"));
    assert!(
        !mail.busy(NOW + LISTEN_SECONDS + 1, "laptop"),
        "its asker has not asked in thirty seconds"
    );

    mail.complete(NOW, "laptop", &id, &piece("half", ""))
        .expect("in flight");
    let heard = NOW + LISTEN_SECONDS;
    mail.collect(heard, "local", &id, None).expect("listening");
    assert!(mail.busy(heard, "laptop"));
    assert!(
        !mail.busy(heard + 1, "laptop"),
        "its machine has not posted in thirty seconds"
    );

    let id = running(&mail);
    assert!(mail.busy(NOW, "laptop"));
    mail.complete(NOW, "laptop", &id, &ending("done", 0))
        .expect("in flight");
    assert!(
        !mail.busy(NOW, "laptop"),
        "a finished job is not a running one"
    );
}

/// **A read is in flight at most once, and its answer is kept.** Asked twice
/// it is queued once; the capture lands in what is known whoever was waiting;
/// asked again it goes out again, and the earlier answer stands until the
/// fresh one replaces it. Another read's answer is its own.
#[test]
fn a_read_is_queued_once_and_its_answer_is_kept() {
    let mail = quick();
    let read = call("laptop", "bz");
    assert_eq!(mail.known(&read), None, "never asked");
    mail.ask(NOW, &read);
    mail.ask(NOW, &read);
    assert!(mail.queued(NOW, &read));
    let work = took(&mail, "laptop");
    assert_eq!(work.len(), 1, "asked twice, queued once");
    for handed in &work {
        finish(&mail, "laptop", &handed.id, &ran(0)).expect("in flight");
    }
    assert!(!mail.queued(NOW, &read), "answered with nobody waiting");
    assert_eq!(mail.known(&read), Some(ran(0)));

    mail.ask(NOW, &read);
    assert!(mail.queued(NOW, &read), "asked again, out again");
    assert_eq!(mail.known(&read), Some(ran(0)), "the last answer stands");
    for handed in took(&mail, "laptop") {
        finish(&mail, "laptop", &handed.id, &ran(7)).expect("in flight");
    }
    assert!(!mail.queued(NOW, &read));
    assert_eq!(mail.known(&read), Some(ran(7)), "replaced, not kept beside");
    assert_eq!(mail.known(&call("laptop", "shell")), None);
}

/// **An ask ends as any run does**: its machine going quiet answers it in
/// doubt, and that is what is known. And a read nobody has asked in an hour is
/// swept with its flight.
#[test]
fn an_ask_ends_in_doubt_or_is_swept() {
    let mail = quick();
    let read = call("laptop", "bz");
    mail.ask(NOW, &read);
    for handed in took(&mail, "laptop") {
        mail.complete(NOW, "laptop", &handed.id, &piece("half", ""))
            .expect("in flight");
    }
    assert!(mail.queued(NOW + LISTEN_SECONDS, &read));
    assert!(
        !mail.queued(NOW + LISTEN_SECONDS + 1, &read),
        "it went quiet"
    );
    let said = mail.known(&read).expect("the doubt is the answer");
    assert_eq!((said.stdout.as_str(), said.exit_code), ("half", 1));

    mail.ask(NOW, &read);
    mail.post(NOW + TTL_SECONDS + 1, "local", &call("desktop", "bz"));
    assert_eq!(mail.known(&read), None, "an hour unasked is not known");
    assert!(!mail.queued(NOW + TTL_SECONDS + 1, &read), "nor in flight");
}
