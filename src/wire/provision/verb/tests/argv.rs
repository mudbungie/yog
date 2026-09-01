//! The argv `yog wire-certs` does not take (bl-a0dd).
//!
//! Every setting is an environment reading, so a word after the verb is a
//! setting the shell put in the wrong place. It used to vanish: the mint aimed
//! at the default loopback endpoint, exit was `0`, and the operator was told it
//! had worked — a wrong trust root, correctable only by a rotation that
//! distrusts every certificate already issued.

use crate::wire::provision::verb::{READS, SUBCMD, stray};

/// **The empty tail is the whole of this verb's argv**, so nothing is refused
/// on a bare invocation — the shape every caller in the tree spends.
#[test]
fn a_bare_invocation_has_no_stray_word() {
    assert_eq!(stray(&[]), None);
}

/// A setting spelled AFTER the verb refuses, names the word it could not read,
/// and hands back the prefix spelling (bl-a0dd). It is not a matter of taste:
/// argv reached no reading at all, so `yog wire-certs WIRE_HOST=… WIRE_PORT=…`
/// used to mint the DEFAULT loopback endpoint and exit 0 — a wrong trust root
/// reported as a success, correctable only by a rotation.
#[test]
fn a_setting_spelled_after_the_verb_refuses_and_says_where_it_goes() {
    let refusal = stray(&[
        "WIRE_HOST=engine.example.com".to_owned(),
        "WIRE_PORT=7737".to_owned(),
    ])
    .expect("a word past the verb refuses");
    assert!(
        refusal.contains("WIRE_HOST=engine.example.com"),
        "{refusal}"
    );
    // The remedy is the prefix, and every reading is named so the operator can
    // see which words this verb actually takes.
    assert!(
        refusal.contains(&format!("yog {SUBCMD}")) && refusal.contains("BEFORE the verb"),
        "{refusal}"
    );
    for key in READS {
        assert!(refusal.contains(key), "{key} unnamed in {refusal}");
    }
}

/// Any word refuses, not just a `KEY=value` one: the verb reads no argv, so a
/// bare word is as unreadable as a misplaced setting and the first one is the
/// one reported.
#[test]
fn the_first_stray_word_is_the_one_named() {
    let refusal = stray(&["rotate".to_owned(), "now".to_owned()]).expect("refused");
    assert!(refusal.contains("\"rotate\""), "{refusal}");
    assert!(!refusal.contains("\"now\""), "{refusal}");
}
