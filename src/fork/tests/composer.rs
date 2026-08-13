//! **S12-T3 cohort-one-path** (the fire half): ×N is a `Vec`'s length, and
//! nothing branches on it.

use super::{attempt, choices};
use crate::fork::Choices;
use crate::fork::composer::Composer;

/// The composer seeds from what the workspace offers — the first point that
/// declares a role, wearing that point's first role — with the goal empty
/// (VISION V2.1) and exactly one attempt.
#[test]
fn a_seeded_composer_is_one_attempt_from_the_first_offered_policy() {
    let seeded = Composer::seeded(&choices());
    assert_eq!(seeded.goal, "");
    assert_eq!(seeded.attempts, vec![attempt("aaaa1111", "worker", &[])]);
    // A seat with nothing on offer seeds an empty attempt, which `ready`
    // refuses — the caller has already declined to paint it.
    let bare = Composer::seeded(&Choices::default());
    assert_eq!(bare.attempts.len(), 1);
    assert!(!bare.ready());
}

/// ×N grows by cloning the last candidate — a variant is nearly always the
/// previous one with a control moved — and shrinks by dropping from the end.
/// The floor is one, so `0` is not a case, it is a value.
#[test]
fn times_n_grows_by_cloning_and_floors_at_one() {
    let mut c = Composer::seeded(&choices());
    c.attempts[0] = attempt("config/strict", "worker", &["bash"]);
    c.resize(3);
    assert_eq!(c.attempts.len(), 3);
    assert!(
        c.attempts
            .iter()
            .all(|a| a.skills == vec!["bash".to_owned()])
    );
    c.attempts[2].role = "scribe".to_owned();
    c.resize(1);
    assert_eq!(
        c.attempts,
        vec![attempt("config/strict", "worker", &["bash"])]
    );
    c.resize(0);
    assert_eq!(
        c.attempts.len(),
        1,
        "a composer with no attempt cannot fire"
    );
}

/// Skills are per attempt: toggling one candidate's skill leaves its siblings
/// alone, and a toggle at an index the composer no longer has is a no-op —
/// a stale click from a frame that has since shrunk edits nothing.
#[test]
fn skills_toggle_per_candidate_and_a_stale_index_edits_nothing() {
    let mut c = Composer::seeded(&choices());
    c.resize(2);
    c.toggle_skill(1, "bash");
    assert!(c.attempts[0].skills.is_empty());
    assert_eq!(c.attempts[1].skills, vec!["bash".to_owned()]);
    c.toggle_skill(1, "bash");
    assert!(c.attempts[1].skills.is_empty(), "the toggle goes both ways");
    c.toggle_skill(9, "bash");
    assert_eq!(c.attempts.len(), 2);
}

/// Readiness is one rule for every seat: text in the goal, and every candidate
/// naming both a fork point and a role. The widget's button and a headless
/// caller refuse on the same sentence.
#[test]
fn ready_wants_a_goal_and_a_whole_policy_per_candidate() {
    let mut c = Composer::seeded(&choices());
    assert!(!c.ready(), "an empty goal fires nothing");
    c.goal = "   \n ".to_owned();
    assert!(!c.ready(), "whitespace is not a goal");
    c.goal = "try it the other way".to_owned();
    assert!(c.ready());
    c.resize(2);
    c.attempts[1].role = String::new();
    assert!(!c.ready(), "a candidate with no role names no model");
    c.attempts[1].role = "worker".to_owned();
    c.attempts[1].from = String::new();
    assert!(!c.ready(), "a fork with no ref is a different gesture");
    c.attempts.clear();
    assert!(!c.ready());
}
