//! The scoped intake (REMOTE §4, bl-8bbc): scoping as narrowing, absence as the
//! resolver's own refusal, and the create that auto-registers its client.

use super::*;
use crate::cli_outbound::Cli;
use serde_json::json;
use tempfile::tempdir;

/// **Enumeration answers the registered set** (REMOTE §4, bl-8bbc): a
/// connection sees the workspaces its certificate is seated in, and the rest
/// are not withheld — they are simply not there.
#[test]
fn a_wire_client_enumerates_only_its_registrations() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let ctx = over(
        root.path(),
        world_of(data.path(), &["home", "corp"]),
        data.path().to_path_buf(),
        Cli::new("/no/such/litany"),
    );
    let phone = seat("phone");
    crate::registry::register(root.path(), &phone.client, "home").unwrap();
    let reply = ctx.answer_as(&phone, &json!({"op": "workspaces"}));
    assert_eq!(listed(&reply), ["home".to_owned()]);

    // An in-world caller holds no certificate and is not scoped (§3): the
    // deposit inbox is the world's own residents' door. Sorted by path, because
    // the set is the §3.1 enumeration (I9's stable roster) and not the order a
    // fixture happened to name them in.
    assert_eq!(
        listed(&ctx.answer(&json!({"op": "workspaces"}))),
        ["corp".to_owned(), "home".to_owned()]
    );
    // A certificate the operator has not seated sees no workspace at all —
    // the general path with no registrations, not a bootstrap case.
    assert!(listed(&ctx.answer_as(&seat("stranger"), &json!({"op": "workspaces"}))).is_empty());
}

/// **Absence, not a scope error** (§4): a gesture naming an unregistered
/// workspace is refused in the identical bytes a workspace nobody founded
/// earns, so no reply confirms that the workspace exists.
#[test]
fn an_unregistered_workspace_refuses_exactly_as_an_unknown_one() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let ctx = over(
        root.path(),
        world_of(data.path(), &["home", "corp"]),
        data.path().to_path_buf(),
        Cli::new("/no/such/litany"),
    );
    let phone = seat("phone");
    crate::registry::register(root.path(), &phone.client, "home").unwrap();
    let hidden = ctx.answer_as(&phone, &json!({"op": "conversations", "workspace": "corp"}));
    let absent = ctx.answer_as(
        &phone,
        &json!({"op": "conversations", "workspace": "corp2"}),
    );
    assert_eq!(hidden["ok"], false);
    assert_eq!(hidden["error"], "unknown workspace \"corp\"");
    assert_eq!(absent["error"], "unknown workspace \"corp2\"");
    // The workspace it IS registered in answers.
    assert_eq!(
        ctx.answer_as(&phone, &json!({"op": "conversations", "workspace": "home"}))["ok"],
        true
    );
}

/// A torn envelope over the wire refuses in-band exactly as a deposited one
/// does — the scoped intake decodes with the same codec and adds no verb.
#[test]
fn a_torn_envelope_refuses_over_the_wire_too() {
    let root = tempdir().unwrap();
    let refusal = ctx(root.path()).answer_as(&seat("phone"), &json!({"op": "enhance"}));
    assert_eq!(refusal["ok"], false);
    assert!(
        refusal["error"]
            .as_str()
            .unwrap_or_default()
            .contains("enhance")
    );
}

/// **A workspace created over the wire auto-registers its creating client**
/// (REMOTE §4, bl-8bbc) — with nothing to detect. Under scope a gesture can
/// name only a registered workspace or one it just founded, so a successful
/// answer naming a workspace outside the scope IS a creation, and the next
/// gesture reads it as an ordinary registration.
#[test]
fn a_workspace_created_over_the_wire_registers_its_creator() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let bin = tempdir().unwrap();
    seed(data.path());
    let ctx = over(
        root.path(),
        world_of(data.path(), &[]),
        data.path().to_path_buf(),
        fake_litany(bin.path()),
    );
    let phone = seat("phone");
    let reply = ctx.answer_as(
        &phone,
        &json!({"op": "prepare", "workspace": "fresh", "payload": {"rung": "bare"}}),
    );
    assert_eq!(reply["kind"], "prepared", "{reply}");
    assert!(
        crate::binding::workspace_path(data.path(), "fresh").is_dir(),
        "the raise founded it under yog's flat names root"
    );
    assert_eq!(
        crate::registry::registered(root.path(), &phone.client),
        std::collections::BTreeSet::from(["fresh".to_owned()])
    );
    // And nobody else was seated by it.
    assert!(crate::registry::registered(root.path(), &client("laptop")).is_empty());
}

