//! The doctor's rows: what each check reads, and that a failing one always
//! names an act.

use super::*;
use crate::boundary::dispatch::{Caller, Deps};
use crate::cli_outbound::Cli;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// A box with a world, a state root and a HOME of its own — every check here
/// reads one of those three and nothing ambient.
fn deps(tmp: &TempDir, caller: Caller) -> Deps {
    let world = crate::test_support::world_under(tmp.path());
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: tmp.path().join("state-root"),
        home: tmp.path().join("home"),
        yog_data_root: tmp.path().join("data"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world,
        snapshot: Arc::new(crate::app::Snapshot::empty(0)),
        caller,
    }
}

/// The row for `check`, owned — the house rule is borrow in, own out, and a
/// clone of four short strings is not worth a lifetime on the signature.
fn row(rows: &[Row], check: &str) -> Row {
    rows.iter()
        .find(|row| row.check == check)
        .unwrap_or_else(|| panic!("no {check} row in {rows:?}"))
        .clone()
}

/// **The state the install lane found a stranger in** (bl-28f4): a box with no
/// material, no endpoint, no listener and no identity. Every row fails, and
/// every failing row names the act — which is the whole of what this gesture
/// adds, since each of those facts was already derived somewhere and none of
/// them was ever asked for together.
#[test]
fn an_unwired_box_fails_every_check_and_names_an_act_for_each() {
    let tmp = tempdir().expect("tmp");
    let deps = deps(&tmp, Caller::default());
    let rows = examine(&deps, None);

    assert_eq!(rows.len(), 4, "the engine's own four: {rows:?}");
    for row in &rows {
        assert!(!row.ok, "{row:?}");
        let remedy = row.remedy.as_deref().unwrap_or_default();
        assert!(!remedy.is_empty(), "a failing row names an act: {row:?}");
    }
    assert!(row(&rows, "wire").fact.contains("no wire material"));
    assert!(
        row(&rows, "address")
            .remedy
            .as_deref()
            .unwrap_or_default()
            .contains("WIRE_PORT")
    );
    assert!(row(&rows, "git").fact.contains("no git identity"));
}

/// **A provisioned box reads back what it holds**, and the `:0` a boot writes
/// for itself is still a failing row — it is the one address nothing
/// downstream can use, which is exactly what this gesture exists to say before
/// a seat discovers it.
#[test]
fn a_self_provisioned_box_passes_the_material_and_fails_the_endpoint() {
    let tmp = tempdir().expect("tmp");
    let deps = deps(&tmp, Caller::default());
    let dir = crate::wire::material::dir(&deps.world);
    crate::wire::provision::ensure(&dir).expect("the boot's own mint");

    let rows = examine(&deps, None);
    assert!(row(&rows, "wire").ok, "{rows:?}");
    assert!(!row(&rows, "address").ok, "a `:0` is a request");
    assert!(row(&rows, "address").fact.contains("127.0.0.1:0"));

    // …and a stated endpoint passes, which is bl-98ef's act read back.
    crate::wire::provision::state(&dir, "127.0.0.1:7737").expect("stated");
    assert!(row(&examine(&deps, None), "address").ok);
}

/// **What the listener bound is the fact the file cannot give** (REMOTE §8):
/// `address` holds a request, and only the bind learns what a `:0` became. The
/// boot says it once on stderr; this is the same sentence, asked for.
#[test]
fn the_listener_row_answers_what_this_process_bound() {
    let tmp = tempdir().expect("tmp");
    let listening = crate::wire::Listening::default();
    let deps = deps(
        &tmp,
        Caller {
            listening: listening.clone(),
            ..Caller::default()
        },
    );
    assert!(!row(&examine(&deps, None), "listener").ok, "nothing bound");
    listening.state("127.0.0.1:39969");
    let bound = examine(&deps, None);
    assert!(row(&bound, "listener").ok);
    assert!(row(&bound, "listener").fact.contains("127.0.0.1:39969"));
}

/// A box whose git identity is set passes that row, read out of the world's own
/// HOME rather than the ambient one.
#[test]
fn the_git_row_reads_this_worlds_own_identity() {
    let tmp = tempdir().expect("tmp");
    let deps = deps(&tmp, Caller::default());
    std::fs::create_dir_all(&deps.home).expect("home");
    std::fs::write(
        deps.home.join(".gitconfig"),
        "[user]\n\tname = A Name\n\temail = someone@example.com\n",
    )
    .expect("gitconfig");
    let row = row(&examine(&deps, None), "git");
    assert!(row.ok, "{row:?}");
    assert!(row.fact.contains("A Name"), "{row:?}");
}

/// **Half a wire is a fault and says which files are missing** — the read's own
/// three answers, and the one an operator meets after a mint that died
/// mid-way.
#[test]
fn a_half_provisioned_directory_fails_the_wire_row_in_materials_words() {
    let tmp = tempdir().expect("tmp");
    let deps = deps(&tmp, Caller::default());
    let dir = crate::wire::material::dir(&deps.world);
    crate::wire::provision::ensure(&dir).expect("the boot's own mint");
    std::fs::remove_file(dir.join("server.key")).expect("half a leaf");

    let row = row(&examine(&deps, None), "wire");
    assert!(!row.ok, "{row:?}");
    assert!(row.fact.contains("half-provisioned"), "{row:?}");
    assert!(
        row.remedy.unwrap_or_default().contains("FORCE=1"),
        "a rotation is the only act that heals it"
    );
}

/// **A wall that can start a conversation passes**, and the row says so in the
/// workspace's own name — the gate's `Ok` arm, which is every box past its
/// first sign-in.
#[test]
fn a_signed_in_wall_passes_its_row() {
    let tmp = tempdir().expect("tmp");
    let mut deps = deps(&tmp, Caller::default());
    deps.world = crate::test_support::signed(&deps.world);
    let ws = tmp.path().join("names").join("alba");
    let row = row(&examine(&deps, Some(("alba", &ws))), "wall");
    assert!(row.ok, "{row:?}");
    assert!(row.fact.contains("alba"), "{row:?}");
    assert!(
        row.remedy.is_none(),
        "nothing to do about a wall that works"
    );
}

/// **Naming a workspace adds the two that are a workspace's**, and the wall row
/// is the `Prompt` door's own gate — the doctor quotes the sentence the fire
/// would refuse with rather than judging a wall a second time.
#[test]
fn a_named_workspace_adds_its_wall_and_its_roster() {
    let tmp = tempdir().expect("tmp");
    let deps = deps(&tmp, Caller::default());
    let ws = tmp.path().join("names").join("alba");
    let rows = examine(&deps, Some(("alba", &ws)));

    assert_eq!(rows.len(), 6, "four engine rows and two workspace ones");
    assert!(row(&rows, "clients").ok, "informational, and it passes");
    assert!(row(&rows, "clients").fact.contains("0 registered"));
    // The wall row's verdict depends on what brazen answers on this box: an
    // unanswerable table refuses nothing (the gate's own contract), so the beat
    // asserts the row is THERE and names the workspace either way.
    let wall = row(&rows, "wall");
    assert!(
        wall.fact.contains("alba"),
        "the wall row names the workspace: {wall:?}"
    );
    assert_eq!(
        wall.ok,
        wall.remedy.is_none(),
        "a remedy exactly when it failed"
    );
}
