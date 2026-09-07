//! **A workspace this box founds is seen by the seats this box minted**
//! (bl-bd48, bl-0fbc) — [`founding`](crate::boundary::consumer::founding)'s own
//! beats, at the intake, because the rule is about what two doors do and not
//! about a function.

use super::*;
use serde_json::json;
use tempfile::tempdir;

/// A context whose world holds real wire material — the box's own mint, which
/// is what decides which seats exist to be registered.
fn provisioned(
    root: &std::path::Path,
    data: &std::path::Path,
    bin: &std::path::Path,
) -> ConsumerCtx {
    seed(data);
    let world = crate::test_support::world_under(data);
    crate::test_support::wire::mint(&crate::wire::material::dir(&world));
    over_world(
        root,
        world_of(data, &[]),
        data.to_path_buf(),
        fake_litany(bin),
        world,
    )
}

/// The bare-rung prepare envelope, aimed by name.
fn prepare(ws: &str) -> serde_json::Value {
    json!({"op": "prepare", "workspace": ws, "payload": {"rung": "bare"}})
}

/// **The dead end, closed** (bl-0fbc): `yog gesture --ws fleet '/prepare'` is
/// the act both READMEs teach on a headless box, and the in-world identity that
/// makes it is registered nowhere — so every seat that later dialled in was
/// answered `unknown workspace`, and no gesture could repair it. The box's own
/// leaves are seated by the founding itself.
#[test]
fn a_workspace_founded_in_world_is_seen_by_the_boxs_own_seats() {
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = provisioned(root.path(), data.path(), bin.path());
    assert_eq!(ctx.answer(&prepare("fleet"))["kind"], "prepared");

    for role in crate::wire::material::SEATS {
        let client = crate::registry::Client::parse(&role.common_name()).unwrap();
        assert!(
            crate::registry::registered(root.path(), &client).contains("fleet"),
            "{} sees what this box founded",
            role.common_name()
        );
    }
    // And it is a real read, not just a file: the seat carrying this box's own
    // client leaf enumerates the workspace it could not see before.
    let seated = operator(
        crate::registry::Client::parse(&crate::wire::material::Role::Client.common_name()).unwrap(),
    );
    assert_eq!(
        listed(&ctx.answer_as(&seated, &json!({"op": "workspaces"}))),
        ["fleet".to_owned()]
    );
    // §4 is otherwise untouched: a certificate this box did not mint sees
    // nothing until it is enrolled.
    assert!(listed(&ctx.answer_as(&seat("stranger"), &json!({"op": "workspaces"}))).is_empty());
}

/// **Founding only, never joining** (REMOTE §4): revocation is deleting the
/// file, so a gesture that merely names a standing workspace must not write it
/// back. The second prepare is the resume every start flow performs.
#[test]
fn a_gesture_over_a_standing_workspace_seats_nothing() {
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = provisioned(root.path(), data.path(), bin.path());
    assert_eq!(ctx.answer(&prepare("fleet"))["kind"], "prepared");
    let window = crate::registry::window();
    std::fs::remove_file(crate::registry::registrations(root.path(), &window).join("fleet"))
        .expect("the operator's own revocation");

    // The wall is on disk now, so this one resumes rather than founds.
    assert_eq!(ctx.answer(&prepare("fleet"))["kind"], "prepared");
    assert!(
        !crate::registry::registered(root.path(), &window).contains("fleet"),
        "a revocation stands until the operator writes it back"
    );
}

/// A box that minted nothing seats nothing — the present-only read, which is
/// what keeps a registration from naming a certificate that does not exist.
#[test]
fn a_box_with_no_material_seats_no_one() {
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    seed(data.path());
    let ctx = over_world(
        root.path(),
        world_of(data.path(), &[]),
        data.path().to_path_buf(),
        fake_litany(bin.path()),
        crate::test_support::world_under(data.path()),
    );
    assert_eq!(ctx.answer(&prepare("fleet"))["kind"], "prepared");
    assert!(
        !root.path().join(crate::registry::CLIENTS).exists(),
        "no leaf, no seat"
    );
}

/// **The doctor answers through the chokepoint** (bl-28f4), with a workspace
/// and without: the engine's own four always, its wall and its roster when one
/// is named. The gesture is the one workspace-addressed read that may name
/// none, because the box it exists for may hold none.
#[test]
fn the_doctor_answers_with_a_workspace_and_without_one() {
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = provisioned(root.path(), data.path(), bin.path());
    assert_eq!(ctx.answer(&prepare("fleet"))["kind"], "prepared");

    let bare = ctx.answer(&json!({"op": "doctor"}));
    assert_eq!(bare["kind"], "doctor", "{bare}");
    let checks: Vec<&str> = bare["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row["check"].as_str())
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(checks, ["wire", "address", "listener", "git"], "{bare}");

    let scoped = ctx.answer(&json!({"op": "doctor", "workspace": "fleet"}));
    let named: Vec<&str> = scoped["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row["check"].as_str())
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        named,
        ["wire", "address", "listener", "git", "wall", "clients"],
        "{scoped}"
    );
    // …and a workspace nothing enumerates is refused by the one resolution that
    // stands ahead of the table, exactly as any other named read is.
    let refusal = ctx.answer(&json!({"op": "doctor", "workspace": "gone"}));
    assert_eq!(refusal["ok"], false, "{refusal}");
}