/// **The raise can only found, never join** — a name already taken refuses
/// with the resolver's own sentence, which is what keeps a create from being a
/// way into a workspace the scope hides.
#[test]
fn a_create_naming_a_workspace_that_exists_refuses_rather_than_joining_it() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let bin = tempdir().unwrap();
    seed(data.path());
    let taken = crate::binding::workspace_path(data.path(), "corp");
    std::fs::create_dir_all(&taken).unwrap();
    let ctx = over(
        root.path(),
        world_of(data.path(), &["corp"]),
        data.path().to_path_buf(),
        fake_litany(bin.path()),
    );
    let phone = seat("phone");
    let refusal = ctx.answer_as(
        &phone,
        &json!({"op": "prepare", "workspace": "corp", "payload": {"rung": "bare"}}),
    );
    assert_eq!(refusal["ok"], false);
    assert_eq!(refusal["error"], "unknown workspace \"corp\"");
    assert!(crate::registry::registered(root.path(), &phone.client).is_empty());
}

/// The raised name becomes a directory, so it is checked on the same terms the
/// client identity is: a name that could carry a separator is a name that could
/// address the filesystem.
#[test]
fn a_raise_naming_something_that_is_not_a_plain_component_refuses() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let ctx = over(
        root.path(),
        world_of(data.path(), &[]),
        data.path().to_path_buf(),
        Cli::new("/no/such/litany"),
    );
    for name in ["../elsewhere", ".."] {
        let refusal = ctx.answer_as(
            &seat("phone"),
            &json!({"op": "prepare", "workspace": name, "payload": {"rung": "bare"}}),
        );
        assert_eq!(refusal["ok"], false, "{name}");
        assert_eq!(refusal["error"], format!("unknown workspace {name:?}"));
    }
    assert!(crate::registry::registered(root.path(), &client("phone")).is_empty());
}

/// **A held read is one request, and it spends the scope like one** (REMOTE §4,
/// bl-73e7). The address resolves at connect, under the caller's registrations,
/// so an unregistered workspace answers `None` here — and `None` is not a
/// second refusal path: the intake falls back to the one-frame answer, which
/// refuses in the resolver's own words. A seat cannot tell a refused follow
/// from any other refused read, which is exactly REMOTE §4's absence.
#[test]
fn a_follow_resolves_its_address_under_the_callers_scope() {
    let root = tempdir().unwrap();
    let data = tempdir().unwrap();
    let ctx = over(
        root.path(),
        world_of(data.path(), &["home", "corp"]),
        data.path().to_path_buf(),
        Cli::new("/no/such/litany"),
    );
    let phone = seat("phone");
    crate::registry::register(root.path(), &phone.client, "home").unwrap();
    let follow = |workspace: &str, agent: &str| json!({"op": "follow", "workspace": workspace, "agent": agent});

    // Nothing this seat may not see: the workspace it is not seated in, and a
    // conversation the workspace does not carry, both answer no stream.
    assert!(ctx.follow(&phone, &follow("corp", "c-1")).is_none());
    assert!(ctx.follow(&phone, &follow("home", "nobody")).is_none());
    // And the refusal a seat actually reads is the resolver's, one frame long.
    let said = |workspace: &str| {
        ctx.answer_as(&phone, &follow(workspace, "c-1"))["error"]
            .as_str()
            .unwrap_or_default()
            .replace(workspace, "<name>")
    };
    assert_eq!(
        said("corp"),
        said("nowhere"),
        "absence, not a scope error: a workspace this seat is not seated in \
         refuses in the identical bytes one nobody founded earns"
    );
}

/// Every other request is `None` here too, and that is the whole of what makes
/// the intake two arms rather than three: a read that is not follow-class is
/// answered by the one function that answers everything else.
#[test]
fn nothing_but_a_follow_is_a_stream() {
    let root = tempdir().unwrap();
    let ctx = ctx(root.path());
    for request in [
        json!({"op": "workspaces"}),
        json!({"op": "teleport"}),
        json!("not even an object"),
    ] {
        assert!(ctx.follow(&seat("phone"), &request).is_none());
    }
}
