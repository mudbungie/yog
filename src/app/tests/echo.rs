//! The §7.2 **pending echo** at the model, beside the §3.4 claim it is one
//! value with: what a follow-up's echo paints, what it refuses to move, and the
//! two reconciliations that retire it. Split from [`super::started`] at §12's
//! per-file budget on the seam those two nouns already draw — the claim is about
//! the *focus* a start takes, this is about the *deposit* a send stands in for.

use super::Harness;
use crate::watch::Mark;
use std::time::Duration;

/// The §7.2 echo the §8.2 `message` verb leaves, at the model: the same one
/// pending value, addressing an agent instead of a name — so it paints on the
/// conversation it was aimed at, retires on the message landing, and **moves no
/// focus**. The operator was already there; their own message landing must not
/// drag them back from wherever they have since gone.
#[test]
fn a_message_echoes_without_claiming_the_focus() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    model.focus_agent(&h.ws, "c-1");
    let landed = model
        .tree(&h.ws)
        .and_then(|t| t.agents.first().map(|a| a.messages))
        .expect("the fixture's conversation");

    model.await_message(&h.ws, "c-1", "and again", 0);
    model.refresh();
    let echoed = model
        .tree(&h.ws)
        .and_then(|t| t.agents.first().cloned())
        .expect("the folded row");
    assert_eq!(
        echoed.pending.len(),
        1,
        "the send is on the row as the undelivered deposit it is (§5.1 #11)"
    );
    assert!(
        echoed.pending[0].in_memory(),
        "and it is yog's word, not disk's"
    );
    assert_eq!(echoed.pending[0].deposit.body, "and again");

    // The operator walks off it while the driver is still cold.
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    model.refresh();
    assert!(
        model.focus().agent.is_none(),
        "a follow-up's echo carries no focus claim to spend"
    );

    // The driver flushes it: the echo retires and the row is the derivation's.
    let messages = h.ws.join("agents/c-1/messages");
    std::fs::create_dir_all(&messages).unwrap();
    std::fs::write(
        messages.join(format!("{:03}-user.md", landed + 1)),
        "and again",
    )
    .unwrap();
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    model.tick();
    clock.advance(Duration::from_millis(150));
    assert!(model.tick(), "the derivation carries the message now");
    assert!(
        model
            .tree(&h.ws)
            .and_then(|t| t.agents.first())
            .is_some_and(|a| a.pending.is_empty()),
        "the echo gave its seat up rather than doubling the message"
    );
}

/// The fold runs only when one of its two inputs moved (§7.2). The rendered
/// `Arc` is `SnapMemo`'s invalidation key, so a fold that allocated per frame
/// would rebuild the transcript per frame — the 35 ms/frame cost bl-e90a
/// removed, reintroduced by the echo.
#[test]
fn an_idle_frame_refolds_nothing_and_hands_back_the_same_pointer() {
    use std::sync::Arc;
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.refresh();
    let quiet = Arc::clone(&model.snap);
    model.refresh();
    assert!(
        Arc::ptr_eq(&quiet, &model.snap),
        "nothing pending, nothing published: the same snapshot, not a copy of it"
    );

    model.await_message(&h.ws, "c-1", "and again", 0);
    model.refresh();
    let folded = Arc::clone(&model.snap);
    assert!(!Arc::ptr_eq(&quiet, &folded), "a new echo is a new fold");
    model.refresh();
    assert!(
        Arc::ptr_eq(&folded, &model.snap),
        "and holding the same echo over the same derivation folds nothing again"
    );
}

/// One deposit as the answered §11 listing spells it.
fn listed(name: &str, body: &str) -> crate::inboxview::InboxEntry {
    crate::inboxview::InboxEntry {
        name: name.into(),
        raw: body.as_bytes().to_vec(),
        deposit: crate::inboxview::Deposit {
            sender: Some("user".into()),
            body: body.into(),
            ..crate::inboxview::Deposit::default()
        },
    }
}

/// **The queue seat's own reconciliation** (§7.2, bl-78d8), which is the third
/// projection's half of `rows`' *"freshened and never duplicated"*: the echo
/// folds onto the answered listing while that listing is still the one the act
/// was queued against, and yields the moment it grows — because what grew it is
/// the deposit this echo stands for.
///
/// The key is the listing's **length**, never its text. `landed` cannot serve
/// here and that is the whole reason there are two predicates: the §8.2 verb is
/// piped, so the file is on disk before the receipt mints the echo, while
/// `messages/` only moves at the driver's next step boundary — seconds in which
/// the queue held the solid deposit and the faded echo side by side, saying the
/// same words.
#[test]
fn the_queue_echo_yields_the_moment_the_answer_carries_its_deposit() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.focus_agent(&h.ws, "c-1");
    // Queued against a seat showing one deposit — the baseline the act carries.
    model.await_message(&h.ws, "c-1", "and again", 1);
    let stale = vec![listed("user-001.md", "already mail")];
    let echoed = model.echoed_pending("c-1", stale.clone());
    assert_eq!(echoed.len(), 2, "the answer has not moved: the echo shows");
    assert!(echoed[1].in_memory(), "and it is yog's word, not disk's");
    assert_eq!(echoed[1].deposit.body, "and again");

    // The listing grows: the file the echo stood in for is in the answer.
    let mut fresh = stale.clone();
    fresh.push(listed("user-002.md", "and again"));
    let yielded = model.echoed_pending("c-1", fresh.clone());
    assert_eq!(yielded, fresh, "one row, and it is the deposit's own");
    assert!(
        !yielded.iter().any(crate::inboxview::InboxEntry::in_memory),
        "nothing faded is left beside it (§7.2)"
    );

    // Another conversation's queue is not this echo's seat, however long its
    // own listing is — the target is matched before the count is read.
    assert_eq!(model.echoed_pending("c-2", stale.clone()), stale);
}
