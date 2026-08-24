//! **The detached driver's own verdict** (§8.1, §13.3) — the one op class whose
//! outcome is not in its exit code, split off [`super`] at the cap on the seam
//! it already had: every row there is judged by what the verb returned, and
//! these two are judged by what a sink file said afterwards.
//!
//! A `lernie prompt` launches detached, so its line lands clean at
//! [`DETACHED_EXIT`] and the only evidence of what happened next is the captured
//! stderr the next sweep folds in. Both directions stand here, because the fold
//! must make a death a failure **and** must leave a benign notice alone: the two
//! beats are one another's burden check.

use super::{model, world};
use crate::opslog::{self, DETACHED_EXIT, Origin};
use std::fs;
use std::path::Path;

#[test]
fn a_detached_prompt_that_died_after_launch_surfaces_on_the_next_sweep() {
    let w = world();
    let (_c, mut m) = model(&w);
    let ws = Path::new("/ws/cobalt-gecko");
    opslog::append(
        m.state_root(),
        &opslog::OpEntry {
            ts: "17".into(),
            argv: vec![
                "lernie".into(),
                "prompt".into(),
                ws.to_string_lossy().into_owned(),
                "goal".into(),
            ],
            cwd: "/ws".into(),
            exit: DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: Origin::Conversation,
        },
    )
    .unwrap();
    // The launch alone: a clean `-2` row, nothing to see — the §13.3 hole.
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    assert!(m.last_failure(Origin::Conversation).is_none());
    assert!(!m.snap.ops.last().unwrap().failed());
    assert!(!m.snap.ops.last().unwrap().has_output());

    // The driver then refuses and dies, leaving its cry in the sink file.
    let sink = opslog::detached::sink(m.state_root(), "17", ws);
    fs::create_dir_all(sink.parent().unwrap()).unwrap();
    fs::write(&sink, "brazen 0.0.2 refuses 0.0.3\n").unwrap();

    // The next sweep folds it in: the row becomes a rendered failure and the
    // originating surface — the one that fired the bare rung, and only it —
    // gets its ichor-red banner (§7.3). No new ops line.
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    let row = m.snap.ops.last().unwrap();
    assert!(
        row.failed(),
        "the dead driver's stderr makes the row a failure"
    );
    assert!(row.has_output(), "and the pane offers the expansion");
    assert_eq!(
        m.last_failure(Origin::Conversation).unwrap().stderr_tail,
        "brazen 0.0.2 refuses 0.0.3"
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

/// **THE BALL** (bl-1296): the same sweep, over a sink holding a benign lernie
/// driver notice. The row folds the tail in and stays out of every alarm — no
/// §7.3 banner on the surface that fired it, nothing in the chip's ⚠ count —
/// while the expandable ops row keeps the text. The dead-driver beat above is
/// the arm this must not weaken, which is why the two stand side by side.
#[test]
fn a_detached_prompt_whose_driver_only_filed_a_notice_never_banners() {
    let w = world();
    let (_c, mut m) = model(&w);
    let ws = Path::new("/ws/cobalt-gecko");
    opslog::append(
        m.state_root(),
        &opslog::OpEntry {
            ts: "19".into(),
            argv: vec![
                "lernie".into(),
                "prompt".into(),
                ws.to_string_lossy().into_owned(),
                "goal".into(),
            ],
            cwd: "/ws".into(),
            exit: DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: Origin::Conversation,
        },
    )
    .unwrap();

    // The driver lands a compaction decline and carries on, into its §8.1 sink.
    let notice = "lernie: compaction landing [c-2] superseded — a compaction landed \
                  since its fork point (ARCH §2.6); the branch continues\n";
    let sink = opslog::detached::sink(m.state_root(), "19", ws);
    fs::create_dir_all(sink.parent().unwrap()).unwrap();
    fs::write(&sink, notice).unwrap();

    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    let row = m.snap.ops.last().unwrap();
    assert!(!row.failed(), "a benign notice is not a rendered failure");
    assert_eq!(row.stderr, notice, "and the trail still carries its words");
    assert!(row.has_output(), "so the pane offers the expansion");
    assert!(
        m.last_failure(Origin::Conversation).is_none(),
        "the surface that fired it is not bannered"
    );
    assert_eq!(m.activity().errors, 0, "and the chip counts no failure");
}
