//! The act's two meanings (REMOTE §8.3, bl-0127), over a real wall and a fake
//! brazen: with no code it starts the pair's sign-in — one command line for
//! every row, the row's own brazen picking the flow — and with one it
//! finishes the run that stands waiting, and no other.

use std::time::Duration;

use tempfile::tempdir;

use super::super::Runs;
use super::{lines, script, settled, wired, workspace};
use crate::cli_outbound::Cli;
use crate::login::LoginView;

/// A brazen that serves the pasted-code sign-in: the bare line prints a URL on
/// stdout that names the moment it was asked (so two starts are tellable
/// apart) and exits clean; a code is accepted or refused in brazen's words.
const BZ: &str = "#!/bin/sh\necho \"$*\" 1>&2\ncase \"$*\" in\n\
    *--code=good*) echo 'logged in' 1>&2 ;;\n\
    *--code=*) echo 'login failed: wrong code' 1>&2; exit 77 ;;\n\
    *) echo \"https://x/authorize?state=$$\" ;;\nesac\n";

/// Wait for the pair's run to stand waiting for a code, bounded like
/// [`settled`] and for its reason.
fn waiting(runs: &Runs, provider: &str) -> LoginView {
    for _ in 0..600 {
        let view = runs.standing(&workspace(), provider);
        if view.authorize.is_some() {
            return view;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("{provider}'s run never came to wait for a code");
}

/// **One command line for every row, and a code finishes the run that printed
/// the URL.** The argv is the assertion for the first half — no flow word, and
/// a qualified name handed on untouched for the `bz` arm to route — and the
/// one continuous buffer is the assertion for the second.
#[test]
fn a_start_fires_the_bare_line_and_a_code_finishes_the_waiting_run() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    let runs = Runs::of(Cli::new(script(dir.path(), "bz", BZ)));
    let act = |provider: &str, code: Option<&str>, ts: &str| {
        runs.start(&world, &workspace(), provider, code, state.path(), ts)
    };

    let receipt = act("sub@laptop", None, "100").expect("started");
    assert_eq!((receipt.outcome, receipt.authorize), (None, None));
    let asked = waiting(&runs, "sub@laptop");
    assert_eq!(asked.outcome, None, "a URL is not a sign-in");
    assert!(
        lines(&asked).contains(&"--login --provider sub@laptop".to_owned()),
        "{asked:?}"
    );

    // A refused code settles the run; what it said follows what was said.
    let receipt = act("sub@laptop", Some("bad"), "101").expect("fired");
    assert_eq!(
        receipt.authorize, None,
        "the receipt is the standing re-read"
    );
    let refused = settled(&runs, "sub@laptop");
    assert_eq!(refused.outcome, Some(77));
    let said = lines(&refused);
    assert!(
        said.contains(&"--login --provider sub@laptop --code=bad".to_owned()),
        "{said:?}"
    );
    assert_eq!(
        said.last().map(String::as_str),
        Some("login failed: wrong code")
    );
    assert_eq!(said.len(), 4, "one buffer across both command lines");

    // A settled run takes no second code: the sign-in starts over instead.
    let spent = act("sub@laptop", Some("good"), "102").expect_err("nothing waits");
    assert!(spent.contains("/login sub@laptop"), "{spent}");
    act("sub@laptop", None, "103").expect("restarted");
    waiting(&runs, "sub@laptop");
    act("sub@laptop", Some("good"), "104").expect("fired");
    let done = settled(&runs, "sub@laptop");
    assert_eq!((done.outcome, done.fallback), (Some(0), None));
    assert_eq!(crate::opslog::tail(state.path(), 8).len(), 4);
}

/// **A code is taken only by a run that is waiting for one.** A pair nobody
/// started and a run still live — a device flow polling — both refuse it,
/// naming the restart, and the live run is left exactly as it stood.
#[test]
fn a_code_with_no_run_waiting_for_it_refuses_and_touches_nothing() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    let runs = Runs::of(Cli::new("/definitely/not/a/real/bz-xyz"));
    let code = |provider: &str| {
        runs.start(
            &world,
            &workspace(),
            provider,
            Some("c"),
            state.path(),
            "100",
        )
    };

    let nobody = code("openai").expect_err("no run");
    assert!(
        nobody.contains("no sign-in for \"openai\" is waiting for a code"),
        "{nobody}"
    );
    assert_eq!(runs.standing(&workspace(), "openai"), LoginView::default());

    // The sender is held: a dropped one would end the wired run by itself.
    let (run, _tx) = wired(dir.path());
    let serial = runs.seat(&workspace(), "live", run, 100);
    assert!(code("live").is_err(), "a live run is not a waiting one");
    assert!(
        runs.read_once(&workspace(), "live", serial),
        "its reader still owns it: the refusal re-stamped nothing"
    );
    assert!(
        crate::opslog::tail(state.path(), 8).is_empty(),
        "nothing ran"
    );
}

/// **Starting again is the cancel, and only the newest URL stands.** Two seats
/// starting the same sign-in share the pair's one run: the second start
/// replaces the first, so the standing every lane re-reads is the newer URL.
#[test]
fn a_second_start_replaces_a_waiting_run_with_a_new_url() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    let runs = Runs::of(Cli::new(script(dir.path(), "bz", BZ)));
    let start = |ts: &str| {
        runs.start(&world, &workspace(), "web", None, state.path(), ts)
            .expect("started")
    };

    start("100");
    let first = waiting(&runs, "web").authorize;
    let receipt = start("101");
    assert_eq!(receipt.authorize, None, "the older URL went with its run");
    let second = waiting(&runs, "web");
    assert_ne!(second.authorize, first, "a new start is a new URL");
    assert_eq!(lines(&second).len(), 2, "and a buffer that starts over");
}
