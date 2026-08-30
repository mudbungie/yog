//! The advertisement's gate: who is asking, what a set must be, and where it
//! lands (REMOTE §5).

use super::*;
use crate::boundary::dispatch::Caller;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::registry::Client;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;

fn tool(name: &str) -> Tool {
    Tool {
        name: name.to_owned(),
        description: "does a thing".to_owned(),
        input_schema: json!({"type": "object"}),
    }
}

fn deps(state_root: &Path, client: Client) -> Deps {
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::xdg::Env::from_env(),
        snapshot: Arc::new(snapshot(
            Path::new("/names/alba"),
            "alba",
            Vec::new(),
            Vec::new(),
        )),
        caller: Caller {
            client,
            ..Caller::default()
        },
    }
}

/// The identity is the intake's, so the set lands under the connection's own
/// name — and the receipt carries nothing, because the stored set *is* the set
/// the gesture carried.
#[test]
fn a_connections_set_lands_under_its_own_identity() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = deps(root.path(), laptop.clone());
    assert_eq!(
        advertise(&deps, &[tool("Bash")]),
        Ok(crate::boundary::reply::Reply::Advertised)
    );
    assert_eq!(
        crate::registry::tools::read(root.path(), &laptop),
        vec![tool("Bash")]
    );
}

/// **An intake with no client identity refuses in band** (REMOTE §5): the
/// deposit inbox, `yog gesture` and the window all carry `local`, and a tool
/// set with no certificate behind it has nobody to belong to.
#[test]
fn an_in_world_caller_is_refused_with_a_sentence() {
    let root = tempdir().expect("tempdir");
    let refusal = advertise(&deps(root.path(), Client::local()), &[tool("Bash")])
        .expect_err("refused in band");
    assert!(refusal.contains("no client identity"), "{refusal}");
    assert!(!root.path().join(crate::registry::CLIENTS).exists());
}

/// A set that cannot be addressed declines loudly and writes nothing — the
/// validation stands ahead of the store, so a refused presentation leaves the
/// stored set exactly as it was.
#[test]
fn a_colliding_name_declines_and_leaves_the_stored_set_alone() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = deps(root.path(), laptop.clone());
    advertise(&deps, &[tool("Bash")]).expect("stored");
    let refusal = advertise(&deps, &[tool("Read"), tool("Read")]).expect_err("declined");
    assert!(refusal.contains("duplicate tool name"), "{refusal}");
    assert_eq!(
        crate::registry::tools::read(root.path(), &laptop),
        vec![tool("Bash")],
        "a refused presentation changes nothing"
    );
}

/// An unwritable registry is a refusal, not a panic.
#[test]
fn an_unwritable_registry_refuses() {
    let root = tempdir().expect("tempdir");
    std::fs::write(root.path().join(crate::registry::CLIENTS), b"file").expect("write");
    let deps = deps(root.path(), Client::parse("laptop").expect("identity"));
    assert!(advertise(&deps, &[tool("Bash")]).is_err());
}
