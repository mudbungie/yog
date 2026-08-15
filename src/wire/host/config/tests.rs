//! The tool host's document: the one reading that produces two, and every way
//! a document refuses to be one (REMOTE §5.2).

use super::*;
use serde_json::json;
use tempfile::tempdir;

fn write(dir: &Path, doc: &Value) -> PathBuf {
    let file = dir.join(TOOLS);
    std::fs::write(&file, doc.to_string()).expect("config");
    file
}

fn document() -> Value {
    json!([
        {"name": "Bash", "description": "run a command in a shell",
         "input_schema": {"type": "object",
                          "properties": {"command": {"type": "string"}},
                          "required": ["command"]},
         "command": ["/usr/local/libexec/bash-tool", "--quiet"],
         "cwd": "/srv/work"},
        {"name": "Read", "description": "read a file",
         "input_schema": {"type": "object"},
         "command": ["/usr/local/libexec/read-tool"]}
    ])
}

/// **One document, two readings** (REMOTE §5.2): the local half is read, and
/// the advertisement is the same rows with that half dropped — verbatim
/// schemas included, so a host cannot offer what it cannot run.
#[test]
fn the_advertisement_is_this_document_with_the_local_half_dropped() {
    let dir = tempdir().expect("tmp");
    let set = read(&write(dir.path(), &document())).expect("read");
    assert_eq!(set.len(), 2);
    assert_eq!(
        set.first().map(|l| l.command.clone()),
        Some(vec![
            "/usr/local/libexec/bash-tool".to_owned(),
            "--quiet".to_owned()
        ])
    );
    assert_eq!(
        set.first().and_then(|l| l.cwd.clone()),
        Some(PathBuf::from("/srv/work"))
    );
    assert_eq!(
        set.get(1).and_then(|l| l.cwd.clone()),
        None,
        "cwd is optional"
    );

    let presented = advertisement(&set);
    assert_eq!(
        presented.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
        vec!["Bash".to_owned(), "Read".to_owned()]
    );
    assert_eq!(
        presented.first().map(|t| t.input_schema.clone()),
        document()[0].get("input_schema").cloned(),
        "the schema crosses verbatim"
    );
    // The whole of the projection: nothing in the presented element says how it
    // is run, because the presented element is REMOTE §5.1's three facts.
    assert_eq!(
        crate::registry::tools::one(presented.first().expect("a row"))
            .as_object()
            .map(serde_json::Map::len),
        Some(3)
    );
}

/// A name is resolved by position against the very list the caller passed in —
/// an index, never a borrow, and `None` for a name this machine does not carry.
#[test]
fn a_name_resolves_by_position_or_not_at_all() {
    let dir = tempdir().expect("tmp");
    let set = read(&write(dir.path(), &document())).expect("read");
    assert_eq!(position(&set, "Read"), Some(1));
    assert_eq!(position(&set, "Rm"), None);
}

/// **Absent is a refusal, not the empty set**: starting a tool host is an
/// explicit act, so a machine with no document is told so rather than quietly
/// advertising nothing.
#[test]
fn an_absent_document_refuses_naming_the_path() {
    let dir = tempdir().expect("tmp");
    let file = dir.path().join(TOOLS);
    let e = read(&file).expect_err("no config");
    assert!(
        e.contains(TOOLS) && e.contains("no tool-host config"),
        "{e}"
    );
}

/// Every other way a document fails to be one, each naming what it was.
#[test]
fn a_malformed_document_refuses_by_what_is_wrong_with_it() {
    let dir = tempdir().expect("tmp");
    for (doc, needle) in [
        (json!({"name": "Bash"}), "not a JSON array"),
        (json!(["Bash"]), "not a JSON object"),
        (
            json!([{"name": "Bash", "description": "d", "input_schema": {}}]),
            "\"command\"",
        ),
        (
            json!([{"name": "Bash", "description": "d", "input_schema": {},
                    "command": []}]),
            "empty argv",
        ),
        (
            json!([{"name": "Bash", "description": "d", "command": ["x"]}]),
            "input_schema",
        ),
        (
            json!([{"name": "a/b", "description": "d", "input_schema": {},
                    "command": ["x"]}]),
            "unusable tool name",
        ),
        (
            json!([{"name": "Bash", "description": "d", "input_schema": {}, "command": ["x"]},
                   {"name": "Bash", "description": "e", "input_schema": {}, "command": ["y"]}]),
            "duplicate tool name",
        ),
        (
            json!([{"name": "Bash", "description": "d", "input_schema": {},
                    "command": ["x"], "cwd": 7}]),
            "\"cwd\"",
        ),
    ] {
        let e = read(&write(dir.path(), &doc)).expect_err("not a config");
        assert!(e.contains(needle), "{doc}\nsaid: {e}");
    }
    let file = dir.path().join(TOOLS);
    std::fs::write(&file, "not json").expect("config");
    assert!(read(&file).is_err(), "unparseable is a refusal too");
}

/// The document sits beside the wire material, never inside the world subtree —
/// a reseed must not take an operator's file with it.
#[test]
fn the_document_is_the_data_roots_own_sibling_of_the_world() {
    let world = crate::test_support::no_world();
    assert_eq!(path(&world), world.yog_data_root().join(TOOLS));
    assert_ne!(
        path(&world).parent(),
        Some(world.yog_state_root().as_path())
    );
}
