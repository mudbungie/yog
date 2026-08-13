//! The provider-table projection (§5.1 #20/#21, §8.3): listing order, the two
//! consumed columns, the login-capability read off `auth`, the row view every
//! surface renders (name · credential fact · offer), and the shapeless payload
//! folding to no rows.

use super::{ProviderRow, provider_rows, row_views};

/// The operator's real table, verbatim from `bz --list-providers --json` on the
/// box that filed bl-b4e5 — the fixture the acceptance asserts against, so the
/// tests never read the ambient config to know it.
const LIVE_LISTING: &str = r#"{"providers":[
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
/// true — and the words are one derivation, shared by the Login pane and the
/// §9.5 config rows.
#[test]
fn row_views_state_the_credential_fact_in_words() {
    let rows = provider_rows(LIVE_LISTING);
    let creds = vec![
        ("codex".to_string(), true),
        ("local".to_string(), true),
        ("claude-code".to_string(), false),
        ("claude-session-direct".to_string(), false),
        ("anthropic".to_string(), false),
    ];
    let views = row_views(&rows, &creds);
    assert_eq!(
        views.iter().map(|v| v.fact.clone()).collect::<Vec<_>>(),
        [
            "auth oauth2 · signed in",
            // Keyless rows say so whatever the credentials dir holds.
            "auth none · no credential needed",
            "auth none · no credential needed",
            "auth oauth2 · not signed in",
            "auth api_key · no credential stored",
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

#[test]
fn a_keyed_row_with_a_credential_file_reports_it_stored_not_signed_in() {
    // "signed in" is a sentence only a login can make true; a keyed row's file
    // is a stored credential, and nothing more is claimed about it.
    let rows = provider_rows(r#"{"providers":[{"name":"anthropic","auth":"api_key"}]}"#);
    let views = row_views(&rows, &[("anthropic".to_string(), true)]);
    assert_eq!(views[0].fact, "auth api_key · credential stored");
}

#[test]
fn a_row_the_presence_read_omits_is_credential_less_not_a_hole() {
    let rows = provider_rows(r#"{"providers":[{"name":"codex","auth":"oauth2"}]}"#);
    let views = row_views(&rows, &[]);
    assert_eq!(views[0].fact, "auth oauth2 · not signed in");
    assert_eq!(
        row_views(&[], &[("codex".to_string(), true)]).len(),
        0,
        "the table names the rows; the presence list only answers about them"
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
                auth: String::new(),
            },
            ProviderRow {
                name: String::new(),
                auth: "oauth2".to_owned(),
            },
        ]
    );
    assert!(rows[0].login_blocked().is_some());
}

/// The name-only projection the §9.2 provider gate consumes: the same rows,
/// the same order, one column.
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
