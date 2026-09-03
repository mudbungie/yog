//! **The typed half of a config read** (§9.5, bl-dc3f): `Query::ReadConfig`
//! answers the file's own schema applied to the very text it returns, so one
//! answer serves the raw editor and the controls pane and the file stays the
//! single fact.
//!
//! Its own file beside [`reads`](super::reads) on the seam the answer itself
//! has — those cases are about which bytes come back, these about what those
//! bytes are read AS.

use super::{applying, ask, brazen_file, fire, quiet, seeing};
use crate::boundary::Query;
use crate::boundary::config::{ConfigFile, Read};
use crate::boundary::reply::{ConfigView, Reply};
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::form::Control;
use crate::git_tree::tests::fixture::Fixture;
use crate::test_support::TEMPLATE_PROVIDERS;
use std::path::Path;
use tempfile::tempdir;

fn reading(file: ConfigFile) -> Query {
    Query::Config(Read::File { file })
}

fn answered(deps: &crate::boundary::dispatch::Deps, file: ConfigFile) -> ConfigView {
    match ask(deps, &reading(file)) {
        Ok(Reply::Config(view)) => view,
        other => panic!("a config read answers a config: {other:?}"),
    }
}

/// yog's own clock file: three bounded numbers, each carrying the bounds the
/// worker parses by — so a seat judges at input against `app::cadence`'s consts
/// rather than a second copy of them.
#[test]
fn a_typed_file_answers_its_settings_beside_its_text() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let template = crate::app::cadence::TEMPLATE;
    assert!(fire(&deps, &applying(ConfigFile::Cadence, template)).is_ok());

    let ConfigView { text, settings } = answered(&deps, ConfigFile::Cadence);
    assert_eq!(text, template, "the raw view is untouched");
    let names: Vec<&str> = settings.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["debounce_ms", "cheap_sweep_ms", "full_sweep_ms"]);
    assert!(settings.iter().all(|s| s.entry == "watcher"));
    assert!(settings.iter().all(|s| s.fault.is_none()));
    assert_eq!(
        settings.iter().map(|s| s.control).collect::<Vec<Control>>(),
        vec![
            Control::Number {
                min: crate::app::cadence::DEBOUNCE_BOUNDS.0,
                max: crate::app::cadence::DEBOUNCE_BOUNDS.1,
            },
            Control::Number {
                min: crate::app::cadence::CHEAP_SWEEP_BOUNDS.0,
                max: crate::app::cadence::CHEAP_SWEEP_BOUNDS.1,
            },
            Control::Number {
                min: crate::app::cadence::FULL_SWEEP_BOUNDS.0,
                max: crate::app::cadence::FULL_SWEEP_BOUNDS.1,
            },
        ],
        "the bounds are the worker's own, answered"
    );
}

/// §9.5's raw-text fallback, as the general path with empty input: brazen's
/// `config.toml` is a shape yog declines to interpret, so it answers its bytes
/// and an **empty** settings list — never an absent field, and never a form
/// over a shape yog guessed at.
#[test]
fn a_file_yog_has_no_grammar_for_answers_no_settings_and_all_its_text() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let view = answered(&deps, brazen_file());
    assert_eq!(view.text, super::ACME);
    assert!(view.settings.is_empty(), "{view:?}");

    // And a lineage path with no table is the same answer, not a second rule.
    let fx = Fixture::new();
    let deps = seeing(&quiet(root.path()), &[fx.path.as_path()]);
    let view = answered(
        &deps,
        ConfigFile::Branch {
            workspace: crate::naming::leaf(&fx.path),
            lineage: "default".to_owned(),
            origin: EditOrigin::Advance,
            path: "version".to_owned(),
        },
    );
    assert!(view.settings.is_empty(), "{view:?}");
}

/// The one judgement that is not a fact of the text: a role bound to a provider
/// row brazen's table does not carry is **faulted on the wire**, in the words
/// `grammar::is_unknown_row` gives it — the §9.4 pick gate's own judgement, so
/// the typed view and the gate cannot disagree.
#[test]
fn a_role_bound_to_a_dead_provider_row_is_faulted_in_the_answer() {
    let root = tempdir().unwrap();
    let fx = Fixture::new();
    fx.commit_other(
        crate::model_pick::PROVIDERS,
        &TEMPLATE_PROVIDERS.replace("provider: anthropic", "provider: gone"),
    );
    let deps = seeing(&quiet(root.path()), &[fx.path.as_path()]);
    let settings = answered(&deps, roles_file(&fx.path)).settings;

    let provider = settings
        .iter()
        .find(|s| s.entry == "worker" && s.name == "provider")
        .expect("the worker's provider row");
    assert_eq!(provider.control, Control::Provider);
    assert!(
        provider
            .fault
            .as_deref()
            .is_some_and(|f| f.contains("`gone`")),
        "{provider:?}"
    );
    assert!(
        settings
            .iter()
            .any(|s| s.name == "tools" && s.control == Control::List),
        "and the role's other controls cross with it: {settings:?}"
    );
}

fn roles_file(workspace: &Path) -> ConfigFile {
    ConfigFile::Branch {
        workspace: crate::naming::leaf(workspace),
        lineage: "default".to_owned(),
        origin: EditOrigin::Advance,
        path: crate::model_pick::PROVIDERS.to_owned(),
    }
}
