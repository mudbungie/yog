//! Which engine answered, and in whose spelling the two names crossed.

use super::*;

/// **The act path's whole routing** (§8.2): an entry is the answer to its leaf,
/// a name no entry holds is this window's own engine's, and a gesture naming no
/// workspace is too. All three against two live engines, so "which one
/// answered" is a fact and not an arrangement.
#[test]
fn a_gesture_goes_down_the_channel_its_workspace_names() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let there = engine(std::sync::Arc::new(Echo("host")));
    let dial = Dial::compose(&[there.entry("cobalt", "home")], here.window());

    assert_eq!(
        said(dial.answered(&about("cobalt"))),
        "host:home",
        "the entry answered, and the host's own name for the workspace crossed"
    );
    assert_eq!(
        said(dial.answered(&about("ops"))),
        "local:ops",
        "a name no entry holds goes where it always went, unrewritten"
    );
    assert_eq!(
        said(dial.answered(&unaddressed())),
        "local:-",
        "and so does a gesture naming no workspace at all"
    );
}

/// The **inbound** direction: a reply that identifies a workspace lands wearing
/// this box's name for it, so nothing above the channel boundary ever sees the
/// host's spelling.
#[test]
fn a_reply_lands_in_this_boxs_own_spelling() {
    let here = engine(std::sync::Arc::new(Lists("ops")));
    let there = engine(std::sync::Arc::new(Lists("home")));
    let dial = Dial::compose(&[there.entry("cobalt", "home")], here.window());
    let Ok(Reply::Workspaces(view)) = dial.answered(&about("cobalt")) else {
        panic!("the entry answered a roster");
    };
    assert_eq!(
        view.rows
            .iter()
            .map(|r| r.workspace.clone())
            .collect::<Vec<_>>(),
        vec!["cobalt".to_owned()],
        "renamed back to the leaf — the client's name is the only one above here"
    );
}

/// **An entry that exists is the answer to its name even when it cannot be
/// dialled** (§8.2, bl-4e31): its sentence stands in place of the answer, and
/// nothing falls through to another engine on the strength of a missing file.
/// The local channel is untouched by it — which is the whole isolation claim,
/// asserted rather than assumed.
#[test]
fn a_channel_that_cannot_be_dialled_refuses_only_itself() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let dial = Dial::compose(
        &[broken("cobalt"), unreachable(&here, "zinc")],
        here.window(),
    );

    assert_eq!(
        said(dial.answered(&about("cobalt"))),
        "cobalt is an empty entry",
        "the entry's own sentence, answered without a socket"
    );
    assert!(
        said(dial.answered(&about("zinc"))).contains("connect 127.0.0.1:1"),
        "and a host that is not there is the transport's, named with the address"
    );
    assert_eq!(
        said(dial.answered(&about("ops"))),
        "local:ops",
        "while this window's own engine answers exactly as it did"
    );
}

/// **The follow lane resolves rather than fans out** (§8.2): it is dialled at
/// whichever channel hosts the focused conversation, carrying that host's name.
#[test]
fn the_lane_dials_the_channel_that_hosts_the_subject() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let there = engine(std::sync::Arc::new(Echo("host")));
    let dial = Dial::compose(&[there.entry("cobalt", "home")], here.window());
    let following = codec::encode(&Gesture::Ask(Query::Follow {
        workspace: "cobalt".to_owned(),
        agent: "c-1".to_owned(),
    }));
    let mut frames = Vec::new();
    let held = dial.followed(&following, &mut |frame| {
        frames.push(said(frame));
        true
    });
    assert_eq!(held, Ok(()), "the engine terminated the stream");
    assert_eq!(
        frames,
        vec!["host:home".to_owned()],
        "held open at the entry's engine, under the host's name"
    );
}

/// A held read on a channel that cannot be dialled is the same one sentence a
/// routed ask earns — there is no second failure shape for a lane.
#[test]
fn a_lane_on_a_channel_that_cannot_be_dialled_says_so_once() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let dial = Dial::compose(&[broken("cobalt")], here.window());
    let following = codec::encode(&Gesture::Ask(Query::Follow {
        workspace: "cobalt".to_owned(),
        agent: "c-1".to_owned(),
    }));
    assert_eq!(
        dial.followed(&following, &mut |_| true),
        Err("cobalt is an empty entry".to_owned()),
        "and no frame was ever handed over"
    );
}

/// A window that holds no entry dials exactly one channel — §8.2's migration
/// clause, which is the shape every box had before entries existed.
#[test]
fn a_window_holding_no_entry_dials_one_channel() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let dial = Dial::of(here.window());
    assert_eq!(said(dial.answered(&about("cobalt"))), "local:cobalt");
}
