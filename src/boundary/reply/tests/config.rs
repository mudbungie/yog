//! The §9 config family's replies, encoded: what a write landed and what the
//! §16.3 knob now reads (bl-3f46), and — bl-0164 — a destination's bytes and
//! the provider table with its credential fact, and — bl-dff8 — the §9.3
//! lineage browse and the §9.4 model roster.

use super::super::*;
use crate::boundary::reply::ConfigView;
use crate::config_edit::branch::{ConfigBranch, Lineage};
use crate::config_edit::brazen::ProviderRowView;

#[test]
fn the_config_family_says_what_landed_and_what_the_knob_now_reads() {
    // The receipt says that it landed and nothing else (REMOTE §8, bl-ccf7):
    // a destination determines its own location, so a path here would be the
    // address the gesture just gave, respelled as the engine's home root.
    let applied = encode(&Reply::Applied);
    assert_eq!(applied["ok"], true);
    assert_eq!(applied["kind"], "applied");
    assert_eq!(applied.get("file"), None);
    // The knob reads back as the branch alone — the space it is a branch of is
    // a pure function of the workspace the gesture named (bl-ccf7).
    let marks = encode(&Reply::Marks {
        branch: "balls/agents/corp".into(),
    });
    assert_eq!(marks["ok"], true);
    assert_eq!(marks["kind"], "marks");
    assert_eq!(marks["branch"], "balls/agents/corp");
    assert_eq!(marks.get("space"), None);
}

#[test]
fn a_config_text_reply_carries_the_bytes_verbatim() {
    let empty = encode(&Reply::Config(ConfigView {
        text: String::new(),
        settings: Vec::new(),
    }));
    assert_eq!(empty["ok"], true);
    assert_eq!(empty["kind"], "config");
    assert_eq!(empty["text"], "");
    let text = "models:\n  m-1:\n    provider: acme\n";
    assert_eq!(
        encode(&Reply::Config(ConfigView {
            text: text.to_owned(),
            settings: Vec::new(),
        }))["text"],
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
            effort: true,
            priority: true,
        },
        ProviderRowView {
            name: "zinc".to_owned(),
            fact: "auth none · no credential needed".to_owned(),
            blocked: Some("keyless — nothing to log in".to_owned()),
            effort: true,
            priority: false,
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

/// The browse row (bl-dff8): the lineage's bare name — the word `/config
/// branch <lineage> …` takes — its tip both short and full, and the paths that
/// tip holds, which are the paths a read may ask for.
#[test]
fn a_lineages_reply_carries_each_tip_and_the_files_it_holds() {
    let value = encode(&Reply::Lineages(vec![Lineage {
        branch: ConfigBranch {
            name: "default".to_owned(),
            tip_oid: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            tip_short_oid: "01234567".to_owned(),
            tip_timestamp_unix: 1_700_000_000,
        },
        files: vec!["providers.yaml".to_owned(), "version".to_owned()],
    }]));
    assert_eq!(value["kind"], "lineages");
    let rows = value["rows"].as_array().expect("rows");
    assert_eq!(rows[0]["name"], "default");
    assert_eq!(rows[0]["oid"], "0123456789abcdef0123456789abcdef01234567");
    assert_eq!(rows[0]["short_oid"], "01234567");
    assert_eq!(rows[0]["committed"], 1_700_000_000);
    assert_eq!(rows[0]["files"][0], "providers.yaml");
}

/// The roster's rows are bare ids, in the provider's own order — a model has
/// no other fact yog knows (§9.4).
#[test]
fn a_models_reply_is_the_ids_in_the_order_the_provider_listed_them() {
    let value = encode(&Reply::Models(vec!["m-9".to_owned(), "m-1".to_owned()]));
    assert_eq!(value["ok"], true);
    assert_eq!(value["kind"], "models");
    assert_eq!(value["rows"][0], "m-9");
    assert_eq!(value["rows"][1], "m-1");
}
