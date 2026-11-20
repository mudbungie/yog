//! **A role's row says which machine its provider's name selects** (REMOTE
//! §5.7, bl-471d), driven through the chokepoint against the real registry.

use super::{ask, quiet};
use crate::boundary::Query;
use crate::boundary::config::Read;
use crate::boundary::reply::{ConfigAnswer, Reply};
use crate::git_tree::tests::fixture::Fixture;
use tempfile::tempdir;

/// **A role on a registered machine's row says so** (REMOTE §5.7, bl-471d):
/// the reading the `bz` arm makes of the name — the longest registered client
/// that fits — beside the name, with nobody awake to list the row. A suffix
/// no machine registered here bears is the server's own row, whole.
#[test]
fn the_roles_read_names_the_machine_a_qualified_provider_selects() {
    let root = tempdir().unwrap();
    let fx = Fixture::new();
    fx.commit_other(
        "providers.yaml",
        "roles:\n  worker:\n    provider: sub@lab@annex\n    model: m-9\n  compactor:\n    provider: team@corp\n    model: m-1\n",
    );
    let deps = super::seeing(&quiet(root.path()), &[fx.path.as_path()]);
    let workspace = crate::naming::leaf(&fx.path);
    for client in ["annex", "lab@annex"] {
        crate::test_support::foot::seat(&deps.state_root, client, &workspace, &[]);
    }
    let Ok(Reply::Config(ConfigAnswer::Roles(rows))) =
        ask(&deps, &Query::Config(Read::Roles { workspace }))
    else {
        panic!("roles answers roles");
    };
    assert_eq!(rows[0].provider, "sub@lab@annex");
    assert_eq!(
        rows[0].at,
        Some(crate::boundary::reply::Place {
            client: "lab@annex".to_owned(),
            row: "sub".to_owned(),
        })
    );
    assert_eq!(rows[1].at, None);
}
