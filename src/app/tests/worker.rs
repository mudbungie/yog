//! The derivation thread and the two honesty surfaces that only exist because
//! of it (DESIGN §7.2, bl-ee0a): a late pass names itself, a stale snapshot says
//! how stale, and a growing conversation is named on the ops surface.
//!
//! The pass itself is driven by hand everywhere else in this suite. What is
//! proven here is the shell around it — that a real thread picks work up,
//! publishes, wakes the window, and stops cleanly.

use super::{Harness, Rig};
use crate::app::Worker;
use crate::test_support::FakeClock;
use crate::ui_state::SystemClock;
use crate::watch::{Mark, Repaint};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// A [`Repaint`] double counting requests — the window the worker would wake.
struct CountingRepaint(Arc<AtomicUsize>);

impl Repaint for CountingRepaint {
    fn request(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

/// Poll `probe` until it yields or `timeout` elapses. The worker is a real
/// thread here; nothing else in this file waits on one.
fn wait_until(timeout: Duration, mut probe: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if probe() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    probe()
}

#[test]
fn wait_until_gives_up_and_reports_the_last_reading() {
    assert!(!wait_until(Duration::from_millis(20), || false));
}

#[test]
fn the_worker_thread_derives_a_marked_root_and_wakes_the_window() {
    let h = Harness::new();
    // A real clock: this test is about the thread, and the pass it runs must
    // reach its own debounce without a test advancing anything.
    let (mut model, deriver) = crate::AppModel::boot(
        h.roots.clone(),
        None,
        Arc::new(SystemClock),
        Box::new(super::harness::no_balls()),
        Some("me".to_string()),
    );
    let dirty = deriver.dirty_handle();
    let count = Arc::new(AtomicUsize::new(0));
    let worker = Worker::spawn(deriver, CountingRepaint(Arc::clone(&count)));

    // Disk moves and the root is announced; the worker — nobody else — derives.
    h.build_more("c-2", "yo");
    dirty.mark_all([(h.ws.clone(), Mark::Watch)]);
    assert!(
        wait_until(Duration::from_secs(5), || count.load(Ordering::Relaxed) > 0),
        "the worker published and asked for a repaint"
    );
    assert!(
        wait_until(Duration::from_secs(5), || {
            model.refresh();
            model.tree(&h.ws).is_some_and(|t| t.agents.len() == 2)
        }),
        "and the frame's snapshot carries the new agent"
    );
    drop(worker); // clean stop + join
}

/// Every `yog-drift late` row on the trail, in order.
fn late_rows(rig: &Rig) -> Vec<String> {
    rig.snap
        .ops
        .iter()
        .filter(|r| r.drift())
        .map(|r| r.argv.clone())
        .filter(|argv| argv.ends_with("late"))
        .collect()
}

#[test]
fn a_pass_that_outruns_its_cadence_names_itself_once_however_long_it_lasts() {
    // §7.2 as rewritten: the frame renders the last *completed* derivation, so a
    // pass that takes longer than the cadence it promised means everything on
    // screen was that far behind. That is drift, and it is written down rather
    // than felt as a frozen window.
    let h = Harness::new();
    let clock = FakeClock::lurching(Duration::from_secs(1));
    let (model, deriver) = crate::AppModel::boot(
        h.roots.clone(),
        None,
        clock.arc(),
        Box::new(super::harness::no_balls()),
        Some("me".to_string()),
    );
    let mut rig = Rig { model, deriver };
    rig.tick();
    assert_eq!(late_rows(&rig).len(), 1, "one late-pass line");
    // bl-4b28 — and it stays one. This clock never speeds up, so every pass
    // after it is late too; a row apiece is what turned a 4472-row trail into
    // 4447 restatements of the same sentence and 25 rows of real history.
    rig.tick();
    rig.tick();
    let late = late_rows(&rig);
    assert_eq!(
        late.len(),
        1,
        "a derivation that never keeps cadence is ONE event: {late:?}"
    );
}

#[test]
fn the_ops_surface_says_how_stale_the_rendered_derivation_is() {
    // **The `Query::Workspaces` answer carries it** (bl-b4b5): the seat folds
    // the same standing question the tab bar stands on, and the age is the
    // caller's own `now_unix` against the snapshot's wall-clock completion
    // stamp — which is the whole reason the stamp stopped being an `Instant`.
    let h = Harness::new();
    let (clock, model) = h.model();
    assert_eq!(
        crate::test_support::chrome::notes(&model).0,
        None,
        "a snapshot the worker just published is not stale"
    );
    // Passes stop completing: the full sweep no longer re-stamps the snapshot,
    // and the frame says so instead of pretending it is current.
    clock.advance(Duration::from_secs(40));
    assert_eq!(
        crate::test_support::chrome::notes(&model).0.as_deref(),
        Some("derivation 40 s behind"),
        "the age of what is on screen, not a claim about what it should be"
    );
}

#[test]
fn a_conversation_whose_descent_grows_is_named_on_the_ops_surface() {
    // The storm signal (bl-ee0a): 227 branches under one conversation used to
    // render as yog being slow. Now the sweep that finds the growth says whose
    // it is, which points the operator at litany in one glance.
    let h = Harness::new();
    let (clock, mut model) = h.model();
    assert_eq!(
        crate::test_support::chrome::notes(&model).1,
        None,
        "a quiet workspace says nothing"
    );
    // Two children dispatched under the fixture's root conversation.
    h.build_more("c-1-kid-1", "sub one");
    h.build_more("c-1-kid-2", "sub two");
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    super::derive::settle(&mut model, &clock);
    assert_eq!(
        crate::test_support::chrome::notes(&model).1.as_deref(),
        Some("hello +2 branches"),
        "named by the §3.3 display name the roster shows"
    );
}
