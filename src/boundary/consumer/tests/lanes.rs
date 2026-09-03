//! **The held lanes at the scoped intake** (REMOTE §3, §4, §8.3, §14.1;
//! bl-73e7, bl-c285, bl-09aa): which asks become a stream, what a stream costs
//! a caller that may not have one, and the fall-through that words every
//! refusal.
//!
//! Split from [`scope`](super::scope) at §12's cap on the seam
//! `consumer/lanes.rs` already draws: that file's subject is what a certificate
//! may *see*, and this one's is what a held read may *be*. What every lane
//! shares — the resolution, the scope spent at connect, the `None` that falls
//! through to the one-frame refusal — is asserted once, here.

use super::*;
use crate::cli_outbound::Cli;
use serde_json::json;
use tempfile::tempdir;

/// **A held read is one request, and it spends the scope like one** (REMOTE §4,
/// bl-73e7). The address resolves at connect, under the caller's registrations,
/// so an unregistered workspace answers `None` here — and `None` is not a
/// second refusal path: the intake falls back to the one-frame answer, which
/// refuses in the resolver's own words. A seat cannot tell a refused follow
/// from any other refused read, which is exactly REMOTE §4's absence.
#[test]
fn a_follow_resolves_its_address_under_the_callers_scope() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let ctx = over(
        root.path(),
        world_of(data.path(), &["home", "corp"]),
        data.path().to_path_buf(),
        Cli::new("/no/such/litany"),
    );
    let phone = seat("phone");
    crate::registry::register(root.path(), &phone.client, "home").unwrap();
    let follow = |workspace: &str, agent: &str| json!({"op": "follow", "workspace": workspace, "agent": agent});

    // Nothing this seat may not see: the workspace it is not seated in, and a
    // conversation the workspace does not carry, both answer no stream.
    assert!(ctx.follow(&phone, &follow("corp", "c-1")).is_none());
    assert!(ctx.follow(&phone, &follow("home", "nobody")).is_none());
    // And the refusal a seat actually reads is the resolver's, one frame long.
    let said = |workspace: &str| {
        ctx.answer_as(&phone, &follow(workspace, "c-1"))["error"]
            .as_str()
            .unwrap_or_default()
            .replace(workspace, "<name>")
    };
    assert_eq!(
        said("corp"),
        said("nowhere"),
        "absence, not a scope error: a workspace this seat is not seated in \
         refuses in the identical bytes one nobody founded earns"
    );
}

/// Every other request is `None` here too, and that is the whole of what makes
/// the intake two arms rather than three: a read that is not follow-class is
/// answered by the one function that answers everything else.
#[test]
fn nothing_but_a_held_lane_is_a_stream() {
    let root = tempdir().unwrap();
    let ctx = ctx(root.path());
    for request in [
        json!({"op": "workspaces"}),
        json!({"op": "teleport"}),
        json!("not even an object"),
    ] {
        assert!(ctx.follow(&seat("phone"), &request).is_none());
    }
}

/// **The attention lane addresses nothing, so it can never be a read nobody can
/// answer** (REMOTE §14.1, bl-09aa): unlike a follow, it has no workspace to
/// resolve and no conversation to find, so a seat registered in nothing is
/// answered a lane — whose frames are empty — rather than a refusal. That is
/// REMOTE §4's absence, said as a stream.
#[test]
fn an_attention_ask_opens_a_lane_for_any_seat() {
    let root = tempdir().unwrap();
    let ctx = ctx(root.path());
    let request = json!({"op": "attention"});
    let unseated = seat("phone");
    let mut lane = ctx
        .follow(&unseated, &request)
        .expect("attention is follow-class");
    assert_eq!(
        lane.next(),
        Some(json!({"ok": true, "kind": "attention", "rows": []})),
        "the answer as of connect, in the same bytes a one-frame read carries"
    );
    // And the wire's intake takes that same door — the lane, not the one-frame
    // answer beside it.
    let intake = crate::wire::intake::Intake::new(std::sync::Arc::new(ctx));
    assert_eq!(
        crate::wire::server::Answerer::answer(&intake, &unseated, request)
            .take(1)
            .count(),
        1,
        "the lane's first frame is written as it is produced"
    );
}

