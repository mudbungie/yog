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
        subject_cwd: false,
    }
}

/// The ordinary state of an advertising machine: enrolled, and therefore
/// **registered somewhere** (REMOTE §5.1) — a set is presented into the
/// workspaces its client is registered in, so a client in none is refused
/// before anything is stored (bl-6b14). Every beat below but the two about that
/// refusal starts here.
fn seated(state_root: &Path, client: &Client) -> Deps {
    crate::registry::register(state_root, client, "alba").expect("the enrolment's own file");
    deps(state_root, client.clone())
}

fn deps(state_root: &Path, client: Client) -> Deps {
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
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
/// name — and the receipt says the document was **written**, because there was
/// none before. It still echoes no set: what it carries is what happened to the
/// file, which is the one fact the advertising box cannot compute.
#[test]
fn a_connections_set_lands_under_its_own_identity() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = seated(root.path(), &laptop);
    assert_eq!(advertise(&deps, &[tool("Bash")]), Ok(advertised(true)));
    assert_eq!(
        crate::registry::tools::read(root.path(), &laptop),
        vec![tool("Bash")]
    );
}

/// **The re-presentation is silent and the restoration is not** (REMOTE §5.1,
/// bl-66d4). Presenting the same set again writes nothing and answers `false`,
/// which is every reconnect and every §5.3 hand-off; presenting it again after
/// something blanked it writes and answers `true`, and that `true` is the box
/// learning it was disarmed while it was absent. Both arrive as the identical
/// `ok` without the field, which is why it exists.
#[test]
fn only_a_write_answers_true_and_a_restoration_is_a_write() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = seated(root.path(), &laptop);
    assert_eq!(advertise(&deps, &[tool("Bash")]), Ok(advertised(true)));
    assert_eq!(advertise(&deps, &[tool("Bash")]), Ok(advertised(false)));

    // A rival bearing the same certificate blanks the box while it is running
    // a tool — the window bl-1462's guards cannot close, because an executing
    // foot holds no parked read.
    assert_eq!(advertise(&deps, &[]), Ok(advertised(true)));
    assert_eq!(advertise(&deps, &[tool("Bash")]), Ok(advertised(true)));
}

/// The receipt, spelled once for the tests that read it.
fn advertised(wrote: bool) -> crate::boundary::reply::Reply {
    crate::boundary::reply::Reply::Advertised { wrote }
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
    let deps = seated(root.path(), &laptop);
    advertise(&deps, &[tool("Bash")]).expect("stored");
    let refusal = advertise(&deps, &[tool("Read"), tool("Read")]).expect_err("declined");
    assert!(refusal.contains("duplicate tool name"), "{refusal}");
    assert_eq!(
        crate::registry::tools::read(root.path(), &laptop),
        vec![tool("Bash")],
        "a refused presentation changes nothing"
    );
}

/// **A serving machine's set may not be replaced under it** (bl-1462). The
/// store was last-writer-wins on the identity, so a second connection bearing
/// the same certificate could present the empty set and disarm a healthy host
/// — engine-side every later invoke was refused for a tool that plainly
/// existed, and neither end was told.
#[test]
fn a_second_connection_may_not_blank_a_serving_machines_set() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = seated(root.path(), &laptop);
    advertise(&deps, &[tool("Bash")]).expect("stored");

    let parked = deps
        .caller
        .mailbox
        .reading("laptop")
        .expect("the machine's own read");
    let refusal = advertise(&deps, &[]).expect_err("refused while serving");
    assert!(refusal.contains("\"laptop\""), "{refusal}");
    assert!(
        refusal.contains("would never learn it was disarmed"),
        "{refusal}"
    );
    assert_eq!(
        crate::registry::tools::read(root.path(), &laptop),
        vec![tool("Bash")],
        "the serving machine keeps its tools"
    );

    // The ordinary path is untouched in both directions: the machine itself
    // re-presenting the set in force writes nothing and is not refused, and
    // once nothing is serving, a changed set lands as it always did.
    advertise(&deps, &[tool("Bash")]).expect("an unchanged set is not a replacement");
    drop(parked);
    advertise(&deps, &[tool("Read")]).expect("nobody is serving");
    assert_eq!(
        crate::registry::tools::read(root.path(), &laptop),
        vec![tool("Read")]
    );
}

/// An unwritable store is a refusal, not a panic. The client is seated, so the
/// refusal is the store's own and not the registration gate's — a directory
/// where the document goes is what no write can get past.
#[test]
fn an_unwritable_store_refuses() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = seated(root.path(), &laptop);
    std::fs::create_dir_all(
        crate::registry::dir(root.path(), &laptop).join(crate::registry::tools::TOOLS),
    )
    .expect("a directory in the document's place");
    assert!(advertise(&deps, &[tool("Bash")]).is_err());
}

/// **An advertisement into nothing is told so** (REMOTE §5.1, bl-6b14). The
/// documented way to provision a second machine — `wire-certs WIRE_LEAF=` —
/// mints a leaf and registers it nowhere, so the foot dialled, advertised,
/// parked on its mailbox read and served nobody, in silence at both ends. The
/// refusal names the client and the act that seats it, and nothing is stored
/// for nobody to read.
#[test]
fn a_client_registered_nowhere_is_refused_naming_the_enrolment() {
    let root = tempdir().expect("tempdir");
    let devbox = Client::parse("devbox").expect("identity");
    let deps = deps(root.path(), devbox.clone());
    let refusal = advertise(&deps, &[tool("Bash")]).expect_err("presented to nobody");
    assert!(refusal.contains("registered in no workspace"), "{refusal}");
    assert!(refusal.contains("/enroll devbox foot"), "{refusal}");
    assert!(
        crate::registry::tools::read(root.path(), &devbox).is_empty(),
        "and nothing is stored for nobody to see"
    );
}
