use super::*;
use std::fs;
use tempfile::TempDir;

/// The §3.1 shape/length/reserved rules, over an empty root set (no collision
/// can fire), stated as the table they are.
#[test]
fn validate_enforces_the_shape_the_wordlist_used_to_guarantee() {
    let cases: &[(&str, Result<&str, NameError>)] = &[
        // The everyday names §3.1 names by example.
        ("ops", Ok("ops")),
        ("dev", Ok("dev")),
        ("acme-corp", Ok("acme-corp")),
        ("s3", Ok("s3")),
        // Whitespace is forgiven — and only whitespace (`normalize`).
        ("  ops  ", Ok("ops")),
        // The empty name is the shape rule with no input, not a case of its own.
        ("", Err(NameError::Shape)),
        ("   ", Err(NameError::Shape)),
        // Uppercase, spaces, path separators, dots: all path-unsafe or unlawful.
        ("Ops!", Err(NameError::Shape)),
        ("Ops", Err(NameError::Shape)),
        ("two words", Err(NameError::Shape)),
        ("a/b", Err(NameError::Shape)),
        ("..", Err(NameError::Shape)),
        // Hyphens join words; they never lead, trail, or double.
        ("-ops", Err(NameError::Shape)),
        ("ops-", Err(NameError::Shape)),
        ("a--b", Err(NameError::Shape)),
        // bl's own unstamped-claim fallback.
        ("unknown", Err(NameError::Reserved)),
    ];
    for (typed, want) in cases {
        let got = validate(typed, &[]);
        assert_eq!(got, want.clone().map(str::to_owned), "typed={typed:?}");
    }
    // 32 bytes is the bound, not 33.
    let at_cap = "a".repeat(MAX_BYTES);
    assert_eq!(validate(&at_cap, &[]), Ok(at_cap.clone()));
    assert_eq!(
        validate(&"a".repeat(MAX_BYTES + 1), &[]),
        Err(NameError::TooLong)
    );
    // The bootstrap default is itself a lawful name (§3.1) — the constant and
    // the validation cannot drift apart.
    assert_eq!(validate(DEFAULT_NAME, &[]), Ok(DEFAULT_NAME.to_owned()));
}

/// The collision half (§3.1): an existing leaf under **any** of the three roots
/// refuses the name outright — and occupancy is wider than enumeration.
#[test]
fn validate_refuses_a_leaf_that_exists_under_any_root() {
    let (yog, litany) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let foreign = litany.path().join("workspaces");
    let replay = litany.path().join("replays");
    // A half-created dir (no repo.git) still owns its name; so does a file.
    fs::create_dir_all(yog.path().join("ops")).unwrap();
    fs::create_dir_all(foreign.join("acme")).unwrap();
    fs::create_dir_all(foreign.join("20260101T-aa")).unwrap();
    fs::create_dir_all(&replay).unwrap();
    fs::write(replay.join("notes"), "x").unwrap();
    let roots = [yog.path().to_path_buf(), foreign, replay];
    assert_eq!(
        validate("ops", &roots),
        Err(NameError::Taken("ops".to_owned()))
    );
    assert_eq!(
        validate("acme", &roots).unwrap_err(),
        NameError::Taken("acme".to_owned()),
        "a leaf under litany's own root occupies the name too"
    );
    assert_eq!(
        validate("20260101T-aa", &roots),
        Err(NameError::Shape),
        "shape is asked first: a litany auto-id is not a name a human may type"
    );
    assert_eq!(
        validate("notes", &roots),
        Err(NameError::Taken("notes".to_owned()))
    );
    assert_eq!(validate("dev", &roots), Ok("dev".to_owned()));
    // A missing root contributes nothing (the general path with no inputs).
    assert_eq!(
        validate("ops", &[yog.path().join("gone")]),
        Ok("ops".to_owned())
    );
}

/// Every refusal states its reason to the operator (§11: rendered inline at the
/// form, never an ops wound).
#[test]
fn every_refusal_carries_an_operator_facing_reason() {
    for err in [
        NameError::Shape,
        NameError::TooLong,
        NameError::Reserved,
        NameError::Taken("ops".to_owned()),
    ] {
        assert!(!err.to_string().is_empty());
    }
    assert_eq!(
        NameError::Taken("ops".to_owned()).to_string(),
        "`ops` already exists — pick another name"
    );
    assert_eq!(NameError::TooLong.to_string(), "a name is at most 32 bytes");
    assert!(NameError::Reserved.to_string().starts_with("`unknown`"));
}
