//! The provider-table projection (§5.1 #20/#21, §8.3): listing order, the three
//! consumed columns, login read off `auth` (§8.3), the row view every surface
//! renders (name · credential fact · offer), and the shapeless payload folding
//! to no rows.
//!
//! The reads over the *other* column — what `protocol` says about a yog turn
//! (§9.4) — are [`capability`], beside the production module that owns them:
//! §12 gives `providers/capability.rs` its own row for that seam, and the
//! corpus follows it. Both suites share [`LIVE_LISTING`], because a second
//! copy of the operator's real table is a second table.

mod capability;

use super::{ProviderRow, provider_rows, row_views};

/// The operator's real table, verbatim from `bz --list-providers --json` on the
/// box that filed bl-b4e5 — the fixture the acceptance asserts against, so the
/// tests never read the ambient config to know it.
pub(super) const LIVE_LISTING: &str = r#"{"providers":[
    {"auth":"oauth2","credential":"stored","name":"codex","protocol":"openai_responses"},
    {"auth":"none","credential":"not required","name":"local","protocol":"ollama_chat"},
    {"auth":"none","credential":"not required","name":"claude-code","protocol":"claude_code"},
    {"auth":"oauth2","credential":"ambient","name":"claude-session-direct","protocol":"anthropic_messages"},
    {"auth":"api_key","credential":"missing","name":"anthropic","protocol":"anthropic_messages"}
]}"#;

#[test]
fn provider_rows_reads_name_and_auth_in_listing_order() {
    let rows = provider_rows(LIVE_LISTING);
    assert_eq!(
        rows.iter().map(|r| r.name.clone()).collect::<Vec<_>>(),
        [
            "codex",
            "local",
            "claude-code",
            "claude-session-direct",
            "anthropic"
        ]
    );
    assert_eq!(rows[0].auth, "oauth2");
    assert_eq!(rows[1].auth, "none");
}

#[test]
fn login_is_blocked_for_every_row_without_an_oauth_block() {
    let rows = provider_rows(LIVE_LISTING);
    // The two oauth2 rows are the loginable ones (bl-b4e5 defect 2).
    assert_eq!(rows[0].login_blocked(), None, "codex is oauth2");
    assert_eq!(
        rows[3].login_blocked(),
        None,
        "claude-session-direct is oauth2"
    );
    // bl-402f: the reason is the operator's next move, per credential model —
    // `local` / `claude-code` are keyless (nothing to log in), `anthropic` is
    // api-keyed (the key is a config setting), and neither gets a verb.
    assert_eq!(
        rows[1].login_blocked().as_deref(),
        Some("keyless — nothing to log in")
    );
    assert_eq!(
        rows[2].login_blocked().as_deref(),
        Some("keyless — nothing to log in")
    );
    assert_eq!(
        rows[4].login_blocked().as_deref(),
        Some("api-key provider — set the key in Config")
    );
}

