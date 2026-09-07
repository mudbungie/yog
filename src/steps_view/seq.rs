//! **Which record directory a step sequence names** (ARCH §2.3, bl-136f) — the
//! one join between the number an operator types and the directory litany
//! wrote.
//!
//! Step records live in zero-padded directories (`001`, `002`, …) and the
//! listing prints them that way, so copying from the listing has always
//! worked. Typing the number the listing shows did not: `3` was joined
//! verbatim, named a directory that is not there, and every record then read
//! back [`Absent`](super::Doc::Absent) — the classification that exists to tell
//! an *empty* record from a malformed one, reporting a **wrong path** instead.
//! The answer was `ok:true` with every field absent, which is a false negative
//! on the audit path.
//!
//! Two halves, and neither is a special case. A seq of digits is read as the
//! **number** it is and re-spelled at the record's own width, so `3`, `03` and
//! `003` are one address rather than three. And a seq that names no directory
//! is **refused, naming the ones that do** — never answered as a step whose
//! records are all absent, because the records were never opened.

use std::path::Path;

/// The record directory `seq` names, or a refusal naming the seqs that exist.
///
/// The listing this refusal quotes is [`super::step_seqs`]'s, so what a seat is
/// told exists is exactly what [`super::build`] would have shown it.
pub(super) fn resolve(workspace: &Path, agent_id: &str, seq: &str) -> Result<String, String> {
    let seqs = super::step_seqs(workspace, agent_id);
    let want = padded(seq);
    if seqs.contains(&want) {
        return Ok(want);
    }
    if seqs.is_empty() {
        return Err(format!(
            "unknown step {seq:?}: this conversation has taken no step"
        ));
    }
    Err(format!(
        "unknown step {seq:?}: steps here are {}",
        seqs.join(", ")
    ))
}

/// A numeric `seq` at the records' own width (`3` → `001`-shaped `003`), any
/// other spelling untouched — only the directories themselves say what a record
/// may be called, so a non-numeric needle is carried to the listing verbatim
/// and refused there rather than mangled here.
///
/// Leading zeros come off before the pad, so `0003` and `3` are the same
/// address; a number too long for the width keeps every digit it has, which is
/// the general path rather than a truncation.
fn padded(seq: &str) -> String {
    if seq.is_empty() || !seq.bytes().all(|b| b.is_ascii_digit()) {
        return seq.to_owned();
    }
    let digits = seq.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    format!("{digits:0>width$}", width = super::STEP_SEQ_WIDTH)
}
