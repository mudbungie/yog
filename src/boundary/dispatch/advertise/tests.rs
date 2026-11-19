//! The advertisement's gate: who is asking, what a set must be, and where it
//! lands (REMOTE §5).

use super::*;
use crate::boundary::dispatch::Caller;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::registry::Client;
use crate::registry::mailbox::{Mailbox, Taken};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

fn tool(name: &str) -> Tool {
    Tool {
        name: name.to_owned(),
        description: "does a thing".to_owned(),
        input_schema: json!({"type": "object"}),
        subject_cwd: false,
    }
}

/// The ordinary state of an advertising machine: registered, and therefore
/// **registered somewhere** (REMOTE §5.1) — a set is presented into the
/// workspaces its client is registered in, so a client in none is refused
/// before anything is stored (bl-6b14). Every beat below but the two about that
/// refusal starts here.
fn seated(state_root: &Path, client: &Client) -> Deps {
    crate::registry::register(state_root, client, "alba").expect("the registration's own file");
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

/// A mailbox whose hold outlasts the beat — twenty seconds — so a read that
/// ended inside it was ended by the advertisement and not by its own clock.
fn patient(mut deps: Deps) -> Deps {
    deps.caller.mailbox = Mailbox::holding(4000, Duration::from_millis(5));
    deps
}

/// Park a read for `client` on its own thread, and answer once it is parked.
fn parked(deps: &Deps, client: &str) -> std::thread::JoinHandle<Result<Taken, String>> {
    let (mail, name) = (deps.caller.mailbox.clone(), client.to_owned());
    let reader = std::thread::spawn(move || mail.take(&name));
    crate::test_support::foot::parked(&deps.caller.mailbox, client);
    reader
}

/// **A restarted foot with an edited tool document is serving at once**
/// (REMOTE §5.1, bl-96e0). The read its predecessor parked is still standing —
/// a parked read sleeps through its peer's death — and the new process's
/// advertisement used to be refused for as long as it stood. Now the set comes
/// into force, the old read is ended and told, and the read the new process
/// makes next is granted without waiting.
#[test]
fn a_restart_with_a_changed_set_is_serving_at_once() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = patient(seated(root.path(), &laptop));
    advertise(&deps, &[tool("Bash")]).expect("stored");
    let old = parked(&deps, "laptop");

    assert_eq!(
        advertise(&deps, &[tool("Read")]),
        Ok(Reply::Advertised { wrote: true })
    );
    assert_eq!(
        crate::registry::tools::read(root.path(), &laptop),
        vec![tool("Read")],
        "the new set is the set in force"
    );
    let serving = deps.caller.mailbox.reading("laptop");
    assert!(
        serving.is_ok(),
        "the new activation's read is granted at once"
    );
    assert_eq!(
        old.join().expect("the old read"),
        Ok(Taken::Superseded),
        "and a predecessor that is still alive learns it lost the queue"
    );
}

/// **So is one whose document did not change** — the plain restart, which was
/// let through at the advertisement and then refused at its own first read.
#[test]
fn a_restart_with_the_same_set_is_serving_at_once() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = patient(seated(root.path(), &laptop));
    advertise(&deps, &[tool("Bash")]).expect("stored");
    let old = parked(&deps, "laptop");

    assert_eq!(
        advertise(&deps, &[tool("Bash")]),
        Ok(Reply::Advertised { wrote: false })
    );
    let serving = deps.caller.mailbox.reading("laptop");
    assert!(serving.is_ok(), "granted at once");
    assert_eq!(old.join().expect("the old read"), Ok(Taken::Superseded));
}

/// **A refused advertisement ends nothing.** The read is ended after the store,
/// so a set that could not be presented leaves the machine that is serving
/// exactly as it was.
#[test]
fn a_refused_advertisement_leaves_the_parked_read_alone() {
    let root = tempdir().expect("tempdir");
    let laptop = Client::parse("laptop").expect("identity");
    let deps = seated(root.path(), &laptop);
    let _parked = deps.caller.mailbox.reading("laptop").expect("its read");
    advertise(&deps, &[tool("Read"), tool("Read")]).expect_err("declined");
    assert!(deps.caller.mailbox.serving("laptop"));
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
fn a_client_registered_nowhere_is_refused_naming_the_registration() {
    let root = tempdir().expect("tempdir");
    let devbox = Client::parse("devbox").expect("identity");
    let deps = deps(root.path(), devbox.clone());
    let refusal = advertise(&deps, &[tool("Bash")]).expect_err("presented to nobody");
    assert!(refusal.contains("registered in no workspace"), "{refusal}");
    assert!(refusal.contains("clients/devbox/workspaces"), "{refusal}");
    assert!(
        crate::registry::tools::read(root.path(), &devbox).is_empty(),
        "and nothing is stored for nobody to see"
    );
}
