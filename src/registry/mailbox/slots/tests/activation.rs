//! **A newer activation takes the queue** (REMOTE §5.1, bl-96e0): an
//! advertisement ends the read its certificate holds, that read is told, and
//! the one that follows is granted at once.

use std::time::{Duration, Instant};

use super::super::read::Taken;
use super::stream::piece;
use super::{Mailbox, NOW, call, poll, quick, took};

/// A hold long enough that a read which ended inside it was ended, not expired:
/// twenty seconds, looked at every five milliseconds.
fn patient() -> Mailbox {
    Mailbox::holding(4000, Duration::from_millis(5))
}

/// Wait until `client` is parked — a fact both threads observe (bl-b8c8), never
/// a sleep that stands in for one.
fn parked(mail: &Mailbox, client: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !mail.serving(client) {
        assert!(Instant::now() < deadline, "{client} never parked");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// **The restart, at the mailbox.** A read is parked — the process behind it
/// may be dead or alive, and the server cannot tell. Its certificate's next
/// activation ends it: the slot is free before `supersede` returns, and the
/// parked read answers that it was superseded rather than sleeping out its
/// hold.
#[test]
fn an_activation_ends_the_parked_read_and_tells_it() {
    let mail = patient();
    let old = mail.clone();
    let began = Instant::now();
    let reader = std::thread::spawn(move || old.take("laptop"));
    parked(&mail, "laptop");

    mail.supersede("laptop");
    assert!(!mail.serving("laptop"), "the slot is free at once");
    let told = reader.join().expect("the old read");
    assert_eq!(
        told,
        Ok(Taken::Superseded),
        "the read that lost the queue is told so"
    );
    assert!(
        told.is_ok_and(|taken| taken.work().is_empty()),
        "and was handed nothing"
    );
    assert!(
        began.elapsed() < Duration::from_secs(10),
        "and it was ended, not left to expire"
    );
}

/// **A superseded read is handed nothing more**, even with work waiting: what
/// is queued goes to the activation that took the queue.
#[test]
fn work_posted_after_the_activation_goes_to_the_successor() {
    let mail = patient();
    let old = mail.clone();
    let reader = std::thread::spawn(move || old.take("laptop"));
    parked(&mail, "laptop");

    mail.supersede("laptop");
    let id = mail.post(NOW, "local", &call("laptop", "Bash"));
    assert_eq!(reader.join().expect("the old read"), Ok(Taken::Superseded));
    assert_eq!(
        took(&quick_on(&mail), "laptop")
            .first()
            .map(|i| i.id.clone()),
        Some(id),
        "handed once, to the read that holds the queue"
    );
}

/// The same slots under a hold that ends — a second handle on one mailbox, so
/// a beat that reads after the fact does not wait twenty seconds for nothing.
fn quick_on(mail: &Mailbox) -> Mailbox {
    Mailbox {
        cell: mail.cell.clone(),
        ..quick()
    }
}

/// **The superseded read's exit does not release its successor's claim.** The
/// claim is one read's, told apart by its token: a predecessor that wakes after
/// the new reader has parked must leave that reader holding the queue.
#[test]
fn a_superseded_read_leaving_does_not_free_its_successors_slot() {
    let mail = quick();
    let old = mail.reading("laptop").expect("the first activation's read");
    mail.supersede("laptop");
    let new = mail.reading("laptop").expect("granted at once");
    drop(old);
    assert!(
        mail.serving("laptop"),
        "the successor still holds the queue"
    );
    drop(new);
    assert!(!mail.serving("laptop"));
}

/// **With nothing parked it ends nothing** — the general path with no read to
/// end, which is every advertisement a foot makes after a hand-off.
#[test]
fn an_activation_with_nothing_parked_changes_nothing() {
    let mail = quick();
    mail.supersede("laptop");
    assert!(took(&mail, "laptop").is_empty());
}

/// **An activation is per certificate**: one machine restarting ends no other
/// machine's read.
#[test]
fn an_activation_ends_only_its_own_certificates_read() {
    let mail = quick();
    let _phone = mail.reading("phone").expect("the phone's own slot");
    mail.supersede("laptop");
    assert!(mail.serving("phone"));
}

/// **Work the superseded activation held ends as any dead foot's does**
/// (REMOTE §5.6, §5.7 — unchanged): a run it had begun to post is answered in
/// doubt and never handed out again; a hand-off it never acknowledged goes to
/// its successor under the id it was first handed.
#[test]
fn work_the_superseded_activation_held_follows_the_lease() {
    let mail = quick();
    let begun = mail.post(NOW, "local", &call("laptop", "Bash"));
    let unheard = mail.post(NOW, "local", &call("laptop", "Bash"));
    assert_eq!(took(&mail, "laptop").len(), 2, "both handed to the old one");
    mail.complete(NOW, "laptop", &begun, &piece("half", ""))
        .expect("in flight");

    mail.supersede("laptop");
    let again: Vec<String> = took(&mail, "laptop").into_iter().map(|i| i.id).collect();
    assert_eq!(again, vec![unheard], "only the unacknowledged one");
    let said = poll(&mail, "local", &begun)
        .expect("the asker's own handle")
        .expect("the server ended the run itself");
    assert_eq!(said.stdout, "half");
    assert_ne!(said.exit_code, 0);
    assert!(
        said.stderr
            .contains("asked for work again without finishing it"),
        "{}",
        said.stderr
    );
}
