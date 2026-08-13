use super::*;
use std::fs;
use tempfile::TempDir;

/// An [`Rng`] that yields one scripted draw forever — the mint takes exactly
/// one, so a fixed value pins the scan's start index.
struct Fixed(u64);

impl Rng for Fixed {
    fn next_u64(&mut self) -> u64 {
        self.0
    }
}

fn set(names: &[&str]) -> HashSet<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}

/// Three words ⇒ a three-name pool, indices 0..3 in scan order: `ash bay cove`.
const ABC: &[&str] = &["ash", "bay", "cove"];

/// A wordlist, the RNG draw, the occupied names, and the expected mint.
type Case = (
    &'static [&'static str],
    u64,
    &'static [&'static str],
    Result<&'static str, MintError>,
);

#[test]
fn mint_scans_from_the_draw_and_retries_past_collisions() {
    let cases: &[Case] = &[
        // Draw picks the start index; nothing occupied ⇒ that word is the name.
        (ABC, 0, &[], Ok("ash")),
        (ABC, 2, &[], Ok("cove")),
        // A draw wider than the pool wraps into it (5 % 3 == 2).
        (ABC, 5, &[], Ok("cove")),
        // Collision retry: the start is taken, discarded, the scan re-samples
        // the next word.
        (ABC, 0, &["ash"], Ok("bay")),
        // Retry wraps past the end of the pool: "cove" then "ash" taken.
        (ABC, 2, &["cove", "ash"], Ok("bay")),
        // Pool exhaustion: every word of a two-word list is occupied — the
        // retry is bounded by the pool, so this errors instead of looping.
        (
            &["ash", "bay"],
            0,
            &["ash", "bay"],
            Err(MintError::Exhausted(2)),
        ),
        // The empty list is the general path with no inputs — an empty pool.
        (&[], 0, &[], Err(MintError::Exhausted(0))),
    ];
    for (words, draw, taken, expect) in cases {
        let got = mint_from(words, &mut Fixed(*draw), &set(taken));
        let want = expect.clone().map(str::to_owned);
        assert_eq!(got, want, "words={words:?} draw={draw} taken={taken:?}");
    }
}

#[test]
fn exhaustion_error_names_the_pool_size() {
    let err = MintError::Exhausted(2);
    assert_eq!(
        err.to_string(),
        "name pool exhausted: all 2 words are occupied"
    );
}

/// The three properties `words.txt` promises its consumer (bl-ccf7, restated as
/// constraints 1–3 on bl-9769). Cheap, and it pins the artifact against a
/// careless future edit — the mint's path-safety and its "never mistakable for
/// a human identity" guarantee rest entirely on the data.
#[test]
fn embedded_wordlist_holds_its_invariants() {
    let words = wordlist();
    // (1) Parse rule: the `#` header and blank lines are gone, nothing else is.
    assert!(!words.iter().any(|w| w.starts_with('#') || w.is_empty()));
    // (2) Charset `^[a-z]{3,9}$` — what makes a minted word path-safe as a
    // dir leaf and shell-safe unquoted as a `bl claim --as` value.
    for w in &words {
        assert!(
            (3..=9).contains(&w.len()) && w.chars().all(|c| c.is_ascii_lowercase()),
            "{w:?} violates ^[a-z]{{3,9}}$"
        );
    }
    let unique: HashSet<&&str> = words.iter().collect();
    assert_eq!(unique.len(), words.len(), "duplicate word in words.txt");
    // (3) No human-identity collision: bl's own `--as` fallback literal is the
    // one word that must never be mintable (bl validates nothing itself).
    assert!(!unique.contains(&"unknown"));
    // (4) The curated count — the whole one-word pool, and the bound on the
    // collision-retry scan. An edit to the list is meant to update this line —
    // that is the canary, not a nuisance.
    assert_eq!(words.len(), 7395);
}

#[test]
fn mint_over_the_embedded_list_avoids_the_occupied_set() {
    // Deterministic per seed: the same generator state mints the same name.
    let name = mint(&mut SplitMix64::from_seed(7), &HashSet::new()).unwrap();
    let again = mint(&mut SplitMix64::from_seed(7), &HashSet::new()).unwrap();
    assert_eq!(name, again);
    // The one-word shape (bl-d12f): a single wordlist entry, never a compound.
    assert!(!name.contains('-'));
    assert!(wordlist().contains(&name.as_str()));
    // Occupying that name pushes the mint to a different one.
    let next = mint(&mut SplitMix64::from_seed(7), &set(&[&name])).unwrap();
    assert_ne!(next, name);
}

#[test]
fn splitmix64_advances_and_seeds_from_entropy() {
    let mut rng = SplitMix64::from_seed(0);
    let draws: HashSet<u64> = (0..64).map(|_| rng.next_u64()).collect();
    assert_eq!(draws.len(), 64, "generator repeated within 64 draws");
    // The entropy path is real seeding, not a constant.
    assert_ne!(
        SplitMix64::from_entropy().next_u64(),
        SplitMix64::from_seed(0).next_u64()
    );
}

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
    let (yog, lernie) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let foreign = lernie.path().join("workspaces");
    let replay = lernie.path().join("replays");
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
        "a leaf under lernie's own root occupies the name too"
    );
    assert_eq!(
        validate("20260101T-aa", &roots),
        Err(NameError::Shape),
        "shape is asked first: a lernie auto-id is not a name a human may type"
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
