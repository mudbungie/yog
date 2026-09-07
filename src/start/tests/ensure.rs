//! The workspace executor (§4.2, Z5): the idempotent, **atomic** `litany new`
//! ensure and the §8.6/§3.7 policy convergence outside its create skip. The
//! name-shaped halves — the mint mapping and the worktree ladder — are
//! [`super::names`]'s since §12's cap split them off (bl-1af5); the `bl`-facing
//! executors are [`super::exec`]'s concern.

use super::{World, fake_fail, fake_litany};
use crate::binding::workspace_path;
use crate::cli_outbound::Cli;
use crate::opslog::{Origin, YOG_STEP};
use crate::start::{Deps, StartError, execute_ensure_workspace};
use crate::world::{Layout, layout_under};
use std::path::PathBuf;

/// The world layout anchored on this world's yog data root — where the §8.6
/// capability-control shim is resolved from.
fn layout(w: &World) -> Layout {
    layout_under(w.yog.path())
}

/// Start deps whose `litany` is the only binary these rungs reach.
fn deps(w: &World, litany: &Cli) -> Deps {
    Deps {
        bl: Cli::new("/no/bl"),
        litany: litany.clone(),
        state_root: w.state.path().to_path_buf(),
        yog_binary: PathBuf::from("/no/yog"),
    }
}

#[test]
fn ensure_skips_when_the_workspace_already_exists() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "n");
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();
    let litany = Cli::new("/definitely/not/a/real/litany");
    let created = execute_ensure_workspace(
        &deps(&w, &litany),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap();
    assert!(!created, "existing workspace skipped");
    assert!(w.ops().is_empty(), "skip runs and logs nothing");
}

#[test]
fn ensure_creates_the_workspace_and_logs() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    let litany = Cli::new(fake_litany(w.bin.path()));
    assert!(
        execute_ensure_workspace(
            &deps(&w, &litany),
            "TS",
            &ws,
            "default",
            &layout(&w),
            Origin::Balls
        )
        .unwrap()
    );
    assert!(ws.parent().unwrap().is_dir(), "parent chain mkdir -p'd");
    assert!(ws.join("repo.git").is_dir(), "the birth landed at the name");
    // The trail names the path litany was actually given — an I3 temp in the
    // workspace's own parent, renamed into place when litany finished (bl-1af5).
    let born = &w.ops()[0].argv[2];
    assert_eq!(
        std::path::Path::new(born).parent(),
        ws.parent(),
        "born beside its destination, so the landing is a same-dir rename: {born}"
    );
    assert!(
        crate::scratch::is_temp(
            &std::path::Path::new(born)
                .file_name()
                .unwrap()
                .to_string_lossy()
        ),
        "{born}"
    );
    assert!(
        !ws.parent().unwrap().join(born).exists(),
        "the temp is gone"
    );
}

/// **A birth is one act or none** (bl-1af5): a `litany new` that dies part way
/// used to leave `<workspace>/repo.git` behind, which IS a workspace by §3.1 —
/// so the name was enumerated, unaddressable to the client that failed to make
/// it, and unusable forever with no in-band exit. Nothing is left now, and the
/// very next attempt at the same name founds it.
#[test]
fn a_failed_birth_leaves_no_workspace_and_wedges_no_name() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "n");
    // A `litany new` that makes the directory and the marker, then dies — the
    // shape of every failure between the bare repo and the first commit.
    let half = super::write_exec(
        w.bin.path(),
        "litany",
        "#!/bin/sh\nnew_half() { mkdir -p \"$1/repo.git\"; }\n\
         case \"$1\" in new) new_half \"$2\"; exit 1 ;; esac\nexit 0\n",
    );
    let err = execute_ensure_workspace(
        &deps(&w, &Cli::new(half)),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap_err();
    assert!(matches!(err, StartError::VerbFailed { verb: "new", .. }));
    assert!(!ws.exists(), "the name is free: {}", ws.display());
    assert!(
        crate::binding::workspaces(w.yog.path(), w.yog.path()).is_empty(),
        "and nothing enumerates the debris"
    );
    // The same name founds on the very next attempt.
    let litany = Cli::new(fake_litany(w.bin.path()));
    assert!(
        execute_ensure_workspace(
            &deps(&w, &litany),
            "TS",
            &ws,
            "default",
            &layout(&w),
            Origin::Balls
        )
        .unwrap()
    );
    assert!(ws.join("repo.git").is_dir());
}

