//! **A machine that is working, and what it last answered** (REMOTE §5.7,
//! bl-3b4b): `busy` is the job the mailbox handed out and still believes in,
//! and `known` is the one replica of a machine's answer, served only then.

use super::super::TTL_SECONDS;
use super::super::run::LISTEN_SECONDS;
use super::stream::{ending, piece, running};
use super::{NOW, call, quick, ran, took};

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

/// **The replica is served only while its machine is busy**, is replaced by
/// each fresh answer, is one read's and not another's, and is swept at the
/// hour with everything else this map holds.
#[test]
fn what_a_machine_last_said_is_known_while_it_works() {
    let mail = quick();
    let (read, other) = (call("laptop", "bz"), call("laptop", "shell"));
    mail.remember(NOW, &read, &ran(0));
    assert_eq!(mail.known(NOW, &read), None, "an idle machine is asked");

    running(&mail);
    assert_eq!(mail.known(NOW, &read), Some(ran(0)));
    assert_eq!(mail.known(NOW, &other), None, "another read's answer");
    mail.remember(NOW, &read, &ran(7));
    assert_eq!(
        mail.known(NOW, &read),
        Some(ran(7)),
        "replaced, not kept beside"
    );

    let late = NOW + TTL_SECONDS + 1;
    mail.post(late, "local", &call("laptop", "bz"));
    assert_eq!(took(&mail, "laptop").len(), 1);
    assert!(mail.busy(late, "laptop"));
    assert_eq!(mail.known(late, &read), None, "an hour old is not known");
}
