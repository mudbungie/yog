//! The frame's act half: what a click posts, which root its receipt re-derives,
//! and the one `Err` a window with no engine behind it earns.

use crate::app::tests::Harness;
use crate::boundary::reply::Reply;
use crate::boundary::{Action, Gesture, codec};
use crate::watch::Mark;

/// A `lernie` short verb — names no project, so its substrate root is the yog
/// state root the ops trail is written under; it does name the fixture's one
/// enumerated workspace.
fn nudge() -> Action {
    Action::Nudge {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
    }
}

/// **What crosses is the gesture, and only the gesture.** The envelope a post
/// puts on the wire is the codec's own `Act` spelling — the same bytes a phone
/// seat would send — so a posted act carries no path, no binary and nothing
/// this box resolved.
#[test]
fn a_post_puts_the_codec_spelling_of_the_act_on_the_wire() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    let (post, outbox) = crate::wire::post::pair();
    rig.model.adopt_post(post);
    let ticket = rig.model.post_act(&nudge());
    let (sent, envelope) = outbox.next().expect("the act is queued at once");
    assert_eq!(sent, ticket);
    assert_eq!(envelope, codec::encode(&Gesture::Act(nudge())));
}

/// **The aftermath belongs to the receipt.** The roots a `lernie` verb touches
/// are marked dirty when the engine says it is done — not at the click, where
/// the act has not happened yet.
#[test]
fn the_root_is_re_derived_when_the_receipt_lands_and_not_before() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    let (post, outbox) = crate::wire::post::pair();
    rig.model.adopt_post(post);
    let ticket = rig.model.post_act(&nudge());
    rig.model.settle_acts();
    assert!(
        rig.model.dirty_handle().drain().is_empty(),
        "the act has not happened yet, so nothing has changed to re-read"
    );

    let (sent, _) = outbox.next().expect("queued");
    outbox.publish(sent, Ok(Reply::Nudged));
    rig.model.settle_acts();
    assert_eq!(
        rig.model.dirty_handle().drain(),
        [
            (h.roots.yog_state.clone(), Mark::Watch),
            (h.ws.clone(), Mark::Watch)
        ]
        .into_iter()
        .collect(),
        "a lernie verb's ops line lands under the yog state root, and its \
         effect lands in the workspace it named (bl-18e8)"
    );
    assert_eq!(rig.model.act_receipt(ticket), Some(Ok(Reply::Nudged)));
}

/// A ball verb names a **project**, so its receipt re-derives that project's own
/// root — the fact the *action* carries, read off it rather than decided again
/// at each call site. A name the enumeration does not hold falls back to the
/// root every gesture's trail is written under: refusing an unknown name is the
/// engine's job, and this side only has to know what to re-read.
#[test]
fn a_ball_verb_re_derives_the_project_it_named() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    let project = h.roots.home.join("dev").join("proj");
    rig.deriver.projects = vec![project.clone()];
    rig.publish();
    let (post, outbox) = crate::wire::post::pair();
    rig.model.adopt_post(post);
    let close = |name: &str| Action::Close {
        project: name.to_owned(),
        id: "bl-0000".to_owned(),
        name: "ws".to_owned(),
    };
    let known = rig.model.post_act(&close("proj"));
    let unknown = rig.model.post_act(&close("nowhere"));
    for expected in [project, h.roots.yog_state.clone()] {
        let (sent, _) = outbox.next().expect("queued");
        outbox.publish(sent, Ok(Reply::Acked));
        rig.model.settle_acts();
        assert_eq!(
            rig.model.dirty_handle().drain(),
            [(expected, Mark::Watch)].into_iter().collect()
        );
    }
    assert!(rig.model.act_receipt(known).is_some());
    assert!(rig.model.act_receipt(unknown).is_some());
}

/// **A conversation act re-derives the workspace it deposited into** (bl-18e8).
/// The mail lands under the driver's own lock, so the instant it does, disk says
/// "a delivered message on the tail" while the published snapshot still says
/// nobody is driving — the §13.3 orphan alarm's rising edge. The act that
/// created that state is the one thing that knows the workspace changed, so the
/// deposit requests the catch-up that clears its own alarm rather than waiting
/// on a sweep. The ops line still lands under the yog state root, so both roots
/// are named, not one instead of the other.
#[test]
fn a_conversation_act_re_derives_its_workspace_as_well_as_the_trail() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    let (post, outbox) = crate::wire::post::pair();
    rig.model.adopt_post(post);
    let ticket = rig.model.post_act(&Action::Message {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        content: "hello?".to_owned(),
    });
    let (sent, _) = outbox.next().expect("queued");
    outbox.publish(sent, Ok(Reply::Acked));
    rig.model.settle_acts();
    assert_eq!(
        rig.model.dirty_handle().drain(),
        [
            (h.roots.yog_state.clone(), Mark::Watch),
            (h.ws.clone(), Mark::Watch)
        ]
        .into_iter()
        .collect(),
        "the trail's root and the workspace the mail landed in"
    );
    assert!(rig.model.act_receipt(ticket).is_some());
}

/// A workspace name the enumeration does not hold names no root of its own —
/// the same fallback the project half takes, for the same reason: refusing an
/// unknown name is the engine's job, and this side only has to know what to
/// re-read.
#[test]
fn an_unenumerated_workspace_leaves_only_the_trails_root() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    let (post, outbox) = crate::wire::post::pair();
    rig.model.adopt_post(post);
    rig.model.post_act(&Action::Scan {
        workspace: "nowhere".to_owned(),
    });
    let (sent, _) = outbox.next().expect("queued");
    outbox.publish(sent, Ok(Reply::Acked));
    rig.model.settle_acts();
    assert_eq!(
        rig.model.dirty_handle().drain(),
        [(h.roots.yog_state.clone(), Mark::Watch)]
            .into_iter()
            .collect()
    );
}

/// **A window with no engine behind it is the same code path**, one frame
/// shorter: the model boots holding a post nobody drains, and the act's receipt
/// is the sentence saying so.
#[test]
fn a_model_with_no_wire_answers_its_own_post() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    let ticket = rig.model.post_act(&nudge());
    let Some(Err(said)) = rig.model.act_receipt(ticket) else {
        panic!("the send is its own receipt with nobody behind it");
    };
    assert!(said.contains("no wire"), "{said}");
}
