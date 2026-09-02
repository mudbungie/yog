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
/// `litany` that founds one, and a snapshot nothing will ever re-derive.
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
        fake_litany(bin),
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
/// `litany prompt` needs the newborn to be both **registered** — which the
/// create's auto-registration writes — and **enumerated**, which is this fix.
/// One without the other still refuses.
#[test]
fn the_windows_posted_receipt_addresses_the_wall_it_just_raised() {
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = newborn_world(root.path(), data.path(), bin.path());
    let window = crate::registry::window();
    let seated = operator(window.clone());
    let born = ctx.answer_as(&seated, &prepare("home"));
    assert_eq!(born["kind"], "prepared", "{born}");
    assert!(
        crate::registry::registered(root.path(), &window).contains("home"),
        "the create seated its creator (REMOTE §4)"
    );
    let listed = ctx.answer_as(
        &seated,
        &json!({"op": "conversations", "workspace": "home"}),
    );
    assert_eq!(listed["kind"], "conversations", "{listed}");
    // Scope still decides, and it decides on registration rather than on what
    // disk holds: the very same wall is absent to a certificate nobody seated.
    let refusal = ctx.answer_as(
        &seat("stranger"),
        &json!({"op": "conversations", "workspace": "home"}),
    );
    assert_eq!(
        refusal["error"], "unknown workspace \"home\" — none is enumerated here",
        "{refusal}"
    );
}

/// **A birth that died mid-way is resumable, not a wedge** (bl-c9d2):
/// `litany new` makes the directory before it makes the `repo.git` marker, so
/// a killed birth leaves a marker-less directory no root enumerates. The
/// resolver used to refuse that name forever — `unknown workspace`, a
/// sentence about addressing for a half-written filesystem — with no in-band
/// exit. Now the raise resolves past debris that is not a workspace, and the
/// idempotent ensure's `litany new` finishes what the dead one started.
#[test]
fn a_half_born_directory_is_resumed_not_wedged() {
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
    let (root, data, bin) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let ctx = newborn_world(root.path(), data.path(), bin.path());
    let window = crate::registry::window();
    let seated = operator(window.clone());
    let born = ctx.answer_as(&seated, &prepare("home"));
    assert_eq!(born["kind"], "prepared", "{born}");
    let refusal = ctx.answer_as(&seat("stranger"), &prepare("home"));
    assert_eq!(
        refusal["error"], "unknown workspace \"home\" — none is enumerated here",
        "{refusal}"
    );
}

/// **Project birth is a barrier too** (bl-3377): bl-6c9e stated its ruling as a
/// rule about *existence* and then folded one set, so `yog bl prime` founded a
/// project the intake could not address until the next full sweep — and the
/// refusal was byte-identical to a typo. The clones dir is one readdir away,
/// exactly as the workspace roots are.
#[test]
fn a_gesture_addresses_a_project_primed_since_the_last_derivation() {
    let (root, data, world_root) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let world = crate::test_support::world_under(world_root.path());
    let ctx = over_world(
        root.path(),
        world_of(data.path(), &[]),
        data.path().to_path_buf(),
        Cli::new("/no/such/bl"),
        world.clone(),
    );
    let listing = json!({"op": "close", "project": "proj", "id": "bl-1", "name": "alba"});
    let before = ctx.answer(&listing);
    assert_eq!(before["ok"], false, "{before}");
    assert_eq!(
        before["error"], "unknown project \"proj\" — none is enumerated here",
        "the cached set is what the defect resolved over — and it says so"
    );

    // `bl prime` lays exactly this down: one percent-encoded clone dir under
    // the world's balls state. Nothing re-derives the snapshot afterwards,
    // which is the engine's state in the milliseconds after a prime.
    let clones = world.balls_clones_dir();
    std::fs::create_dir_all(clones.join("%2Fd%2Fproj")).expect("a primed project");

    let after = ctx.answer(&listing);
    assert_ne!(
        after["error"], before["error"],
        "the name the prime made addressable is addressable now: {after}"
    );
}
