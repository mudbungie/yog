//! **The detached driver's own verdict** (§8.1, §13.3) — the one op class whose
//! outcome is not in its exit code, split off [`super`] at the cap on the seam
//! it already had: every row there is judged by what the verb returned, and
//! these two are judged by the world the launch left behind.
//!
//! A `lernie prompt` launches detached, so its line lands clean at
//! [`DETACHED_EXIT`] and nothing about what happened next is in it. Since
//! bl-b95e the sweep asks the **state** — is the conversation the row named
//! there, and is anything driving it — and reads the §8.1 sink only when the
//! answer is no. Both directions stand here, because the gate must make a
//! stillbirth a failure **and** must leave a live launch alone whatever its
//! sink says: the two beats are one another's burden check.

use super::{model, world};
use crate::git_tree::tests::fixture::Fixture;
use crate::opslog::{self, DETACHED_EXIT, Origin};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A **derivable** workspace under this world's names root: a real git
/// fixture with no conversations, symlinked where the model enumerates it.
/// The shared world's two workspaces are bare directories holding an empty
/// `repo.git`, so they derive no tree at all — and the §8.1 verdict is a
/// question about a derived tree, never about a path. Call before `model`, so
/// the boot pass enumerates it.
fn derivable(w: &super::super::World) -> (Fixture, PathBuf) {
    let fx = Fixture::new();
    let ws = w.roots.yog_data.join("workspaces").join("gecko");
    std::os::unix::fs::symlink(&fx.path, &ws).unwrap();
    (fx, ws)
}

/// The §3.3 name the fire minted, carried on the row's `--name`.
const MINTED: &str = "vanished-heron";

/// The sink line both beats use — deliberately the same bytes, because the
/// claim is that the bytes decide nothing.
const CRY: &str = "brazen 0.0.2 refuses 0.0.3\n";

/// One detached `lernie prompt` line for `ws`, written as
/// `start::execute_prompt` writes it: `--name` up front, the goal last.
fn launch(m: &crate::AppModel, ts: &str, ws: &Path) {
    opslog::append(
        m.state_root(),
        &opslog::OpEntry {
            ts: ts.into(),
            argv: vec![
                "lernie".into(),
                "prompt".into(),
                "--name".into(),
                MINTED.into(),
                ws.to_string_lossy().into_owned(),
                "goal".into(),
            ],
            cwd: ws.to_string_lossy().into_owned(),
            exit: DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: Origin::Conversation,
        },
    )
    .unwrap();
}

/// Write the dead driver's cry into the sink the launch at `ts` routed to.
fn cry(m: &crate::AppModel, ts: &str, ws: &Path) {
    let sink = opslog::detached::sink(m.state_root(), ts, ws);
    fs::create_dir_all(sink.parent().unwrap()).unwrap();
    fs::write(&sink, CRY).unwrap();
}

#[test]
fn a_detached_prompt_that_died_after_launch_surfaces_on_the_next_sweep() {
    let w = world();
    let (_fx, ws) = derivable(&w);
    let (clock, mut m) = model(&w);
    launch(&m, "0", &ws);
    // Past the §7.3 grace window, so the launch has had its time to produce
    // something. `FakeClock` counts unix seconds from its own origin.
    clock.advance(Duration::from_mins(10));
    // The launch alone: a clean `-2` row, nothing to see — the §13.3 hole.
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    assert!(m.last_failure(Origin::Conversation).is_none());
    assert!(!m.snap.ops.last().unwrap().failed());
    assert!(!m.snap.ops.last().unwrap().has_output());

    // The driver then refuses and dies, leaving its cry in the sink file. The
    // workspace holds no conversation by that name and never will, so the state
    // says the launch produced nothing.
    cry(&m, "0", &ws);

    // The next sweep folds it in: the row becomes a rendered failure and the
    // originating surface — the one that fired the bare rung, and only it —
    // gets its ichor-red banner (§7.3). No new ops line.
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    let row = m.snap.ops.last().unwrap();
    assert!(row.failed(), "a launch that produced nothing is a failure");
    assert!(row.has_output(), "and the pane offers the expansion");
    assert_eq!(
        m.last_failure(Origin::Conversation).unwrap().stderr_tail,
        CRY.trim_end()
    );
    assert!(
        m.last_failure(Origin::Balls).is_none(),
        "the balls fold offered no bare rung and says nothing"
    );
    assert_eq!(
        opslog::tail(m.state_root(), 8).len(),
        1,
        "no line rewritten"
    );
}

/// **THE BALL** (bl-b95e): the same sweep, the same sink bytes, over a launch
/// **younger than the §7.3 grace window**. Nothing is folded, so nothing
/// alarms — the rising edge of a healthy start is indistinguishable from a
/// death until yog has had time to look (bl-18e8), and the sink's words cannot
/// close that gap because they are never the trigger. The beat above is the arm
/// this must not weaken, which is why the two stand side by side over one
/// string.
#[test]
fn a_launch_inside_the_grace_window_never_alarms_however_its_sink_reads() {
    let w = world();
    let (_fx, ws) = derivable(&w);
    let (_c, mut m) = model(&w);
    // Stamped at the clock's own origin and never advanced: a launch that has
    // only just happened.
    launch(&m, "0", &ws);
    cry(&m, "0", &ws);

    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    let row = m.snap.ops.last().unwrap();
    assert!(
        !row.failed(),
        "a launch still inside its grace is no failure"
    );
    assert!(row.stderr.is_empty(), "and its sink was never read");
    assert!(!row.has_output());
    assert!(
        m.last_failure(Origin::Conversation).is_none(),
        "the surface that fired it is not bannered"
    );
    assert_eq!(m.activity().errors, 0, "and the chip counts no failure");
}
