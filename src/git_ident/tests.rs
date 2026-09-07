//! The classifier, against git's real words — captured from `git commit` on a
//! box with no identity, verbatim but for the host, which is the whole input
//! this module has.

use super::*;

/// git's capture on an identity-less box, as `litany new` hands it up.
const SAID: &str = "Author identity unknown\n\n\
     *** Please tell me who you are.\n\n\
     Run\n\n  git config --global user.email \"you@example.com\"\n  \
     git config --global user.name \"Your Name\"\n\n\
     to set your account's default identity.\n\
     Omit --global to set the identity only in this repository.\n\n\
     fatal: unable to auto-detect email address (got 'root@box.(none)')\n";

/// **The sighting** (bl-c28c): twelve lines of embedded `\n` became one
/// sentence that names the prerequisite and both commands.
#[test]
fn gits_own_banner_becomes_the_named_prerequisite() {
    let said = prerequisite(SAID).unwrap();
    assert!(
        said.starts_with("git has no identity on this box"),
        "{said}"
    );
    assert!(said.contains("git config --global user.name"), "{said}");
    assert!(said.contains("git config --global user.email"), "{said}");
    assert!(
        !said.contains('\n'),
        "one sentence, never a capture: {said}"
    );
}

/// The banner answers for the whole family, `Committer identity unknown` and
/// the auto-detection-disabled `fatal:` included — one string rather than a
/// list of git's `fatal:` spellings that this file would have to keep in step.
#[test]
fn the_banner_answers_for_every_spelling_of_the_complaint() {
    let committer = SAID
        .replace("Author identity", "Committer identity")
        .replace(
            "unable to auto-detect email address (got 'root@box.(none)')",
            "no email was given and auto-detection is disabled",
        );
    assert_eq!(prerequisite(&committer), Some(PREREQUISITE.to_owned()));
}

/// Every other failure is not this one and rides back as itself: a capture yog
/// cannot read is still the truest thing it has.
#[test]
fn another_failure_is_not_this_one() {
    assert_eq!(prerequisite(""), None);
    assert_eq!(
        prerequisite("litany new: destination is not empty: /w/home\n"),
        None
    );
}
