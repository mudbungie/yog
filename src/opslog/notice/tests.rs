//! The notice classifier in both directions (bl-1296): every enumerated lernie
//! notice is recognized, and every near-miss — a real death, an unprefixed
//! line, a tail that mixes the two — is not.
//!
//! The samples are the lines lernie's pinned release actually prints on the
//! detached driver's stderr, operands stubbed. A classifier battery over words
//! no substrate emits proves nothing, which is the rule the sibling remedy
//! beats already keep.

use super::{NOTICE_MARKERS, is_notice, looks_notice};

/// One line per benign class the driver reports (lernie 0.0.11: the two
/// compaction-landing outcomes, the retarget decline, the crashed-tool-window
/// settlement, the four accepted-crash-class launch notes, the §6 budget stop).
const SAMPLES: &[&str] = &[
    "lernie: compaction landing [c-2] declined — git could not replay a.md \
     (marked refs/lernie/conflicted/c-2, ARCH §2.6); the branch continues uncompacted",
    "lernie: compaction landing [c-2] superseded — a compaction landed since its \
     fork point (ARCH §2.6); the branch continues",
    "lernie: retarget of [c-1] declined — git could not replay a.md (marked \
     refs/lernie/conflicted/c-1, ARCH §2.6); the branch continues on its previous config",
    "lernie: settling a crashed tool window on [c-1] — 2 unanswered invocation(s) \
     recorded as died (ARCH §6, bl-4187)",
    "lernie: exit launch for c-1: no such file (accepted crash class, ARCH §2.11)",
    "lernie: revival launch for c-1: no such file (accepted crash class, ARCH §2.11)",
    "lernie: post-release inbox re-read for c-1: no such file (accepted crash class, ARCH §2.11)",
    "lernie: post-release launch for c-1: no such file (accepted crash class, ARCH §2.11)",
    "lernie: budget exhausted on c-1; stopping (ARCH §6)",
];

/// The class this exists to stop reddening: every enumerated line, alone on the
/// sink, reads as a notice.
#[test]
fn every_enumerated_lernie_notice_is_recognized() {
    for line in SAMPLES {
        assert!(looks_notice(line), "not recognized: {line}");
        assert!(
            looks_notice(&format!("{line}\n")),
            "trailing newline: {line}"
        );
    }
}

/// No marker may hide behind the others (the `leak-scan` fixture discipline): a
/// dead alternative in the table is a class that silently went back to red, so
/// each one has to be the reason some sample passed.
#[test]
fn every_marker_in_the_table_is_exercised_by_a_sample() {
    for marker in NOTICE_MARKERS {
        assert!(
            SAMPLES
                .iter()
                .any(|line| line.to_ascii_lowercase().contains(marker)),
            "no sample exercises the marker {marker:?}"
        );
    }
}

/// A whole tail of notices is a tail of notices — the driver chattering benignly
/// for hours is the ordinary shape, not the exception.
#[test]
fn a_tail_of_several_notices_is_still_a_notice() {
    let tail = format!("{}\n\n{}\n", SAMPLES[1], SAMPLES[4]);
    assert!(looks_notice(&tail));
}

/// **Fail toward alarming.** A notice does not vouch for the line beside it: a
/// driver that files a notice and then dies has died, in either order.
#[test]
fn a_tail_mixing_a_notice_with_anything_else_stays_a_failure() {
    let death = "lernie: brazen 0.0.2 != 0.0.3";
    assert!(!looks_notice(&format!("{}\n{death}\n", SAMPLES[1])));
    assert!(!looks_notice(&format!("{death}\n{}\n", SAMPLES[1])));
}

/// The near-misses, one per way a line can look benign and not be: a real
/// decline that reaches the sink under a *verb's* prefix rather than the
/// driver's, a driver line with the prefix and no marker, a marker phrase with
/// no prefix, and text from nowhere in particular.
#[test]
fn near_misses_are_not_notices() {
    for line in [
        "lernie prompt: provider error (Config) on provider row \"x\": unknown provider `x`",
        "lernie: brazen 0.0.2 != 0.0.3",
        "compaction landing [c-2] superseded (ARCH §2.6); the branch continues",
        "refusing: version skew",
        "lernie: setpgid: operation not permitted",
    ] {
        assert!(!looks_notice(line), "wrongly benign: {line}");
    }
}

/// An empty or blank tail is the *silent* launch, which is
/// `OpRow::detached`'s fact and not this one — answering `true` here would make
/// every clean handoff a notice.
#[test]
fn a_tail_with_no_lines_at_all_is_not_a_notice() {
    assert!(!looks_notice(""));
    assert!(!looks_notice("\n  \n\t\n"));
}

/// The match is case-insensitive and tolerates the leading whitespace a clipped
/// or wrapped sink can carry — the same shape as the two sibling classifiers.
#[test]
fn the_match_is_case_insensitive_and_ignores_leading_space() {
    assert!(is_notice(
        "   LERNIE: BUDGET exhausted on c-1; STOPPING (ARCH §6)"
    ));
    assert!(!is_notice("   LERNIE: something else entirely"));
}
