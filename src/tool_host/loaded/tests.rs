//! The loaded set: its address, its one spelling, and the union that is the
//! only way it ever changes (REMOTE §5, bl-c907).

use super::*;
use tempfile::TempDir;

fn tool(name: &str) -> Tool {
    Tool {
        name: name.to_owned(),
        description: format!("what {name} does"),
        input_schema: json!({"type": "object", "properties": {"cmd": {"type": "string"}}}),
        subject_cwd: false,
    }
}

fn entry(client: &str, name: &str) -> Entry {
    Entry {
        client: client.to_owned(),
        tool: tool(name),
    }
}

/// The presented name is the client's, an underscore, the advertised name —
/// always, so two laptops both advertising `Bash` are two callable names.
#[test]
fn a_presented_name_carries_the_client_that_advertises_it() {
    assert_eq!(entry("laptop", "Bash").presented(), "laptop_Bash");
    assert_eq!(entry("desk", "Bash").presented(), "desk_Bash");
}

/// A name a provider's tool block would refuse is refused here, at the load.
#[test]
fn a_callable_name_is_ascii_word_characters_and_bounded() {
    assert!(callable("laptop_Bash"));
    assert!(callable("a-b_C9"));
    assert!(callable(&"x".repeat(64)));
    assert!(!callable(""));
    assert!(!callable(&"x".repeat(65)));
    assert!(!callable("laptop.local_Bash"));
}

/// The document hangs off yog's own state root, one per agent per workspace.
#[test]
fn the_document_is_addressed_by_workspace_and_agent() {
    let root = Path::new("/home/u/state/yog");
    assert_eq!(
        path(root, "home", "dulcet-mongoose").expect("a usable address"),
        root.join(LOADED).join("home").join("dulcet-mongoose.json")
    );
}

/// A name that is not a plain path component has no document — the same
/// emptiness a fresh agent reads, never a name that addresses the filesystem.
#[test]
fn a_name_that_could_address_the_filesystem_has_no_document() {
    let root = Path::new("/home/u/state/yog");
    assert!(path(root, "../escape", "a").is_none());
    assert!(path(root, "home", "..").is_none());
    assert!(read(root, "home", "..").is_empty());
}

/// Encode → decode is the identity, schema verbatim, through the *same*
/// element spelling the advertisement and the boundary codec spend.
#[test]
fn a_set_survives_its_one_spelling() {
    let set = vec![entry("laptop", "Bash"), entry("desk", "Read")];
    let wire = encode(&set);
    assert_eq!(decode(&wire).expect("decoded"), set);
    assert_eq!(
        wire[0]["tool"],
        crate::registry::tools::one(&set[0].tool),
        "the element is the registry's spelling, not a second one"
    );
}

/// Strict, naming the offending key — the advertisement's own decode
/// discipline applied to yog's own document.
#[test]
fn a_malformed_document_refuses_naming_what_is_wrong() {
    assert!(
        decode(&json!({}))
            .expect_err("not an array")
            .contains("array")
    );
    assert!(
        decode(&json!([7]))
            .expect_err("not an object")
            .contains("object")
    );
    assert!(
        decode(&json!([{"tool": {"name": "B", "description": "d", "input_schema": {}}}]))
            .expect_err("no client")
            .contains("client")
    );
    assert!(
        decode(&json!([{"client": "laptop"}]))
            .expect_err("no tool")
            .contains("tool")
    );
    assert!(
        decode(&json!([{"client": "laptop", "tool": {"name": "B"}}]))
            .expect_err("bad tool")
            .contains("description")
    );
}

/// An absent, unreadable or undecodable document is the empty set — the same
/// posture every agent has before its first load.
#[test]
fn an_unreadable_document_reads_as_nothing_loaded() {
    let root = TempDir::new().expect("tmp");
    assert!(read(root.path(), "home", "agent").is_empty());

    let file = path(root.path(), "home", "agent").expect("address");
    std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&file, b"{").expect("write");
    assert!(read(root.path(), "home", "agent").is_empty());

    std::fs::write(&file, b"{\"not\": \"an array\"}").expect("write");
    assert!(read(root.path(), "home", "agent").is_empty());
}

/// A load lands, survives the process, and is read back whole.
#[test]
fn a_load_is_durable() {
    let root = TempDir::new().expect("tmp");
    let one = entry("laptop", "Bash");
    let all = add(root.path(), "home", "agent", std::slice::from_ref(&one)).expect("added");
    assert_eq!(all, vec![one.clone()]);
    assert_eq!(read(root.path(), "home", "agent"), vec![one]);
}

/// Union by presented name, later wins, sorted: a second load adds, and a
/// re-load of the same name refreshes the frozen definition rather than
/// doubling it.
#[test]
fn a_second_load_unions_and_a_reload_refreshes() {
    let root = TempDir::new().expect("tmp");
    add(
        root.path(),
        "home",
        "agent",
        &[entry("laptop", "Bash"), entry("desk", "Read")],
    )
    .expect("added");

    let mut refreshed = entry("laptop", "Bash");
    refreshed.tool.description = "a newer sentence".to_owned();
    let all = add(
        root.path(),
        "home",
        "agent",
        &[refreshed.clone(), entry("laptop", "Write")],
    )
    .expect("added");

    assert_eq!(
        all.iter().map(Entry::presented).collect::<Vec<_>>(),
        vec!["desk_Read", "laptop_Bash", "laptop_Write"],
        "unioned by presented name, sorted"
    );
    assert_eq!(all[1], refreshed, "the later definition won");
}

/// A load whose address is not a pair of path components refuses, naming it —
/// the write half of the rule [`path`] enforces on the read half.
#[test]
fn a_load_at_an_unusable_address_refuses() {
    let root = TempDir::new().expect("tmp");
    let e = add(root.path(), "..", "agent", &[entry("laptop", "Bash")]).expect_err("refused");
    assert!(e.to_string().contains("unusable address"), "{e}");
}
