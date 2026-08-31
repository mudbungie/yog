//! The badge vocabulary's own tests: both mappings are total, and no two arms
//! of one mapping say the same words — which is the property that makes the
//! phrase, not the glyph, the fact's carrier.

use super::{op_badge, tool_result_badge};
use crate::opslog::OpOutcome;

/// Every outcome has words, and no two outcomes share them. The two failure
/// arms deliberately share the glyph, so the phrase is what tells them apart.
#[test]
fn every_op_outcome_says_something_of_its_own() {
    let all = [
        OpOutcome::Failed,
        OpOutcome::Retired,
        OpOutcome::Clean,
        OpOutcome::Detached,
    ];
    let mut said: Vec<&str> = all.iter().map(|o| op_badge(*o).1).collect();
    assert!(
        said.iter().all(|w| !w.is_empty()),
        "wordless badge: {said:?}"
    );
    said.sort_unstable();
    let before = said.len();
    said.dedup();
    assert_eq!(said.len(), before, "two outcomes say the same words");
}

/// The retired failure keeps the live failure's glyph — the ⚠ is the fact, the
/// prominence is what retires — so only the words separate them.
#[test]
fn a_retired_failure_keeps_the_glyph_and_changes_the_words() {
    let (live_glyph, live_words) = op_badge(OpOutcome::Failed);
    let (retired_glyph, retired_words) = op_badge(OpOutcome::Retired);
    assert_eq!(live_glyph, retired_glyph);
    assert_ne!(live_words, retired_words);
    assert!(retired_words.contains("retired"), "{retired_words}");
}

/// Both arms of the tool-result flag differ in both carriers.
#[test]
fn a_tool_result_differs_in_glyph_and_words() {
    let (ok_glyph, ok_words) = tool_result_badge(false);
    let (err_glyph, err_words) = tool_result_badge(true);
    assert_ne!(ok_glyph, err_glyph);
    assert_ne!(ok_words, err_words);
    assert!(ok_words.ends_with("ok"), "{ok_words}");
    assert!(err_words.ends_with("error"), "{err_words}");
}
