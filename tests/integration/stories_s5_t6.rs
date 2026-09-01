//! STORIES **S5-T6** login-rows: the provider rows derive from the §5.1
//! #20/#21 config reads, a Login button appears on every row `bz --login` can
//! serve and on **no other** — a row it cannot serve carrying, where the button
//! would be, the reason there is none — and every row states its credential
//! fact **in words** (STORIES S5.1/S5.4, DESIGN §8.3, §9.5, bl-402f). The fact
//! is brazen's own `credential` column since bl-dba3 — yog re-derives none of
//! it, so the fixture states the column rather than laying files on disk.
//!
//! The streamed half of this row — the device lines carried verbatim and the
//! exact-command fallback on a non-zero exit — is shared with S0-T5
//! (`tests/integration/stories_s0_t5.rs`) and is not repeated here.
//!
//! "Nothing is left to a luminance the operator has to interpret": every
//! assertion below is on a *sentence*, because that is the promise.

#![allow(clippy::unwrap_used)]

use yog::config_edit::brazen::{provider_rows, row_names, row_views};

/// brazen's `--list-providers --json`, in routing order — one row per
/// credential model yog must have a sentence for, each with the `credential`
/// column brazen computes through the very `fetch_cred` a run spends. That
/// column IS the credential fact since bl-dba3; yog holds no second answer to
/// it, so a fixture that omitted it would be testing a derivation nothing runs.
const LISTING: &str = r#"{"providers":[
    {"name":"anthropic","auth":"oauth2","credential":"stored"},
    {"name":"openai","auth":"oauth2","credential":"missing"},
    {"name":"borrowed","auth":"oauth2","credential":"ambient"},
    {"name":"deepseek","auth":"api_key","credential":"missing"},
    {"name":"internal","auth":"bearer","credential":"missing"},
    {"name":"local","auth":"none","credential":"not required"},
    {"name":"newthing","auth":"quantum","credential":"missing"}
]}"#;

/// The same table with `deepseek`'s key supplied — a keyed row's secret is a
/// config value, so brazen answers `inline` rather than `stored`, and either way
/// it is a credential the row would spend.
const LISTING_WITH_KEY: &str = r#"{"providers":[
    {"name":"deepseek","auth":"api_key","credential":"inline"}
]}"#;

/// STORIES **S5-T6** login-rows.
#[test]
fn s5_t6_every_provider_row_states_its_credential_fact_and_earns_its_button() {
    // The rows are brazen's listing, in brazen's order — yog re-orders nothing.
    let rows = provider_rows(LISTING);
    assert_eq!(
        row_names(&rows),
        [
            "anthropic",
            "openai",
            "borrowed",
            "deepseek",
            "internal",
            "local",
            "newthing"
        ],
        "listing order IS routing order"
    );

    let views = row_views(&rows);
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
    // A credential brazen resolved from OUTSIDE its own store is still a
    // credential the row would spend, so the row is signed in (bl-dba3: this
    // read *"not signed in"* while a live call through such a row succeeded).
    assert_eq!(fact("borrowed"), "auth oauth2 · signed in");
    // "signed in" is claimed only where a login could have earned it: a keyed
    // row's secret is a *credential stored*, which is a different sentence,
    // because it was a different act.
    let with_key = row_views(&provider_rows(LISTING_WITH_KEY));
    let keyed = with_key.iter().find(|v| v.name == "deepseek").unwrap();
    assert_eq!(keyed.fact, "auth api_key · credential stored");
    assert!(
        !keyed.fact.contains("signed in"),
        "only a login earns 'signed in'"
    );

    // --- The button appears on the oauth2 rows and on NO other …
    assert_eq!(blocked("anthropic"), None, "oauth2 rows get the verb");
    assert_eq!(blocked("openai"), None);
    // …including one already answering off a borrowed credential: the verb is
    // read off `auth`, and signing in replaces what was borrowed.
    assert_eq!(blocked("borrowed"), None);
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
