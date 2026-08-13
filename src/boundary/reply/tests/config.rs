//! The §9 config family's replies, encoded: what a write landed and what the
//! §16.3 knob now reads (bl-3f46), and — bl-0164 — a destination's bytes and
//! the provider table with its credential fact.

use super::super::*;
use crate::config_edit::brazen::ProviderRowView;

#[test]
fn the_config_family_says_what_landed_and_what_the_knob_now_reads() {
    let applied = encode(&Reply::Applied {
        file: "/cfg/models.yaml".into(),
    });
    assert_eq!(applied["ok"], true);
    assert_eq!(applied["kind"], "applied");
    assert_eq!(applied["file"], "/cfg/models.yaml");
    // The knob reads back as both halves of the one answer: the branch, and
    // the space it is a branch of.
    let marks = encode(&Reply::Marks {
        branch: "balls/agents/corp".into(),
        space: std::path::PathBuf::from("/d/yog/world/walls/corp/marks"),
    });
    assert_eq!(marks["ok"], true);
    assert_eq!(marks["kind"], "marks");
    assert_eq!(marks["branch"], "balls/agents/corp");
    assert_eq!(marks["space"], "/d/yog/world/walls/corp/marks");
}

#[test]
fn a_config_text_reply_carries_the_bytes_verbatim() {
    let empty = encode(&Reply::Config {
        text: String::new(),
    });
    assert_eq!(empty["ok"], true);
    assert_eq!(empty["kind"], "config");
    assert_eq!(empty["text"], "");
    let text = "models:\n  m-1:\n    provider: acme\n";
    assert_eq!(
        encode(&Reply::Config {
            text: text.to_owned()
        })["text"],
        text
    );
}

#[test]
fn a_providers_reply_names_each_rows_credential_fact_and_login_block() {
    let value = encode(&Reply::Providers(vec![
        ProviderRowView {
            name: "acme".to_owned(),
            fact: "auth oauth2 · signed in".to_owned(),
            blocked: None,
        },
        ProviderRowView {
            name: "zinc".to_owned(),
            fact: "auth none · no credential needed".to_owned(),
            blocked: Some("keyless — nothing to log in".to_owned()),
        },
    ]));
    assert_eq!(value["kind"], "providers");
    let rows = value["rows"].as_array().expect("rows");
    assert_eq!(rows[0]["name"], "acme");
    assert_eq!(rows[0]["fact"], "auth oauth2 · signed in");
    assert!(
        rows[0]["blocked"].is_null(),
        "a servable row blocks nothing"
    );
    assert_eq!(rows[1]["name"], "zinc");
    assert_eq!(rows[1]["blocked"], "keyless — nothing to log in");
}
