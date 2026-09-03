//! The follow-class door's **second** lane (REMOTE §8.3, bl-c285): a sign-in's
//! output, opened at the one intake that can hold a connection.
//!
//! Its own file beside [`scope`](super::scope) rather than inside it, on the
//! seam production is cut along ([`consumer::lanes`](super::super::lanes)):
//! that file's subject is what a certificate may *see*, and this one's is what
//! a held read may *be*. What the two lanes share — the resolution, the scope
//! spent at connect, the `None` that falls through to the one-frame refusal —
//! is asserted once, there.

use super::{over, seat, world_of};
use serde_json::json;
use tempfile::tempdir;

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
