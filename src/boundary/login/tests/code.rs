//! **The act's second meaning, and the row on another machine** (REMOTE §8.3,
//! §5.7; bl-0127): a sign-in is one bare command line whichever box holds the
//! row, a run that printed a URL says so to every lane once, and the code a
//! human copied goes back through the same act to the run that is waiting.

use std::path::PathBuf;
use std::time::Duration;

use tempfile::tempdir;

use super::super::Frame;
use super::{ask, deps_with, lane, script, settle, wired, workspace};
use crate::boundary::Action;
use crate::boundary::dispatch::{Deps, dispatch};
use crate::boundary::reply::Reply;
use crate::cli_outbound::{Chunk, ExitInfo};
use crate::login::runs::Runs;
use crate::ui_state::UiState;

fn sign_in(deps: &Deps, provider: &str, code: Option<&str>) -> Result<Reply, String> {
    let mut ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let act = Action::Login {
        workspace: crate::naming::leaf(&workspace()),
        provider: provider.to_owned(),
        code: code.map(str::to_owned),
    };
    dispatch(deps, &mut ui, "100", &act)
}

/// **A qualified row is fired exactly as a bare one, and nothing is asked
/// first.** No machine is registered here and none is waiting: the act used to
/// refuse that (it had a catalog to read before it could choose a command
/// line), and now starts — the child carries the qualified name for the `bz`
/// arm to route, and a machine that does not answer says so in the lane.
#[test]
fn a_far_row_is_fired_as_a_bare_one_and_its_code_goes_to_the_waiting_run() {
    let dir = tempdir().expect("tmp");
    let bz = script(
        dir.path(),
        "bz",
        "#!/bin/sh\necho \"$*\" 1>&2\ncase \"$*\" in\n\
         *--code=*) echo 'logged in' 1>&2 ;;\n\
         *) echo 'https://x/authorize?state=s' ;;\nesac\n",
    );
    let deps = deps_with(dir.path(), &bz);
    let said =
        |view: &crate::login::LoginView, line: &str| view.lines.iter().any(|l| l.text == line);

    assert!(sign_in(&deps, "web@laptop", None).is_ok());
    let mut asked = ask(&deps, "web@laptop");
    for _ in 0..600 {
        if asked.authorize.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
        asked = ask(&deps, "web@laptop");
    }
    assert_eq!(
        asked.authorize.as_deref(),
        Some("https://x/authorize?state=s")
    );
    assert_eq!(asked.outcome, None, "a URL is not a sign-in");
    assert!(said(&asked, "--login --provider web@laptop"), "{asked:?}");

    // The code crosses as the act's own key and reaches the child as one word.
    let Ok(Reply::Login(receipt)) = sign_in(&deps, "web@laptop", Some("abc#def")) else {
        panic!("the finishing act answers a standing");
    };
    assert_eq!(receipt.authorize, None);
    let done = settle(&deps, "web@laptop");
    assert_eq!(done.outcome, Some(0));
    assert!(
        said(&done, "--login --provider web@laptop --code=abc#def"),
        "{done:?}"
    );

    // A code where nothing waits is the act's refusal, not a spawn.
    let refusal = sign_in(&deps, "other", Some("abc")).expect_err("nothing waits");
    assert!(refusal.contains("waiting for a code"), "{refusal}");
}

/// **A lane says a run came to wait exactly once.** The URL can arrive as a
/// line before the exit that makes it an `authorize`, so the standing changes
/// with nothing appended — and that is a frame, because a seat cannot act
/// without it. After it the lane is quiet again, and a lane opened later
/// replays the whole of it.
#[test]
fn a_lane_says_a_run_came_to_wait_for_a_code_once() {
    let dir = tempdir().expect("tmp");
    let runs = Runs::default();
    let (run, tx) = wired(dir.path());
    let serial = runs.seat(&workspace(), "web", run, 0);
    let mut held = lane(&runs, "web");
    assert!(
        matches!(held.poll(), Frame::Ready(_)),
        "it opens on the run"
    );

    tx.send(Chunk::Stdout(b"https://x/authorize\n".to_vec()))
        .expect("send");
    assert!(runs.read_once(&workspace(), "web", serial));
    let Frame::Ready(line) = held.poll() else {
        panic!("an append is a frame");
    };
    assert_eq!((line.lines.len(), line.authorize), (1, None));

    tx.send(Chunk::Exited(ExitInfo::Code(0))).expect("send");
    assert!(
        !runs.read_once(&workspace(), "web", serial),
        "a waiting run has nothing left to read"
    );
    let Frame::Ready(asking) = held.poll() else {
        panic!("coming to wait is a frame, with nothing appended");
    };
    assert!(asking.lines.is_empty());
    assert_eq!(asking.authorize.as_deref(), Some("https://x/authorize"));
    assert_eq!(asking.outcome, None);
    assert!(matches!(held.poll(), Frame::Waiting), "said once");

    let Some(Reply::Login(replayed)) = lane(&runs, "web").next() else {
        panic!("a later lane opens on the standing");
    };
    assert_eq!(replayed.lines.len(), 1);
    assert_eq!(replayed.authorize.as_deref(), Some("https://x/authorize"));
}
