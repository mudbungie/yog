//! The read-only derived surfaces over a loaded editor (§5.1 rows 20/23): the
//! effective dump, the model cache, and the folded [`BrazenPaths`]. None of
//! these touch the Apply pipeline — that is [`super`]'s half. (The
//! credential-presence read that stood here went in bl-dba3: brazen's own
//! `credential` column is the fact, and a stat of the credentials directory was
//! a second derivation of it that could not see `ambient` or `inline`.)

use super::{FakeRunner, loaded};
use crate::config_edit::brazen::{BUILT_IN_ROWS_HINT, BrazenPaths};
use crate::config_edit::brazen::{BzRunner as _, row_views};
use crate::test_support::FakeFs;
use crate::xdg::Env;
use std::path::{Path, PathBuf};

#[test]
fn effective_runs_dump_config_against_real_env() {
    let ed = loaded(&FakeFs::default());
    let runner = FakeRunner::with(true, "merged config", "");
    let out = ed.effective(&runner);
    assert_eq!(out.stdout, "merged config");
    assert_eq!(*runner.log(), vec!["effective".to_string()]);
}

/// The rendered table is the runner's own listing, asked once — the whole of
/// what the boundary's `Providers` reply does (bl-dba3: it used to pair that
/// listing with a stat of the credentials directory, and the two answers about
/// one fact disagreed).
#[test]
fn the_rendered_table_is_the_listing_asked_once() {
    let runner = FakeRunner::listing(&["openai", "anthropic"]);
    let views = row_views(&runner.providers());
    assert_eq!(
        views.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
        ["openai", "anthropic"]
    );
    assert_eq!(views[0].fact, "auth none \u{b7} no credential needed");
    assert_eq!(*runner.log(), vec!["providers".to_string()]);
}

#[test]
fn model_cache_reads_present_and_absent() {
    let fs = FakeFs::seed(
        &PathBuf::from("/cache/models/openai.json"),
        b"[{\"id\":\"gpt\"}]",
    );
    let ed = loaded(&fs);
    assert_eq!(
        ed.model_cache("openai", &fs).unwrap().as_deref(),
        Some("[{\"id\":\"gpt\"}]")
    );
    assert_eq!(ed.model_cache("gemini", &fs).unwrap(), None);
}

#[test]
fn refresh_models_dispatches_list_models() {
    let ed = loaded(&FakeFs::default());
    let runner = FakeRunner::ok();
    let _ = ed.refresh_models("openai", &runner);
    assert_eq!(*runner.log(), vec!["list:openai".to_string()]);
}

#[test]
fn built_in_rows_hint_is_a_static_line() {
    assert!(BUILT_IN_ROWS_HINT.contains("built-in"));
}

#[test]
fn brazen_paths_fold_inside_the_wall_and_answer_none_without_one() {
    // No wall named: no config, no credentials, no cache — and no fallback to
    // the machine's own brazen state, however completely XDG is set (§16.2).
    let ambient = Env::from_pairs([
        ("HOME", "/home/u"),
        ("XDG_CONFIG_HOME", "/home/u/.config"),
        ("XDG_DATA_HOME", "/home/u/.local/share"),
        ("XDG_CACHE_HOME", "/home/u/.cache"),
    ]);
    assert_eq!(BrazenPaths::of(&ambient), None);
    // The three leaves under a wall root, one layout for every §10 target.
    let p = BrazenPaths::in_wall(Path::new("/w/walls/corp"));
    assert_eq!(p.config, PathBuf::from("/w/walls/corp/brazen/config.toml"));
    assert_eq!(
        p.credentials_dir,
        PathBuf::from("/w/walls/corp/brazen/credentials")
    );
    assert_eq!(
        p.models_cache_dir,
        PathBuf::from("/w/walls/corp/brazen/models")
    );
    // `of` is `in_wall` at the env's own wall — one fold, not two.
    let lensed = crate::world::wall::env(
        &crate::world::compose(&ambient),
        Path::new("/anywhere/corp"),
    );
    assert_eq!(
        BrazenPaths::of(&lensed),
        Some(BrazenPaths::in_wall(
            &lensed.wall().expect("the lens set a wall")
        ))
    );
}
