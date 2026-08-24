//! Ops-tail / surface-failure view-model tests (§4.2, §7.3), against the shared
//! cloned-project world in [`super`]. The §8.1/§13.3 detached driver — the row
//! whose verdict arrives from a sink file rather than from an exit code — is
//! [`detached`], split off at the cap.

mod detached;

use super::{append_op, model, world};
use crate::AppModel;
use crate::opslog::{self, Origin};

/// Append one `lernie prime` line for `cwd` with the given exit and origin.
fn prime(m: &AppModel, cwd: &str, exit: i32, origin: Origin) {
    opslog::append(
        m.state_root(),
        &opslog::OpEntry {
            ts: "TS".into(),
            argv: vec!["lernie".into(), "prime".into()],
            cwd: cwd.into(),
            exit,
            stdout: String::new(),
            stderr: if exit == 0 {
                String::new()
            } else {
                "unrecognized subcommand\n".into()
            },
            origin,
        },
    )
    .unwrap();
}

#[test]
fn last_failure_projects_only_a_failed_last_op() {
    let w = world();
    let (_c, mut m) = model(&w);
    // No ops → no banner; a clean last op → still no banner (§7.3).
    assert!(m.last_failure(Origin::Balls).is_none());
    append_op(&m);
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    assert!(m.last_failure(Origin::Balls).is_none());
    // A failed last op → the surface failure view-model projects argv + tail.
    prime(&m, "/proj", 2, Origin::Balls);
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    let f = m.last_failure(Origin::Balls).unwrap();
    assert_eq!(f.argv, "lernie prime");
    assert_eq!(f.stderr_tail, "unrecognized subcommand");
}

/// bl-48f8, the whole of it: a failure renders on the surface that originated
/// it and on **no other**. Before the attribution the query was global — one
/// failed start painted the balls fold, the composer and the bootstrap box at
/// once, and DESIGN §7.3's "the originating surface renders the failure" was a
/// promise the code could not keep.
#[test]
fn a_failure_banners_on_its_own_origin_and_on_no_other() {
    let w = world();
    let (_c, mut m) = model(&w);
    // A ball-rung start's substrate step dies: the roster's balls section owns it.
    prime(&m, "/proj", 2, Origin::Balls);
    m.after_lernie_verb();
    m.tick();
    assert_eq!(
        m.last_failure(Origin::Balls).unwrap().argv,
        "lernie prime",
        "the balls fold, where the ▶ Start row that fired it is (§11, bl-6ad8)"
    );
    assert!(
        m.last_failure(Origin::Conversation).is_none(),
        "the composer (and the bootstrap box) says nothing about someone else's start"
    );
    assert!(
        m.last_failure(Origin::World).is_none(),
        "and neither does the config/login/delete class"
    );
}

/// The mirror, per origin class — each one banners on itself alone. The
/// composer's own Enter must not accuse the balls fold, and a config write must
/// accuse neither: the §9 pane, the §16.3 knob, the §8.3 login pane and the §3.6
/// dialog each state their outcome in place, so a banner elsewhere is the same
/// error twice on a surface that did nothing.
#[test]
fn each_origin_class_is_rendered_by_its_own_surface_only() {
    let w = world();
    let (_c, mut m) = model(&w);

    prime(&m, "/ws", 2, Origin::Conversation);
    m.after_lernie_verb();
    m.tick();
    assert!(m.last_failure(Origin::Conversation).is_some());
    assert!(m.last_failure(Origin::Balls).is_none());
    assert!(m.last_failure(Origin::World).is_none());

    prime(&m, "/ws", 3, Origin::World);
    m.after_lernie_verb();
    m.tick();
    assert!(m.last_failure(Origin::World).is_some());
    assert!(m.last_failure(Origin::Balls).is_none());
    assert!(
        m.last_failure(Origin::Conversation).is_some(),
        "and a later World failure does not retire the composer's live one"
    );
}

/// §6's retirement rule is per-surface too (bl-48f8): a surface's banner clears
/// when **that surface's** next action runs clean, and a clean run elsewhere
/// leaves it standing. Globally-scoped, one fold's successful `bl close` wiped a
/// live start failure off the composer — the operator's error simply vanished.
#[test]
fn a_clean_run_clears_only_its_own_surfaces_banner() {
    let w = world();
    let (_c, mut m) = model(&w);
    prime(&m, "/ws", 2, Origin::Conversation);
    prime(&m, "/proj", 0, Origin::Balls); // an unrelated surface succeeds
    m.after_lernie_verb();
    m.tick();
    assert!(
        m.last_failure(Origin::Conversation).is_some(),
        "the composer's failure is still the composer's last word"
    );
    assert!(m.last_failure(Origin::Balls).is_none());
    // Now the composer itself re-runs clean: its banner retires.
    prime(&m, "/ws", 0, Origin::Conversation);
    m.after_lernie_verb();
    m.tick();
    assert!(m.last_failure(Origin::Conversation).is_none());
    assert_eq!(m.snap.ops.len(), 3, "the log keeps every line");
}

#[test]
fn a_re_run_green_verb_retires_its_stale_failure_from_the_chip() {
    let w = world();
    let (_c, mut m) = model(&w);
    // The three-day-old wound: `lernie prime` failed, and stayed the ambient
    // error long after it was fixed (§6 retirement).
    prime(&m, "/w", 2, Origin::Balls);
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    assert_eq!(m.activity().errors, 1, "the fresh failure is live");
    // Re-run green: the failure retires from the chip but stays in the log.
    prime(&m, "/w", 0, Origin::Balls);
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    assert_eq!(
        m.activity(),
        opslog::Activity {
            total: 2,
            errors: 0,
            drifts: 0
        }
    );
    assert_eq!(m.snap.ops.len(), 2, "the log keeps both lines");
    // An unrelated later failure is the one the operator sees.
    append_op(&m); // clean `bl close bl-work`
    prime(&m, "/other", 3, Origin::Balls);
    m.after_lernie_verb();
    m.tick(); // the ops re-read is the worker's next pass (§7.2)
    assert_eq!(m.activity().errors, 1);
    assert_eq!(m.last_failure(Origin::Balls).unwrap().argv, "lernie prime");
}
