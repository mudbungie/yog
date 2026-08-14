//! The two §9 reads that had no boundary spelling until bl-dff8: the §9.3
//! lineage browse (and a lineage destination's own bytes) and the §9.4 model
//! roster. Split from [`reads`](super::reads) at §12's per-file budget, on the
//! seam the reads themselves have — those are files under the world root, these
//! are the workspace's git and the wall's brazen.
//!
//! Both are driven through the same hermetic world and the same `answer`
//! chokepoint; nothing here reaches the network — the roster case names a
//! provider row the wall does not have, which brazen refuses at config
//! resolution, before a request exists.

use super::{ask, quiet};
use crate::boundary::Query;
use crate::boundary::config::ConfigFile;
use crate::boundary::config::read::roster;
use crate::boundary::reply::Reply;
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::brazen::BzOutcome;
use crate::git_tree::tests::fixture::Fixture;
use std::path::Path;
use tempfile::tempdir;

fn reading(file: ConfigFile) -> Query {
    Query::ReadConfig { file }
}

fn lineage_file(workspace: &Path, path: &str) -> ConfigFile {
    ConfigFile::Branch {
        workspace: crate::naming::leaf(workspace),
        lineage: "default".to_owned(),
        origin: EditOrigin::Advance,
        path: path.to_owned(),
    }
}

/// bl-dff8 — the §9.3 pane's Load, spelled: the bytes at the lineage tip, out
/// of the workspace's own bare repo. This is what an Apply on that destination
/// would be diffed against, so a headless seat now edits over what is there.
#[test]
fn a_lineage_read_answers_the_bytes_at_its_tip() {
    let root = tempdir().unwrap();
    let fx = Fixture::new();
    let deps = super::seeing(&quiet(root.path()), &[fx.path.as_path()]);
    // The fixture seeds `version` = "1\n" on the first config commit.
    assert_eq!(
        ask(&deps, &reading(lineage_file(&fx.path, "version"))),
        Ok(Reply::Config {
            text: "1\n".to_owned()
        })
    );
}

/// A path the lineage does not hold refuses in git's own words — never an
/// empty text, which an Apply would then commit over whatever is really there.
#[test]
fn a_path_the_lineage_does_not_hold_refuses_in_gits_words() {
    let root = tempdir().unwrap();
    let fx = Fixture::new();
    let deps = super::seeing(&quiet(root.path()), &[fx.path.as_path()]);
    let err = ask(&deps, &reading(lineage_file(&fx.path, "no-such-file"))).unwrap_err();
    assert!(err.contains("no-such-file"), "{err}");
}

/// The browse (bl-dff8): every lineage the workspace has, each with the files
/// its tip holds — one answer, so the listing and the trees are of one moment.
#[test]
fn the_browse_lists_every_lineage_with_its_own_files() {
    let root = tempdir().unwrap();
    let fx = Fixture::new();
    let deps = super::seeing(&quiet(root.path()), &[fx.path.as_path()]);
    fx.commit_other("workflow.yaml", "events: {}\n");
    fx.orphan_config("island");
    let Ok(Reply::Lineages(rows)) = ask(
        &deps,
        &Query::Lineages {
            workspace: crate::naming::leaf(&(fx.path.clone())),
        },
    ) else {
        panic!("lineages answers lineages");
    };
    let named = |name: &str| {
        rows.iter()
            .find(|r| r.branch.name == name)
            .unwrap_or_else(|| panic!("{name} is not listed: {rows:?}"))
            .clone()
    };
    let default = named("default");
    assert!(default.files.contains(&"workflow.yaml".to_owned()));
    assert_eq!(default.branch.tip_oid.len(), 40);
    assert_eq!(
        Some(default.branch.tip_short_oid.as_str()),
        default.branch.tip_oid.get(..8)
    );
    // A second lineage answers its own tree, not the first's.
    assert!(!named("island").files.contains(&"workflow.yaml".to_owned()));
}

/// A workspace with no repository to browse is said outright — the `work-diff`
/// rule: an unreadable repo shown as no lineages is a lie about the workspace.
#[test]
fn a_workspace_with_no_repo_refuses_the_browse_rather_than_listing_nothing() {
    let root = tempdir().unwrap();
    let nowhere = root.path().join("nowhere");
    let deps = super::seeing(&quiet(root.path()), &[nowhere.as_path()]);
    assert!(
        ask(
            &deps,
            &Query::Lineages {
                workspace: crate::naming::leaf(&nowhere),
            },
        )
        .is_err()
    );
}

/// bl-dff8 — the §9.4 roster, asked **in the named sphere's wall**: the row has
/// to be one that workspace's brazen has. An unknown row refuses in brazen's own
/// words, before a request is composed — which is what a headless picker needs
/// to hear instead of guessing a model id.
#[test]
fn a_roster_for_a_row_this_wall_does_not_have_refuses_by_name() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let err = ask(
        &deps,
        &Query::Models {
            workspace: crate::naming::leaf(&(crate::test_support::fixture_workspace())),
            provider: "not-a-row".to_owned(),
        },
    )
    .unwrap_err();
    assert!(err.contains("not-a-row"), "{err}");
}

/// The roster fold (§9.4), driven over canned runs — the picker's own settle,
/// as a boundary answer: ids in the provider's order, and every unusable
/// outcome a refusal that says why rather than an empty list.
#[test]
fn the_roster_fold_answers_ids_or_says_why_there_are_none() {
    let out = |success: bool, stdout: &str, stderr: &str| BzOutcome {
        success,
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
    };
    assert_eq!(
        roster(&out(
            true,
            "{\"models\":[{\"id\":\"m-9\"},{\"id\":\"m-1\"}]}",
            ""
        )),
        Ok(vec!["m-9".to_owned(), "m-1".to_owned()])
    );
    // Exit 0 with nothing listed is a fact about the provider, named as one.
    assert_eq!(
        roster(&out(true, "{\"models\":[]}", "")),
        Err(crate::model_pick::query::EMPTY_ROSTER.to_owned())
    );
    // A shapeless payload is the same answer — never a parse error a seat
    // would have to tell apart from an empty provider.
    assert_eq!(
        roster(&out(true, "not json at all", "")),
        Err(crate::model_pick::query::EMPTY_ROSTER.to_owned())
    );
    // A failed run answers brazen's own stderr…
    assert_eq!(
        roster(&out(false, "", "  unknown provider `x`\n")),
        Err("unknown provider `x`".to_owned())
    );
    // …and a failure that said nothing still says something.
    assert_eq!(
        roster(&out(false, "", "")),
        Err("bz --list-models failed".to_owned())
    );
}
