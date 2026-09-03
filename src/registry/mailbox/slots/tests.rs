//! The mailbox: a queue per client, a slot per invocation, and a hold that
//! ends rather than hangs.

use std::time::Instant;

use super::*;
use serde_json::json;

fn quick() -> Mailbox {
    Mailbox::holding(2, Duration::from_millis(1))
}

fn call(client: &str, tool: &str) -> Call {
    Call {
        client: client.to_owned(),
        tool: tool.to_owned(),
        input: json!({"command": "ls"}),
        cwd: None,
    }
}

/// One follow-class read that is expected to be granted its reader slot — the
/// ordinary case, so the refusal stays visible where a test means it.
fn took(mail: &Mailbox, client: &str) -> Vec<Invocation> {
    mail.take(client).expect("this client has no other reader")
}

fn ran(exit_code: i32) -> Capture {
    Capture {
        stdout: "out".to_owned(),
        stderr: String::new(),
        exit_code,
    }
}

/// The whole span, in one: post, take, complete, collect — and the slot is gone
/// after the one read that consumes it.
#[test]
fn an_invocation_crosses_and_the_capture_comes_back() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    assert_eq!(
        mail.collect("local", &id),
        Ok(None),
        "nothing has answered yet"
    );
    let taken = took(&mail, "laptop");
    assert_eq!(
        taken,
        vec![Invocation {
            id: id.clone(),
            tool: "Bash".to_owned(),
            input: json!({"command": "ls"}),
            cwd: None,
        }]
    );
    assert_eq!(mail.complete("laptop", &id, &ran(0)), Ok(ran(0)));
    assert_eq!(mail.collect("local", &id), Ok(Some(ran(0))));
    assert!(
        mail.collect("local", &id).is_err(),
        "the slot is released by the read that took it"
    );
}

/// A queue is per client: one host never drains another's work.
#[test]
fn a_take_drains_only_this_clients_work() {
    let mail = quick();
    let mine = mail.post(10, "local", &call("laptop", "Bash"));
    mail.post(10, "local", &call("phone", "Bash"));
    let taken = took(&mail, "laptop");
    assert_eq!(taken.len(), 1);
    assert_eq!(taken.first().map(|i| i.id.clone()), Some(mine));
}

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

/// **One reader per identity** (bl-1462): a second connection presenting the
/// same certificate while one is parked is refused in band, naming the client
/// and the two ways out — and the first reader keeps its work.
#[test]
fn a_second_reader_under_one_identity_is_refused() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    let parked = mail.reading("laptop").expect("the first reader");
    assert!(mail.serving("laptop"), "a machine is serving");

    let said = mail
        .take("laptop")
        .expect_err("one machine's queue has one reader");
    assert!(said.contains("\"laptop\""), "{said}");
    assert!(said.contains("already holding"), "{said}");
    assert!(said.contains("presenting this certificate"), "{said}");

    drop(parked);
    assert_eq!(
        took(&mail, "laptop").first().map(|i| i.id.clone()),
        Some(id),
        "the work was never taken by the refused reader"
    );
}

/// The claim is released **however the read leaves**, so the slot is a claim on
/// a live reader and never a latch of its own — and `serving` answers the same
/// fact the advertisement's gate reads.
#[test]
fn the_reader_slot_is_released_when_the_read_ends() {
    let mail = quick();
    assert!(!mail.serving("laptop"), "nobody is reading");
    assert!(took(&mail, "laptop").is_empty());
    assert!(!mail.serving("laptop"), "the hold ended, so did the claim");
    assert!(
        mail.take("laptop").is_ok(),
        "a client may read again once its own read is over"
    );
}

