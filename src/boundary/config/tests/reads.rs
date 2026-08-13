//! The §9 config family's reads, driven through the chokepoint (§8.5,
//! bl-0164): a destination's bytes, the §16.3 knob and brazen's provider
//! table — [`files`](super::files)/[`knobs`](super::knobs)'s read-only twin,
//! against the same hermetic world.

use super::{ACME, applying, ask, brazen_file, fire, quiet};
use crate::boundary::Query;
use crate::boundary::config::ConfigFile;
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::config_edit::branch::edit::EditOrigin;
use std::path::PathBuf;
use tempfile::tempdir;

fn reading(file: ConfigFile) -> Query {
    Query::ReadConfig { file }
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
    let deps = quiet(root.path());
    let elsewhere = |file| Query::ReadConfig { file };
    assert_eq!(
        ask(&deps, &reading(brazen_file())),
        Ok(Reply::Config {
            text: ACME.to_owned()
        })
    );
    let other = ConfigFile::Brazen {
        workspace: PathBuf::from("/other-sphere"),
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
        &Query::Providers {
            workspace: PathBuf::from(ws),
        },
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
        &Query::Providers {
            workspace: crate::test_support::fixture_workspace(),
        },
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
        ConfigFile::LernieModels,
        ConfigFile::LernieWorkflow {
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
    assert!(fire(&deps, &applying(ConfigFile::LernieModels, text)).is_ok());
    assert_eq!(
        ask(&deps, &reading(ConfigFile::LernieModels)),
        Ok(Reply::Config {
            text: text.to_owned()
        })
    );
}

#[test]
fn a_lineage_destination_refuses_the_boundary_read() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let file = ConfigFile::Branch {
        workspace: root.path().to_path_buf(),
        lineage: "default".to_owned(),
        origin: EditOrigin::Advance,
        path: "providers.yaml".to_owned(),
    };
    let err = ask(&deps, &reading(file)).unwrap_err();
    assert!(err.contains("config pane's own browse"), "{err}");
}

#[test]
fn a_bad_workflow_name_refuses_before_any_read() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let file = ConfigFile::LernieWorkflow {
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
        &Query::Providers {
            workspace: crate::test_support::fixture_workspace(),
        },
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
    let reply = ask(
        &deps,
        &Query::Marks {
            workspace: ws.clone(),
        },
    );
    assert_eq!(
        reply,
        Ok(Reply::Marks {
            branch: crate::world::marks::SHARED_BRANCH.to_owned(),
            space: crate::world::marks::read(&deps.world, &ws).state,
        })
    );
}