/// A rename that cannot land is a `["yog-step","birth"]` row and an `Io` error
/// (Z5), and it still leaves nothing: debris already at the destination that is
/// not an empty directory is `ENOTEMPTY`, where litany used to say `destination
/// is not empty`.
#[test]
fn a_birth_that_cannot_land_logs_its_step_and_leaves_nothing() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "n");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("squatter"), b"x").unwrap();
    let litany = Cli::new(fake_litany(w.bin.path()));
    let err = execute_ensure_workspace(
        &deps(&w, &litany),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap_err();
    assert!(matches!(err, StartError::Io(_)), "{err:?}");
    assert!(!ws.join("repo.git").exists(), "nothing landed");
    let step = w.ops().into_iter().find(|o| o.argv[0] == YOG_STEP);
    assert_eq!(step.expect("a birth step row").argv[1], "birth");
}

#[test]
fn ensure_errors_and_logs_on_a_nonzero_new() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "n");
    let litany = Cli::new(fake_fail(w.bin.path(), "litany", "disk full"));
    let err = execute_ensure_workspace(
        &deps(&w, &litany),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap_err();
    assert!(matches!(err, StartError::VerbFailed { verb: "new", .. }));
}

/// **A fresh box has no git identity** (bl-c28c): `litany new`'s first commit
/// dies with git's twelve-line *"Please tell me who you are"*, and that capture
/// used to cross the boundary verbatim as an `error` string with embedded
/// `\n`s, naming no prerequisite. It is one sentence now, and the ops row still
/// holds the capture for whoever wants it.
#[test]
fn a_new_that_died_for_want_of_a_git_identity_names_the_prerequisite() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "n");
    let litany = Cli::new(fake_fail(
        w.bin.path(),
        "litany",
        "Author identity unknown *** Please tell me who you are. Run            git config --global user.email you@example.com",
    ));
    let err = execute_ensure_workspace(
        &deps(&w, &litany),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap_err();
    let said = err.to_string();
    assert!(
        said.starts_with("git has no identity on this box"),
        "{said}"
    );
    assert!(!said.contains("Please tell me who you are"), "{said}");
    // The trail names the I3 temp litany was given (bl-1af5), beside its
    // destination — the capture is the row's, and the row is still there.
    let ops = w.ops();
    assert_eq!(ops[0].argv[1], "new");
    assert_eq!(std::path::Path::new(&ops[0].argv[2]).parent(), ws.parent());
}

#[test]
fn ensure_logs_a_mkdir_step_failure() {
    // The parent chain cannot be created (a file sits where a dir must go) → a
    // `["yog-step","mkdir"]` row before the Io error (§4.2, Z5), no `litany` spawn.
    let w = World::new();
    let blocker = w.yog.path().join("blocked");
    std::fs::write(&blocker, b"x").unwrap();
    let ws = blocker.join("workspaces").join("n");
    let litany = Cli::new("/definitely/not/a/real/litany");
    let err = execute_ensure_workspace(
        &deps(&w, &litany),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap_err();
    assert!(matches!(err, StartError::Io(_)));
    assert_eq!(w.ops()[0].argv, [YOG_STEP, "mkdir"]);
}

#[test]
fn ensure_creates_whatever_the_birth_template_names() {
    // bl-00ee: bl-c3a9 refused this exact fixture — a template naming a row
    // brazen's table lacks — and §16.2's wall made that refusal permanent, since
    // a newborn workspace's provider table is brazen's shipped rows and the
    // operator's sign-in only reaches it AFTER birth. Birth now judges nothing
    // about providers: the workspace is created, and a dead row is faulted in
    // the §9.5 pane and surfaced at the first dispatch (§8.3) instead.
    let w = World::new();
    let tmpl = layout(&w).litany.join("template");
    std::fs::create_dir_all(&tmpl).unwrap();
    std::fs::write(
        tmpl.join("providers.yaml"),
        "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\n",
    )
    .unwrap();
    let ws = workspace_path(w.yog.path(), "n");
    let litany = Cli::new(fake_litany(w.bin.path()));
    assert!(
        execute_ensure_workspace(
            &deps(&w, &litany),
            "TS",
            &ws,
            "default",
            &layout(&w),
            Origin::Balls
        )
        .unwrap()
    );
    assert!(
        !w.ops().iter().any(|e| e.argv == [YOG_STEP, "template"]),
        "no birth-time provider step remains"
    );
}
