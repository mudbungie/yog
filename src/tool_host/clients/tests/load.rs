//! **The `load` op** (REMOTE §5, §5.1; bl-c907): the one `clients` op that
//! writes.
//!
//! `list` and `get` observe; this one freezes the named definitions into the
//! agent's durable set and says what became callable. That makes its failures
//! its own — a name the client does not advertise, a composed name a provider
//! would refuse, a document that cannot be written — and each of them must
//! refuse **whole**, because a partial load leaves the model believing it holds
//! a tool it does not.

use super::*;

/// `load` freezes the named definitions into the agent's durable set and says
/// what became callable.
#[test]
fn load_makes_the_named_tools_callable_and_durable() {
    let root = TempDir::new().expect("tmp");
    let said = against(
        root.path(),
        vec![row("laptop", true, vec![tool("Bash"), tool("Read")])],
        &json!({"op": "load", "client": "laptop", "tools": ["Bash"]}),
    )
    .expect("loaded");
    assert!(said.contains("loaded, observed 1970-01-01"), "{said}");
    assert!(said.contains("laptop_Bash — what Bash does"), "{said}");
    assert!(said.contains("now holds 1 loaded tool."), "{said}");
    assert!(said.contains("There is no unload"), "{said}");

    let held = crate::tool_host::loaded::read(root.path(), "home", "dulcet-mongoose");
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].tool, tool("Bash"), "the definition is frozen whole");
}

/// A load of a name the client does not advertise refuses whole: a partial
/// load would leave the model believing it holds a tool it does not.
#[test]
fn an_unadvertised_tool_refuses_the_whole_load() {
    let root = TempDir::new().expect("tmp");
    let e = against(
        root.path(),
        vec![row("laptop", true, vec![tool("Bash")])],
        &json!({"op": "load", "client": "laptop", "tools": ["Bash", "Nope"]}),
    )
    .expect_err("refused");
    assert!(e.contains("advertises no tool \"Nope\""), "{e}");
    assert!(
        crate::tool_host::loaded::read(root.path(), "home", "dulcet-mongoose").is_empty(),
        "nothing was recorded"
    );
}

/// A pair whose composed name a provider would refuse declines at the load,
/// naming the name — before the model is ever told it exists.
#[test]
fn a_name_a_provider_would_refuse_declines_at_the_load() {
    let root = TempDir::new().expect("tmp");
    let e = against(
        root.path(),
        vec![row("laptop.local", true, vec![tool("Bash")])],
        &json!({"op": "load", "client": "laptop.local", "tools": ["Bash"]}),
    )
    .expect_err("refused");
    assert!(
        e.contains("\"laptop.local_Bash\" is not a usable tool name"),
        "{e}"
    );
}

/// A load the agent's document cannot hold refuses, naming the address — the
/// write half of the loaded set's own component rule.
#[test]
fn a_load_that_cannot_be_recorded_says_so() {
    let root = TempDir::new().expect("tmp");
    let mut s = site(root.path(), budget());
    s.agent = "..".to_owned();
    let (handle, _seen) = engine(
        root.path(),
        &reply::encode(&Reply::Clients(vec![row(
            "laptop",
            true,
            vec![tool("Bash")],
        )])),
    );
    let e = answer(
        &s,
        &json!({"op": "load", "client": "laptop", "tools": ["Bash"]}),
        &quiet(),
    )
    .expect_err("unrecordable");
    handle.join().expect("engine");
    assert!(e.contains("recording the load"), "{e}");
}
