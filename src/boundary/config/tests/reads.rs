//! The §9 config family's reads, driven through the chokepoint (§8.5,
//! bl-0164): a destination's bytes, the §16.3 knob and brazen's provider
//! table — [`files`](super::files)/[`knobs`](super::knobs)'s read-only twin,
//! against the same hermetic world.

use super::{ACME, applying, ask, brazen_file, fire, quiet};
use crate::boundary::Query;
use crate::boundary::config::ConfigFile;
use crate::boundary::config::Read;
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::git_tree::tests::fixture::Fixture;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn reading(file: ConfigFile) -> Query {
    Query::Config(Read::File { file })
}

#[test]
fn a_brazen_read_answers_the_bytes_on_disk() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    assert_eq!(
        ask(&deps, &reading(brazen_file())),
        Ok(Reply::Config {
            text: ACME.to_owned()
        })
    );
}

/// bl-fcd5 — **the gesture's workspace is the wall, not the seat's.** The same
/// `Deps`, asked about two spheres, answers each sphere's own file and table:
/// the fixture's workspace has the `acme` row, a second workspace has nothing
/// at all. That is the whole fix — a headless seat has no focus to fall back
/// on, so if the named workspace were discarded a teleoperator could reach no
/// provider config anywhere.
#[test]
fn a_config_gesture_reads_the_sphere_it_names_not_the_seats_own() {
    let root = tempdir().unwrap();
    let deps = super::seeing(&quiet(root.path()), &[Path::new("/other-sphere")]);
    let elsewhere = |file| Query::Config(Read::File { file });
    assert_eq!(
        ask(&deps, &reading(brazen_file())),
        Ok(Reply::Config {
            text: ACME.to_owned()
        })
    );
    let other = ConfigFile::Brazen {
        workspace: crate::naming::leaf(&(PathBuf::from("/other-sphere"))),
    };
    assert_eq!(
        ask(&deps, &elsewhere(other)),
        Ok(Reply::Config {
            text: String::new()
        }),
        "another workspace's wall is another workspace's file"
    );
    // And the table follows the same name: rows are read inside the sphere the
    // query points at, so the row this workspace authored is absent from the
    // other's — which lists brazen's compiled-in rows and nothing else, never
    // the machine's own brazen state (§16.2).
    let named = |ws: &str| match ask(
        &deps,
        &Query::Config(Read::Providers {
            workspace: crate::naming::leaf(&(PathBuf::from(ws))),
        }),
    ) {
        Ok(Reply::Providers(rows)) => rows.into_iter().map(|r| r.name).collect::<Vec<_>>(),
        other => panic!("providers answers providers: {other:?}"),
    };
    assert!(named("/ws").iter().any(|n| n == "acme"));
    assert!(!named("/other-sphere").iter().any(|n| n == "acme"));
}

/// The ambient wall a seat happens to carry never wins: a `Deps` whose world
/// has no `YOG_WALL` at all still answers the named sphere in full, because
/// the executor derives the wall from the gesture rather than reading one out
/// of the environment (bl-fcd5).
#[test]
fn a_seat_with_no_wall_of_its_own_still_reaches_the_named_sphere() {
    let root = tempdir().unwrap();
    let deps = Deps {
        world: crate::test_support::no_wall(root.path()),
        ..quiet(root.path())
    };
    assert_eq!(
        ask(&deps, &reading(brazen_file())),
        Ok(Reply::Config {
            text: ACME.to_owned()
        })
    );
    let Ok(Reply::Providers(rows)) = ask(
        &deps,
        &Query::Config(Read::Providers {
            workspace: crate::naming::leaf(&(crate::test_support::fixture_workspace())),
        }),
    ) else {
        panic!("providers answers providers");
    };
    assert!(rows.iter().any(|r| r.name == "acme"), "{rows:?}");
    assert!(fire(&deps, &applying(brazen_file(), ACME)).is_ok());
}

#[test]
fn a_destination_not_there_yet_reads_as_empty_not_a_refusal() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    for file in [
        ConfigFile::LitanyModels,
        ConfigFile::LitanyWorkflow {
            name: "fresh".to_owned(),
        },
        ConfigFile::Cadence,
    ] {
        assert_eq!(
            ask(&deps, &reading(file)),
            Ok(Reply::Config {
                text: String::new()
            })
        );
    }
}

