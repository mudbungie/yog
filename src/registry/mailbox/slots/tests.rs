//! The mailbox: a queue per client, a slot per invocation, and a hold that
//! ends rather than hangs.

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
    let taken = mail.take("laptop");
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
    let taken = mail.take("laptop");
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
    assert_eq!(mail.take("laptop").len(), 1, "handed over once");
    assert_eq!(
        mail.take("laptop").first().map(|i| i.id.clone()),
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
    assert_eq!(mail.take("laptop").len(), 1);
    assert_eq!(mail.complete("laptop", &id, &ran(0)), Ok(ran(0)));
    assert!(
        mail.take("laptop").is_empty(),
        "answered work is finished work"
    );
}

/// The acknowledgement is **per client**: one machine asking again says nothing
/// about what another machine is holding.
#[test]
fn a_read_acknowledges_only_its_own_clients_work() {
    let mail = quick();
    let theirs = mail.post(10, "local", &call("phone", "Bash"));
    assert_eq!(mail.take("phone").len(), 1);
    assert!(mail.take("laptop").is_empty());
    assert_eq!(
        mail.take("phone").first().map(|i| i.id.clone()),
        Some(theirs)
    );
}

/// **The hold ends** (REMOTE §3): a host with nothing waiting is answered with
/// nothing, and asks again. An answer that never came would be the hang.
#[test]
fn an_empty_hold_expires_and_answers_nothing() {
    assert!(quick().take("laptop").is_empty());
}

/// The hold ends **early** when work lands — the whole point of a follow-class
/// read over a poll at human cadence.
#[test]
fn work_landing_mid_hold_ends_it() {
    let mail = Mailbox::holding(400, Duration::from_millis(5));
    let writer = mail.clone();
    let hand = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        writer.post(10, "local", &call("laptop", "Bash"))
    });
    let taken = mail.take("laptop");
    let id = hand.join().expect("the writer");
    assert_eq!(taken.first().map(|i| i.id.clone()), Some(id));
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
    assert_eq!(mail.take("laptop").len(), 1, "work waiting ends the hold");
    assert_eq!(mail.collect("local", &id), Ok(None));
}