/// The claim is **per identity**: one machine reading never shuts another out.
#[test]
fn two_machines_read_at_once() {
    let mail = quick();
    let _phone = mail.reading("phone").expect("the phone's own slot");
    let mine = mail.post(10, "local", &call("laptop", "Bash"));
    assert_eq!(
        took(&mail, "laptop").first().map(|i| i.id.clone()),
        Some(mine)
    );
    assert!(!mail.serving("laptop"), "and it let its own slot go");
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

/// **The hold ends** (REMOTE §3): a host with nothing waiting is answered with
/// nothing, and asks again. An answer that never came would be the hang.
#[test]
fn an_empty_hold_expires_and_answers_nothing() {
    assert!(took(&quick(), "laptop").is_empty());
}

/// The hold ends **early** when work lands — the whole point of a follow-class
/// read over a poll at human cadence.
///
/// **The hand-off is `serving`, never a sleep** (bl-b8c8). The read publishes
/// its reader claim on the way in, so that flag is a fact both threads observe
/// and the post is ordered after the park by the mailbox itself. A sleep is
/// not a rendezvous: on a loaded box the writer's wake lands after the hold
/// has expired, and the beat then fails on the machine rather than on the
/// fold. The deadline is the writer's own escape — it posts anyway, so a read
/// that never parked fails the assertion below instead of hanging the suite.
#[test]
fn work_landing_mid_hold_ends_it() {
    let mail = Mailbox::holding(400, Duration::from_millis(5));
    let writer = mail.clone();
    let hand = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !writer.serving("laptop") && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        writer.post(10, "local", &call("laptop", "Bash"))
    });
    let taken = took(&mail, "laptop");
    let id = hand.join().expect("the writer");
    assert_eq!(
        taken.first().map(|i| i.id.clone()),
        Some(id),
        "the parked read answered with the work rather than expiring empty"
    );
}

/// A handle this engine does not hold refuses **naming it**, at both readers:
/// the completion that quotes it and the poll that waits on it.
#[test]
fn an_unheld_handle_refuses_at_both_readers() {
    let mail = quick();
    let refusal = mail.complete("laptop", "inv-9", &ran(0));
    assert_eq!(refusal, Err(unknown("inv-9")));
    assert!(refusal.unwrap_err().contains("inv-9"));
    assert_eq!(mail.collect("local", "inv-9"), Err(unknown("inv-9")));
}

/// **Absence, not forbidden** (REMOTE §4): a handle addressed to another
/// machine, or posted by another caller, earns the very sentence an id nobody
/// minted earns — a refusal that confirmed existence would be the disclosure.
#[test]
fn another_partys_handle_is_absent_rather_than_forbidden() {
    let mail = quick();
    let id = mail.post(10, "local", &call("laptop", "Bash"));
    assert_eq!(mail.complete("phone", &id, &ran(0)), Err(unknown(&id)));
    assert_eq!(mail.collect("phone", &id), Err(unknown(&id)));
    assert_eq!(mail.complete("laptop", &id, &ran(0)), Ok(ran(0)));
    assert_eq!(mail.collect("local", &id), Ok(Some(ran(0))));
}

/// The sweep: an abandoned slot does not outlive the hour, and the pass runs at
/// the one moment the map can grow.
#[test]
fn a_post_sweeps_what_no_driver_ever_collected() {
    let mail = quick();
    let stale = mail.post(0, "local", &call("laptop", "Bash"));
    assert!(
        mail.collect("local", &stale).is_ok(),
        "live within the hour"
    );
    mail.post(TTL_SECONDS, "local", &call("laptop", "Bash"));
    assert!(
        mail.collect("local", &stale).is_ok(),
        "the boundary is inclusive"
    );
    mail.post(TTL_SECONDS + 1, "local", &call("laptop", "Bash"));
    assert!(mail.collect("local", &stale).is_err(), "swept");
}

/// The production mailbox is the same map with a thirty-second hold — asserted
/// so the default is a fact rather than a hope.
#[test]
fn the_default_hold_is_the_production_bound() {
    let mail = Mailbox::default();
    assert_eq!(mail.waits, HOLD_WAITS);
    assert_eq!(mail.tick, HOLD_TICK);
    let id = mail.post(0, "local", &call("laptop", "Bash"));
    assert_eq!(took(&mail, "laptop").len(), 1, "work waiting ends the hold");
    assert_eq!(mail.collect("local", &id), Ok(None));
}
