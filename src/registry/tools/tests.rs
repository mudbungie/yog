//! The advertised set: its one spelling, its document, and the two ways a
//! presentation is refused (REMOTE §5).

use super::*;
use tempfile::TempDir;

fn client(name: &str) -> Client {
    Client::parse(name).expect("a usable identity")
}

fn tool(name: &str) -> Tool {
    Tool {
        name: name.to_owned(),
        description: format!("what {name} does"),
        input_schema: json!({"type": "object", "properties": {"path": {"type": "string"}}}),
        subject_cwd: false,
    }
}

/// The document is the client's own, beside its pane (§4.1's layout).
#[test]
fn the_set_lands_beside_the_clients_pane_document() {
    let root = Path::new("/home/u/state/yog");
    let c = client("laptop");
    assert_eq!(path(root, &c), super::super::dir(root, &c).join(TOOLS));
    assert_eq!(path(root, &c).file_name().expect("leaf"), "tools.json");
}

/// Encode → decode is the identity, and the schema comes back **verbatim** —
/// the whole point of carrying it as a value rather than a narrowing.
#[test]
fn a_set_survives_its_one_spelling_with_the_schema_untouched() {
    let set = vec![tool("Bash"), tool("Read")];
    let wire = encode(&set);
    assert_eq!(decode(&wire).expect("decoded"), set);
    assert_eq!(wire[0]["input_schema"], set[0].input_schema);
}

/// Strict, as the gesture codec is: an element that is not an object, a missing
/// field, and a body that is not an array each refuse.
#[test]
fn a_malformed_set_refuses_naming_what_is_wrong() {
    assert!(decode(&json!({"tools": []})).is_err(), "not an array");
    assert!(
        decode(&json!([7])).is_err(),
        "an element that is not an object"
    );
    let missing = decode(&json!([{"name": "Bash", "description": "d"}])).expect_err("refused");
    assert!(missing.contains("input_schema"), "{missing}");
    let unnamed = decode(&json!([{"description": "d", "input_schema": {}}])).expect_err("refused");
    assert!(unnamed.contains("name"), "{unnamed}");
}

/// A name that could address the filesystem, and a name said twice: both are
/// sets that cannot be addressed, and both decline loudly.
#[test]
fn an_unusable_or_repeated_name_is_declined_loudly() {
    let bad = validate(&[tool("../etc")]).expect_err("refused");
    assert!(bad.contains("unusable tool name"), "{bad}");
    let twice = validate(&[tool("Bash"), tool("Bash")]).expect_err("refused");
    assert!(twice.contains("duplicate tool name"), "{twice}");
    validate(&[tool("Bash"), tool("Read")]).expect("a set that can be addressed");
    validate(&[]).expect("an empty set is a set");
}

/// Store, read back, and — REMOTE §5's own words — write only when it differs.
#[test]
fn a_set_is_stored_once_and_rewritten_only_when_it_changes() {
    let tmp = TempDir::new().expect("tmp");
    let c = client("laptop");
    assert!(read(tmp.path(), &c).is_empty(), "advertised nothing yet");
    assert!(store(tmp.path(), &c, &[tool("Bash")]).expect("stored"));
    assert_eq!(read(tmp.path(), &c), vec![tool("Bash")]);
    assert!(
        !store(tmp.path(), &c, &[tool("Bash")]).expect("stored"),
        "an unchanged re-presentation writes nothing"
    );
    assert!(store(tmp.path(), &c, &[tool("Read")]).expect("stored"));
    assert_eq!(read(tmp.path(), &c), vec![tool("Read")]);
    // Advertising nothing is advertising: the set is replaced, not merged.
    assert!(store(tmp.path(), &c, &[]).expect("stored"));
    assert!(read(tmp.path(), &c).is_empty());
}

/// An unreadable or nonsense document reads as the empty set — the same posture
/// a client that has never advertised has, so no reader carries two cases.
#[test]
fn a_document_that_cannot_be_read_is_the_empty_set() {
    let tmp = TempDir::new().expect("tmp");
    let c = client("laptop");
    std::fs::create_dir_all(super::super::dir(tmp.path(), &c)).expect("mkdir");
    std::fs::write(path(tmp.path(), &c), b"{not json").expect("write");
    assert!(read(tmp.path(), &c).is_empty());
    std::fs::write(path(tmp.path(), &c), b"[{\"name\": 7}]").expect("write");
    assert!(read(tmp.path(), &c).is_empty(), "a set it cannot decode");
}

/// A path that cannot be made is a refusal, not a panic — the client
/// directory's own parent is a file here.
#[test]
fn an_unwritable_document_refuses() {
    let tmp = TempDir::new().expect("tmp");
    std::fs::write(tmp.path().join(super::super::CLIENTS), b"not a directory").expect("write");
    assert!(store(tmp.path(), &client("laptop"), &[tool("Bash")]).is_err());
}

/// **The consent fact reads strictly** (bl-77be): true rides and rounds,
/// absence is false, and a mistyped value refuses rather than silently
/// dropping — or inventing — an operator's statement.
#[test]
fn subject_cwd_rides_only_when_true_and_a_mistyped_one_refuses() {
    let consenting = Tool {
        subject_cwd: true,
        ..tool("bash")
    };
    let spelled = one(&consenting);
    assert_eq!(spelled.get("subject_cwd"), Some(&serde_json::json!(true)));
    assert_eq!(of_one(&spelled), Ok(consenting));

    let plain = one(&tool("Bash"));
    assert!(plain.get("subject_cwd").is_none(), "absence is the default");
    assert_eq!(of_one(&plain), Ok(tool("Bash")));

    let mut mistyped = one(&tool("Bash"));
    mistyped["subject_cwd"] = serde_json::json!("yes");
    assert_eq!(
        of_one(&mistyped),
        Err("tool: field \"subject_cwd\" is not a boolean".to_owned())
    );
}