#[test]
fn a_bearer_row_is_told_where_its_token_lives() {
    let rows = provider_rows(r#"{"providers":[{"name":"vertex","auth":"bearer"}]}"#);
    assert_eq!(
        rows[0].login_blocked().as_deref(),
        Some("bearer-token provider — set the token in Config")
    );
}

#[test]
fn an_unrecognized_auth_spelling_names_itself_rather_than_guessing() {
    // A spelling yog does not know (a newer brazen, or a degraded column) must
    // not be miscast as api-keyed: the reason quotes what the table said.
    let rows = provider_rows(r#"{"providers":[{"name":"x","auth":"mtls"},{"name":"y"}]}"#);
    let why = rows[0].login_blocked().expect("not oauth2");
    assert!(why.contains("mtls"), "reason quotes the spelling: {why}");
    assert!(why.contains("oauth2"), "and names what bz can serve: {why}");
    assert!(
        rows[1].login_blocked().is_some(),
        "an empty column is not oauth2"
    );
}

/// bl-402f: presence renders (STORIES S5 point 4). Every row states its
/// credential fact in words, phrased by the credential model that makes it
/// true — one derivation, painted by the §8.3 Login pane (bl-20cb: and by it
/// alone) and read without repainting by the boundary and §9.4's remedy.
///
/// **The fact is brazen's `credential` column and nothing else** (bl-dba3).
/// `claude-session-direct` in the fixture carries `ambient` — a credential
/// brazen resolved from outside its own store — so the row is *signed in*,
/// which is what a live call through it proves. This assertion used to read
/// `not signed in`, because the words came from a stat of
/// `<credentials-dir>/<name>.json`, a probe blind to every spelling but
/// `stored`.
#[test]
fn row_views_state_the_credential_fact_in_words() {
    let rows = provider_rows(LIVE_LISTING);
    let views = row_views(&rows);
    assert_eq!(
        views.iter().map(|v| v.fact.clone()).collect::<Vec<_>>(),
        [
            "auth oauth2 \u{b7} signed in",
            // Keyless rows say so; `not required` is not a credential.
            "auth none \u{b7} no credential needed",
            "auth none \u{b7} no credential needed",
            // `ambient` is a credential a run would spend.
            "auth oauth2 \u{b7} signed in",
            "auth api_key \u{b7} no credential stored",
        ]
    );
    assert_eq!(views[0].name, "codex");
    // Only the oauth2 rows carry the Login verb; the rest carry the reason.
    assert_eq!(
        views
            .iter()
            .map(|v| v.blocked.is_none())
            .collect::<Vec<_>>(),
        [true, false, false, true, false]
    );
    assert_eq!(views[4].blocked, rows[4].login_blocked());
}

/// The predicate itself, over every spelling the column has — the four this
/// build knows and one it does not (bl-dba3: an unread spelling is a credential,
/// because `missing` is the only refusal).
#[test]
fn credentialed_is_every_spelling_but_missing_and_not_required() {
    for (credential, want) in [
        ("stored", true),
        ("ambient", true),
        ("inline", true),
        ("keychain-from-a-later-brazen", true),
        ("not required", false),
        ("missing", false),
    ] {
        // The spelling is quoted into its own binding rather than inlined: a
        // `"credential":"<eight or more characters>"` literal is what
        // `leak-rules.sh`'s `credential-assignment` rule exists to catch, and a
        // gate that reads a fixture the way it reads a secret is the gate
        // working.
        let quoted = format!("\"{credential}\"");
        let listing =
            format!(r#"{{"providers":[{{"name":"r","auth":"oauth2","credential":{quoted}}}]}}"#);
        assert_eq!(
            provider_rows(&listing)[0].credentialed(),
            want,
            "{credential}"
        );
    }
}

#[test]
fn a_keyed_row_with_a_credential_reports_it_stored_not_signed_in() {
    // "signed in" is a sentence only a login-capable row can make true; a keyed
    // row's secret is stored, and nothing more is claimed about it.
    let rows = provider_rows(
        r#"{"providers":[{"name":"anthropic","auth":"api_key","credential":"stored"}]}"#,
    );
    assert_eq!(
        row_views(&rows)[0].fact,
        "auth api_key \u{b7} credential stored"
    );
}

#[test]
fn an_unstated_credential_column_is_not_a_refusal() {
    // An absent column folds to the empty string, which is neither `missing`
    // nor `not required` — and [`MISSING`] is documented as the ONE spelling
    // that is a refusal, so a question that went unanswered may not read as a
    // row that cannot answer. The §8.1 start gate reads the column by the same
    // rule, which is the point: one predicate, so the pane and the gate cannot
    // disagree about whether a start can run.
    let rows = provider_rows(r#"{"providers":[{"name":"codex","auth":"oauth2"}]}"#);
    assert_eq!(row_views(&rows)[0].fact, "auth oauth2 \u{b7} signed in");
    assert_eq!(
        row_views(&[]).len(),
        0,
        "the table names the rows, and an empty table names none"
    );
}

#[test]
fn a_shapeless_listing_folds_to_no_rows() {
    // A 78 exit writes its message to stderr and nothing to stdout.
    assert_eq!(provider_rows(""), Vec::new());
    assert_eq!(provider_rows("MalformedFile at line 3"), Vec::new());
    assert_eq!(provider_rows("{}"), Vec::new());
    assert_eq!(provider_rows(r#"{"providers":7}"#), Vec::new());
}

#[test]
fn a_row_missing_a_column_degrades_to_empty_not_loginable() {
    let rows = provider_rows(r#"{"providers":[{"nome":"x"},{"name":7,"auth":"oauth2"}]}"#);
    assert_eq!(
        rows,
        [
            ProviderRow {
                name: String::new(),
                protocol: String::new(),
                auth: String::new(),
                credential: String::new(),
                effort: false,
                priority: false,
            },
            ProviderRow {
                name: String::new(),
                protocol: String::new(),
                auth: "oauth2".to_owned(),
                credential: String::new(),
                effort: false,
                priority: false,
            },
        ]
    );
    assert!(rows[0].login_blocked().is_some());
}

/// The name-only projection every row judgement consumes (§9.4, §9.5): the same
/// rows, the same order, one column.
#[test]
fn row_names_projects_the_name_column_in_listing_order() {
    let rows = provider_rows(
        r#"{"providers":[{"name":"codex","auth":"oauth2"},{"name":"claude-session-direct","auth":"oauth2"}]}"#,
    );
    assert_eq!(
        super::row_names(&rows),
        vec!["codex".to_string(), "claude-session-direct".to_string()]
    );
    assert!(super::row_names(&[]).is_empty());
}
