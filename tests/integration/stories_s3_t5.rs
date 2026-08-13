//! STORIES **S3-T5** empty-project-hint: with **zero projects** in the world, the
//! roster view-model carries the paved-interim hint `yog exec bl prime` (v1 keeps
//! `bl prime` out of the UI, §8.3; the empty-project roster section renders the
//! hint, owned by Z4). No project ⇒ no ball UI at all — the §3.5 unassigned
//! workspace is the general case (STORIES S3 pothole, DESIGN §8.3, §15 M6 Z4).

#![allow(clippy::unwrap_used)]

use crate::support::FakeBl;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// STORIES **S3-T5** empty-project-hint.
#[test]
fn s3_t5_zero_projects_carries_the_yog_exec_bl_prime_hint() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        // The clones root is absent ⇒ `projects::enumerate` finds nothing ⇒ zero
        // projects; the injected `bl` is never consulted.
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    let (m, _worker) = AppModel::boot(
        roots,
        None,
        Arc::new(SystemClock),
        Box::new(FakeBl {
            live: HashMap::new(),
            ..FakeBl::default()
        }),
        None,
    );
    // bl-b491: the command is its own hint line, so the roster can render it
    // whole instead of eliding it inside a sentence.
    assert_eq!(
        m.empty_project_hint().map(|h| h.command),
        Some("yog exec bl prime".to_owned()),
        "the empty-project roster carries the paved interim",
    );
}