/// The editors' Reload, headless: what [`applying`] just landed is exactly
/// what [`reading`] answers back — one pipeline's write, this one's read.
#[test]
fn a_round_trip_apply_then_read_returns_the_same_bytes() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let text = "models:\n  m-1:\n    provider: acme\n";
    assert!(fire(&deps, &applying(ConfigFile::LitanyModels, text)).is_ok());
    assert_eq!(
        ask(&deps, &reading(ConfigFile::LitanyModels)),
        Ok(Reply::Config {
            text: text.to_owned()
        })
    );
}

#[test]
fn a_bad_workflow_name_refuses_before_any_read() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let file = ConfigFile::LitanyWorkflow {
        name: "../escape".to_owned(),
    };
    assert!(ask(&deps, &reading(file)).is_err());
}

#[test]
fn providers_reads_brazens_effective_table() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let Ok(Reply::Providers(rows)) = ask(
        &deps,
        &Query::Config(Read::Providers {
            workspace: crate::naming::leaf(&(crate::test_support::fixture_workspace())),
        }),
    ) else {
        panic!("providers answers providers");
    };
    let acme = rows.iter().find(|r| r.name == "acme").expect("acme row");
    assert_eq!(acme.blocked.as_deref(), Some("keyless — nothing to log in"));
}

#[test]
fn marks_read_answers_the_branch_and_the_space_it_is_a_branch_of() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let ws = root.path().join("workspaces").join("corp");
    let deps = super::seeing(&deps, &[ws.as_path()]);
    let reply = ask(
        &deps,
        &Query::Config(Read::Marks {
            workspace: crate::naming::leaf(&ws),
        }),
    );
    assert_eq!(
        reply,
        Ok(Reply::Marks {
            branch: crate::world::marks::SHARED_BRANCH.to_owned(),
        })
    );
}

/// bl-2410. The assignments read answers what the §9.4 gestures set, out of the
/// commit they stage against — so a control opens on what is in force. The
/// fixture's `worker` is tuned and its `compactor` is not, which is both arms of
/// each optional knob in one read.
#[test]
fn the_roles_read_answers_the_lineages_own_assignments() {
    let root = tempdir().unwrap();
    let fx = Fixture::new();
    fx.commit_other(
        "providers.yaml",
        "roles:\n  worker:\n    provider: acme\n    model: m-9\n    effort: low\n    \
         priority: true\n  compactor:\n    provider: acme\n    model: m-1\n",
    );
    let deps = super::seeing(&quiet(root.path()), &[fx.path.as_path()]);
    let rows = match ask(
        &deps,
        &Query::Config(Read::Roles {
            workspace: crate::naming::leaf(&fx.path),
        }),
    ) {
        Ok(Reply::Roles(rows)) => rows,
        other => panic!("roles answers roles: {other:?}"),
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].role, "worker");
    assert_eq!(rows[0].provider, "acme");
    assert_eq!(rows[0].model, "m-9");
    assert_eq!(rows[0].effort.as_deref(), Some("low"));
    assert!(rows[0].priority);
    assert_eq!(rows[1].role, "compactor");
    assert_eq!(rows[1].effort, None);
    assert!(!rows[1].priority);
}

/// **Nothing set is an answer.** A workspace with no readable config lineage has
/// assigned no role, and that is the honest thing to show a control opening on a
/// fresh world — unlike the §11 `governing` read, which refuses, because *this
/// conversation has no policy* is never true while this is an ordinary state.
#[test]
fn a_workspace_with_no_readable_lineage_declares_no_roles() {
    let root = tempdir().unwrap();
    // Known to the boundary — an unknown name is refused by the address table
    // long before the executor, which is a different answer entirely — but
    // holding no config lineage to read.
    let ws = root.path().join("workspaces").join("fresh");
    std::fs::create_dir_all(&ws).unwrap();
    let deps = super::seeing(&quiet(root.path()), &[ws.as_path()]);
    assert_eq!(
        ask(
            &deps,
            &Query::Config(Read::Roles {
                workspace: crate::naming::leaf(&ws),
            }),
        ),
        Ok(Reply::Roles(vec![])),
    );
}
