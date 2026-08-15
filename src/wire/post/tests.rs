//! The act path's frame half: a ticket per act, a receipt per ticket, and a
//! bound on the receipts nobody reads.

use super::*;
use crate::boundary::reply::Reply;
use serde_json::json;

/// An act envelope with `n` in it — the two tests below only care that two acts
/// are two values, never that they are gestures the codec would accept.
fn act(n: u64) -> Value {
    json!({"op": "nudge", "n": n})
}

/// **A ticket is minted, never derived.** Two identical acts are two acts —
/// which is the whole reason the read path's rule (the envelope is the key)
/// cannot be borrowed here — so they earn different tickets and their receipts
/// do not collide.
#[test]
fn two_identical_acts_are_two_tickets_with_two_receipts() {
    let (mut post, outbox) = pair();
    let (first, second) = (post.send(&act(1)), post.send(&act(1)));
    assert_ne!(
        first, second,
        "an act is not idempotent, so nor is its handle"
    );
    let sent: Vec<(Ticket, Value)> = std::iter::from_fn(|| outbox.next()).take(2).collect();
    assert_eq!(sent[0].0, first, "and they are sent in the order clicked");
    assert_eq!(sent[1].0, second);

    outbox.publish(second, Err("second".to_owned()));
    outbox.publish(first, Err("first".to_owned()));
    assert_eq!(post.settle().len(), 2);
    assert_eq!(post.receipt(first), Some(Err("first".to_owned())));
    assert_eq!(post.receipt(second), Some(Err("second".to_owned())));
    assert_eq!(post.receipt(first), None, "a receipt is spent by its read");
}

/// **Nothing behind the window is answered in the send**, with the same one
/// `Err` a refusal is — so there is no "never came" state for a surface to
/// paint and no timeout to arrange.
#[test]
fn an_act_with_no_wire_behind_it_earns_its_receipt_at_once() {
    let mut post = Post::default();
    let ticket = post.send(&act(1));
    let Some(Err(said)) = post.receipt(ticket) else {
        panic!("the send is its own receipt when there is nobody to send to");
    };
    assert_eq!(said, NO_WIRE);
}

/// A receipt waits for its reader, and the map is bounded: the oldest goes
/// first once [`RECEIPTS_KEPT`] later acts have landed, because a receipt still
/// unread by then has no holder.
#[test]
fn unread_receipts_are_bounded_oldest_first() {
    let (mut post, outbox) = pair();
    let tickets: Vec<Ticket> = (0..=RECEIPTS_KEPT as u64)
        .map(|n| post.send(&act(n)))
        .collect();
    while let Some((ticket, _)) = outbox.next() {
        outbox.publish(ticket, Ok(Reply::Acked));
        if ticket == *tickets.last().expect("minted") {
            break;
        }
    }
    assert_eq!(post.settle().len(), tickets.len());
    assert_eq!(
        post.receipt(tickets[0]),
        None,
        "the oldest unread receipt is the one dropped"
    );
    assert!(post.receipt(*tickets.last().expect("minted")).is_some());
}

/// A window that has gone away: the poster's `next` ends, and a publish into a
/// dropped window says so rather than pretending.
#[test]
fn a_dropped_window_ends_the_outbox() {
    let (post, outbox) = pair();
    drop(post);
    assert!(outbox.next().is_none(), "nothing left to send");
    assert!(!outbox.publish(Ticket(0), Ok(Reply::Acked)));
}