/// **A resolvable follow really opens a stream** — the other side of the beat
/// above. The address resolves under the caller's registrations, the frames
/// are the §7.2 tail's, and the intake hands them back as a stream rather than
/// as one answer.
///
/// **Nothing here polls the stream**, deliberately: a hold that has nothing to
/// say waits out its own patience before it ends (`follow::tests::reading`
/// drives that, with the patience injected), and a beat about *resolution*
/// must not pay a minute for it. What is pinned is that the resolution
/// succeeded and produced an iterator, since `None` is the one thing a caller
/// cannot tell apart from a refusal.
#[test]
fn a_resolvable_follow_opens_a_stream_rather_than_answering_once() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let mut snap = world_of(data.path(), &["home"]);
    let ws = crate::binding::workspace_path(data.path(), "home");
    snap.trees.insert(
        ws,
        crate::git_tree::GitTree {
            commits: vec![],
            agents: vec![crate::boundary::tests::agent(
                "c-1",
                crate::git_tree::AgentState::Quiescent,
                100,
            )],
        },
    );
    let ctx = over(
        root.path(),
        snap,
        data.path().to_path_buf(),
        Cli::new("/no/such/litany"),
    );
    let phone = seat("phone");
    crate::registry::register(root.path(), &phone.client, "home").unwrap();
    let request = json!({"op": "follow", "workspace": "home", "agent": "c-1"});

    assert!(
        ctx.follow(&phone, &request).is_some(),
        "the address resolved, so there is a stream"
    );
    // And the wire's intake takes that same door: a request the consumer can
    // follow becomes that stream, and one it cannot becomes the single-frame
    // answer beside it.
    let intake = crate::wire::intake::Intake::new(std::sync::Arc::new(ctx));
    assert_eq!(
        crate::wire::server::Answerer::answer(&intake, &phone, request)
            .take(0)
            .count(),
        0,
        "the follow arm hands the stream through untouched"
    );
    assert_eq!(
        crate::wire::server::Answerer::answer(&intake, &phone, json!({"op": "workspaces"})).count(),
        1,
        "an ordinary read is one frame"
    );
}

/// A resolvable sign-in lane opens a stream, and its frames are the run's.
///
/// **Nothing here polls it past the opening frame**, deliberately: a lane with
/// nothing further to say waits out its own patience before it ends, and
/// `boundary::login::tests` drives that with the patience injected. What is
/// pinned here is that the second follow-class read reaches the door at all —
/// `None` is the one answer a caller cannot tell apart from a refusal.
#[test]
fn a_sign_in_lane_opens_a_stream_and_its_first_frame_is_the_standing() {
    let root = tempdir().expect("tmp");
    let data = tempdir().expect("tmp");
    let ctx = over(
        root.path(),
        world_of(data.path(), &["home"]),
        data.path().to_path_buf(),
        crate::cli_outbound::Cli::new("/no/such/litany"),
    );
    let phone = seat("phone");
    crate::registry::register(root.path(), &phone.client, "home").expect("registered");
    let request = json!({"op": "login-tail", "workspace": "home", "provider": "acme"});

    let mut frames = ctx.follow(&phone, &request).expect("the address resolved");
    // A row nobody has signed in to is emptiness said out loud, not silence: a
    // seat must not have to tell "never signed in" from "the lane died".
    assert_eq!(
        frames.next(),
        Some(json!({"ok": true, "kind": "login", "lines": []})),
        "the lane opens on the standing"
    );

    // A workspace this seat is not seated in answers no stream, exactly as the
    // tail lane's does — one door, one scope, one fall-through.
    let elsewhere = json!({"op": "login-tail", "workspace": "corp", "provider": "acme"});
    assert!(ctx.follow(&phone, &elsewhere).is_none());
}
