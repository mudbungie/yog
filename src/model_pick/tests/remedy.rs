//! The way out of a credential-shaped roster failure (bl-91f1): that the
//! picker's fault routes to the surface that fixes *this* row, in the words
//! §8.3's Login rows and §9.5's config rows already use, and that it routes
//! nothing else.

use crate::config_edit::brazen::ProviderRow;
use crate::keymap::CenterTab;
use crate::model_pick::remedy;

/// brazen's own decline, verbatim — `resolved_secret`'s `None` arm at the
/// brazen pin (`Cargo.toml` is the pin authority; no version is restated
/// here), which is the line the operator actually read off
/// the picker. The remedy is gated on classifying *this* string, so a reworded
/// upstream decline that stops looking auth-shaped fails here rather than
/// silently retiring the affordance.
const NO_CREDENTIAL: &str = "no credential for this provider: set BRAZEN_API_KEY (or the provider \
     API-key env var / --api-key) or run `bz --login --provider <id>`";

fn row(name: &str, auth: &str) -> ProviderRow {
    ProviderRow {
        name: name.to_owned(),
        protocol: "anthropic_messages".to_owned(),
        auth: auth.to_owned(),
    }
}

/// The motivating case: the operator's `anthropic` row, keyed, with no key.
/// The remedy is the Config tab — never the sign-in brazen's own sentence
/// offers, which this row could only refuse with exit 78.
#[test]
fn a_keyed_row_with_no_key_routes_to_the_config_editor() {
    let found = remedy(&row("anthropic", "api_key"), NO_CREDENTIAL)
        .expect("brazen's decline names a credential, so it is auth-shaped");
    assert_eq!(found.tab, CenterTab::Config);
    assert_eq!(found.verb, "Config");
    assert_eq!(
        found.reason.as_deref(),
        Some("anthropic: api-key provider — set the key in Config"),
        "the sentence is `login_blocked`'s, under the row's own name — not a \
         second phrasing of what a keyed row needs"
    );
}

/// A bearer row takes the same route with its own sentence: one arm, no
/// per-spelling branch.
#[test]
fn a_bearer_row_takes_the_same_route_with_its_own_sentence() {
    let found = remedy(&row("acme", "bearer"), NO_CREDENTIAL).expect("auth-shaped");
    assert_eq!(found.tab, CenterTab::Config);
    assert_eq!(
        found.reason.as_deref(),
        Some("acme: bearer-token provider — set the token in Config")
    );
}

/// A row that signs in gets the verb instead of a sentence, and the verb names
/// its object — the routing bl-8e34 had to derive from a git join, handed.
#[test]
fn an_oauth_row_routes_to_login_and_the_verb_names_the_row() {
    let found =
        remedy(&row("openai-chatgpt", "oauth2"), "401 unauthorized").expect("a 401 is auth-shaped");
    assert_eq!(found.tab, CenterTab::Login);
    assert_eq!(found.verb, "Login: openai-chatgpt");
    assert_eq!(
        found.reason, None,
        "`login_blocked` has no sentence for an oauth row because the verb is \
         the whole answer"
    );
}

/// The two remaining credential models — keyless, and an `auth` spelling this
/// build does not know — still land somewhere real: brazen's `config.toml` is
/// where a row's own `auth` is authored, so the destination does not branch.
#[test]
fn keyless_and_unknown_rows_still_reach_the_file_that_declares_them() {
    for (auth, reason) in [
        ("none", "local: keyless — nothing to log in"),
        (
            "sorcery",
            "local: auth \"sorcery\" — bz --login signs in oauth2 rows only",
        ),
    ] {
        let found = remedy(&row("local", auth), NO_CREDENTIAL).expect("auth-shaped");
        assert_eq!(found.tab, CenterTab::Config);
        assert_eq!(found.reason.as_deref(), Some(reason));
    }
}

/// The gate. A roster that failed on anything else gets no remedy at all: a
/// button that sent the operator to the Config tab for a dead binary or a
/// provider with nothing to offer would be a guess with a control on it. The
/// error and its run-by-hand command still paint — the caller owns those.
#[test]
fn a_failure_that_is_not_about_credentials_offers_no_route() {
    for other in [
        "failed to spawn `bz`: No such file or directory (os error 2)",
        crate::model_pick::query::EMPTY_ROSTER,
        "connection reset by peer",
        "500 internal server error",
    ] {
        assert_eq!(
            remedy(&row("anthropic", "api_key"), other),
            None,
            "{other} is not a credential problem"
        );
    }
}
