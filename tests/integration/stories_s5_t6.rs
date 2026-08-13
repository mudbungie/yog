//! STORIES **S5-T6** login-rows: the provider rows derive from the §5.1
//! #20/#21 config reads, a Login button appears on every row `bz --login` can
//! serve and on **no other** — a row it cannot serve carrying, where the button
//! would be, the reason there is none — and every row states its credential
//! fact **in words** (STORIES S5.1/S5.4, DESIGN §8.3, §9.5, bl-402f).
//!
//! The streamed half of this row — the device lines carried verbatim and the
//! exact-command fallback on a non-zero exit — is shared with S0-T5
//! (`tests/integration/stories_s0_t5.rs`) and is not repeated here.
//!
//! "Nothing is left to a luminance the operator has to interpret": every
//! assertion below is on a *sentence*, because that is the promise.

#![allow(clippy::unwrap_used)]

use tempfile::tempdir;
use yog::config_edit::RealFileIo;
use yog::config_edit::brazen::{credential_presence, provider_rows, row_names, row_views};

/// brazen's `--list-providers --json`, in routing order — one row per
/// credential model yog must have a sentence for.
const LISTING: &str = r#"{"providers":[
    {"name":"anthropic","auth":"oauth2"},
    {"name":"openai","auth":"oauth2"},
    {"name":"deepseek","auth":"api_key"},
    {"name":"internal","auth":"bearer"},
    {"name":"local","auth":"none"},
    {"name":"newthing","auth":"quantum"}
]}"#;

/// STORIES **S5-T6** login-rows.
#[test]
fn s5_t6_every_provider_row_states_its_credential_fact_and_earns_its_button() {
    let home = tempdir().unwrap();
    let creds_dir = home.path().join("credentials");
    std::fs::create_dir_all(&creds_dir).unwrap();
    // Only anthropic has ever been signed in; the file's *existence* is the
    // only credential fact yog ever holds — contents are never read.
    std::fs::write(creds_dir.join("anthropic.json"), "{\"token\":\"secret\"}").unwrap();

    // The rows are brazen's listing, in brazen's order — yog re-orders nothing.
    let rows = provider_rows(LISTING);
    assert_eq!(
        row_names(&rows),
        [
            "anthropic",
            "openai",
            "deepseek",
            "internal",
            "local",
            "newthing"
        ],
        "listing order IS routing order"
    );

    // Presence is one existence read per row (§5.1 #22).
    let creds = credential_presence(&creds_dir, &rows, &RealFileIo);
    assert_eq!(
        creds,
        vec![
            ("anthropic".to_owned(), true),
            ("openai".to_owned(), false),
            ("deepseek".to_owned(), false),
            ("internal".to_owned(), false),
            ("local".to_owned(), false),
            ("newthing".to_owned(), false),
        ]
    );

    let views = row_views(&rows, &creds);
    let fact = |name: &str| {
        views
            .iter()
            .find(|v| v.name == name)
            .unwrap_or_else(|| panic!("row {name}"))
            .fact
            .clone()
    };
    let blocked = |name: &str| {
        views
            .iter()
            .find(|v| v.name == name)
            .unwrap()
            .blocked
            .clone()
    };

    // --- Every row states its fact in WORDS.
    assert_eq!(fact("anthropic"), "auth oauth2 · signed in");
    assert_eq!(fact("openai"), "auth oauth2 · not signed in");
    assert_eq!(fact("local"), "auth none · no credential needed");
    assert_eq!(fact("deepseek"), "auth api_key · no credential stored");
    assert_eq!(fact("internal"), "auth bearer · no credential stored");
    // "signed in" is claimed only where a login could have earned it: a keyed
    // row with a file on disk has a *credential stored*, which is a different
    // sentence, because it was a different act.
    std::fs::write(creds_dir.join("deepseek.json"), "k").unwrap();
    let with_key = row_views(&rows, &credential_presence(&creds_dir, &rows, &RealFileIo));
    let keyed = with_key.iter().find(|v| v.name == "deepseek").unwrap();
    assert_eq!(keyed.fact, "auth api_key · credential stored");
    assert!(
        !keyed.fact.contains("signed in"),
        "only a login earns 'signed in'"
    );

    // --- The button appears on the oauth2 rows and on NO other …
    assert_eq!(blocked("anthropic"), None, "oauth2 rows get the verb");
    assert_eq!(blocked("openai"), None);
    // … and every other row carries, where the button would be, the operator's
    // actual next move — never a dead button that could only exit 78.
    assert_eq!(
        blocked("local").as_deref(),
        Some("keyless — nothing to log in")
    );
    assert_eq!(
        blocked("deepseek").as_deref(),
        Some("api-key provider — set the key in Config")
    );
    assert_eq!(
        blocked("internal").as_deref(),
        Some("bearer-token provider — set the token in Config")
    );
    // An auth spelling this build has never heard of is QUOTED, not guessed at:
    // an unknown model is not oauth2, so it gets no button and says why.
    assert_eq!(
        blocked("newthing").as_deref(),
        Some("auth \"quantum\" — bz --login signs in oauth2 rows only")
    );

    // There is no verdict column and no remediation string anywhere in a row —
    // the phase-1 half died with the gate (§16.7 W13).
    assert!(
        views.iter().all(|v| !v.fact.contains("install")),
        "no remediation column survives"
    );

    // A listing yog cannot read is no rows, never an error and never a guess.
    assert!(provider_rows("not json at all").is_empty());
    assert!(provider_rows(r#"{"providers":"nope"}"#).is_empty());
}
