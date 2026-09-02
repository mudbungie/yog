//! **The intake threads the identity to the ACT side** (REMOTE §5, bl-4e08) —
//! the other half of what `answer_as` already does for reads. A presentation
//! arriving on a connection lands under that connection's certificate; the same
//! bytes arriving at the world's own door refuse, because there is nobody for
//! the set to belong to.

use super::*;
use crate::cli_outbound::Cli;
use serde_json::json;
use tempfile::tempdir;

fn set() -> serde_json::Value {
    json!([{"name": "Bash", "description": "run a command",
            "input_schema": {"type": "object"}}])
}

fn quiet(root: &std::path::Path, data: &std::path::Path) -> ConsumerCtx {
    over(
        root,
        world_of(data, &["home"]),
        data.to_path_buf(),
        Cli::new("/no/such/litany"),
    )
}

/// A tool host's set lands under the certificate that presented it, and the
/// receipt carries nothing but the fact that it landed.
#[test]
fn a_connections_advertisement_lands_under_its_own_identity() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let ctx = quiet(root.path(), data.path());
    let laptop = seat("laptop");
    let reply = ctx.answer_as(&laptop, &json!({"op": "advertise", "tools": set()}));
    assert_eq!(reply["kind"], "advertised");
    assert_eq!(reply["ok"], true);
    let stored = crate::registry::tools::read(root.path(), &laptop.client);
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].name, "Bash");
    // Another certificate's set is its own: cross-client collisions are legal.
    ctx.answer_as(&seat("phone"), &json!({"op": "advertise", "tools": set()}));
    assert_eq!(
        crate::registry::tools::read(root.path(), &client("phone"))[0].name,
        "Bash"
    );
}

/// **The world's own door has no client** (REMOTE §5): the deposit inbox and
/// `yog gesture` carry `local`, so the same envelope refuses in band, with a
/// sentence rather than silence.
#[test]
fn the_in_world_intake_refuses_an_advertisement() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let refusal =
        quiet(root.path(), data.path()).answer(&json!({"op": "advertise", "tools": set()}));
    assert_eq!(refusal["ok"], false);
    assert!(
        refusal["error"]
            .as_str()
            .unwrap_or_default()
            .contains("no client identity"),
        "{refusal}"
    );
}

/// **The roster is the read half, and it is scoped like every other** (REMOTE
/// §4, §5): a client sees its own workspace's clients, and an unregistered name
/// earns the resolver's own refusal rather than an empty list.
#[test]
fn the_roster_answers_the_registered_set_of_a_scoped_workspace() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let ctx = quiet(root.path(), data.path());
    let laptop = seat("laptop");
    crate::registry::register(root.path(), &laptop.client, "home").unwrap();
    ctx.answer_as(&laptop, &json!({"op": "advertise", "tools": set()}));
    let reply = ctx.answer_as(&laptop, &json!({"op": "clients", "workspace": "home"}));
    assert_eq!(reply["kind"], "clients");
    assert_eq!(reply["rows"][0]["client"], "laptop");
    assert_eq!(reply["rows"][0]["tools"][0]["name"], "Bash");
    // Presence is the wire server's RAM and this context holds an empty one:
    // a client that is registered and not connected is a row all the same.
    assert_eq!(reply["rows"][0]["present"], false);
    let refusal = ctx.answer_as(
        &seat("stranger"),
        &json!({"op": "clients", "workspace": "home"}),
    );
    assert_eq!(
        refusal["error"],
        "unknown workspace \"home\" — none is enumerated here"
    );
}
