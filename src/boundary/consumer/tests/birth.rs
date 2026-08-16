//! **Workspace birth is a barrier** (bl-6c9e): a reply that founds a workspace
//! makes its name addressable to the call after it, at both intakes.
//!
//! The defect these reproduce is not a slow derivation — it is a *cache* read
//! where the authority was one readdir away. The cell in every test here holds
//! the empty snapshot the context was built over and no worker ever runs, which
//! is exactly the engine's state in the milliseconds after a birth: the wall is
//! on disk, and the derivation has not read it.

use super::*;
use serde_json::json;
use tempfile::tempdir;

/// The context the drive reproduced on: a world with no workspace at all, a
/// `lernie` that founds one, and a snapshot nothing will ever re-derive.
fn newborn_world(
    root: &std::path::Path,
    data: &std::path::Path,
    bin: &std::path::Path,
) -> ConsumerCtx {
    seed(data);
    over(
        root,
        world_of(data, &[]),
        data.to_path_buf(),
        fake_lernie(bin),
    )
}

/// The bare-rung prepare envelope, aimed by name.
fn prepare(ws: &str) -> serde_json::Value {
    json!({"op": "prepare", "workspace": ws, "payload": {"rung": "bare"}})
}

/// **The drive's own reproduction** (bl-6c9e): `/prepare` founds `home` and
/// answers; the immediate second gesture used to earn `unknown workspace
/// "home"` — twice for another prepare, and for every gesture that cannot found
/// anything, which is the half no `Prepare` fallback could ever have covered.
#[test]
fn a_second_gesture_addresses_the_wall_the_first_founded() {
    let _g = crate::test_support::spawn_guard();
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = newborn_world(root.path(), data.path(), bin.path());
    let born = ctx.answer(&prepare("home"));
    assert_eq!(born["kind"], "prepared", "{born}");
    assert!(
        crate::binding::workspace_path(data.path(), "home")
            .join("repo.git")
            .is_dir(),
        "the raise founded the wall the reply named"
    );
    // The second prepare — resume is the same path as opening (§8.1), so it is a
    // prepared reply and not a refusal.
    let again = ctx.answer(&prepare("home"));
    assert_eq!(again["kind"], "prepared", "{again}");
    // …and a read, which founds nothing: the case the resolver's raise arm
    // never reached.
    let listed = ctx.answer(&json!({"op": "conversations", "workspace": "home"}));
    assert_eq!(listed["ok"], true, "{listed}");
    assert_eq!(listed["kind"], "conversations", "{listed}");
}

/// The same composition for the window (REMOTE §4.1, §9.8): it posts every act
/// over the wire as `yog-window`, so its own Prepare receipt reaching
/// `lernie prompt` needs the newborn to be both **registered** — which the
/// create's auto-registration writes — and **enumerated**, which is this fix.
/// One without the other still refuses.
#[test]
fn the_windows_posted_receipt_addresses_the_wall_it_just_raised() {
    let _g = crate::test_support::spawn_guard();
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = newborn_world(root.path(), data.path(), bin.path());
    let window = crate::registry::window();
    let born = ctx.answer_as(&window, &prepare("home"));
    assert_eq!(born["kind"], "prepared", "{born}");
    assert!(
        crate::registry::registered(root.path(), &window).contains("home"),
        "the create seated its creator (REMOTE §4)"
    );
    let listed = ctx.answer_as(
        &window,
        &json!({"op": "conversations", "workspace": "home"}),
    );
    assert_eq!(listed["kind"], "conversations", "{listed}");
    // Scope still decides, and it decides on registration rather than on what
    // disk holds: the very same wall is absent to a certificate nobody seated.
    let refusal = ctx.answer_as(
        &client("stranger"),
        &json!({"op": "conversations", "workspace": "home"}),
    );
    assert_eq!(refusal["error"], "unknown workspace \"home\"", "{refusal}");
}

/// **A birth that died mid-way is resumable, not a wedge** (bl-c9d2):
/// `lernie new` makes the directory before it makes the `repo.git` marker, so
/// a killed birth leaves a marker-less directory no root enumerates. The
/// resolver used to refuse that name forever — `unknown workspace`, a
/// sentence about addressing for a half-written filesystem — with no in-band
/// exit. Now the raise resolves past debris that is not a workspace, and the
/// idempotent ensure's `lernie new` finishes what the dead one started.
#[test]
fn a_half_born_directory_is_resumed_not_wedged() {
    let _g = crate::test_support::spawn_guard();
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = newborn_world(root.path(), data.path(), bin.path());
    // The debris: the directory exists, the marker does not.
    let debris = crate::binding::workspace_path(data.path(), "home");
    std::fs::create_dir_all(&debris).unwrap();
    let born = ctx.answer(&prepare("home"));
    assert_eq!(born["kind"], "prepared", "{born}");
    assert!(
        debris.join("repo.git").is_dir(),
        "the resume finished the dead birth"
    );
}

/// The scope half stands (REMOTE §4): a directory that IS a workspace and is
/// not in the caller's scope still refuses with the resolver's sentence — the
/// raise can found or resume, never join another client's wall.
#[test]
fn a_scoped_clients_prepare_never_joins_anothers_wall() {
    let _g = crate::test_support::spawn_guard();
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = newborn_world(root.path(), data.path(), bin.path());
    let window = crate::registry::window();
    let born = ctx.answer_as(&window, &prepare("home"));
    assert_eq!(born["kind"], "prepared", "{born}");
    let refusal = ctx.answer_as(&client("stranger"), &prepare("home"));
    assert_eq!(refusal["error"], "unknown workspace \"home\"", "{refusal}");
}
