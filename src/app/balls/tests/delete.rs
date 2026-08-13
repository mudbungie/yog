//! What the §3.6 unmaking releases: the join's **bound** rows for the workspace,
//! and nothing else. Shares this module's cloned-project world.

use super::{model, set_closed, world};
use crate::delete::{Step, plan};

#[test]
fn the_confirmation_names_the_bound_balls_and_the_plan_releases_them() {
    let w = world();
    // A delivered ball claimed by the same workspace: its dead-name claimant is
    // the obituary record (§3.4), so the unmaking must leave it alone.
    set_closed(
        &w,
        r#"[{"id":"bl-done","title":"Done","claimant":"cobalt"}]"#,
    );
    let (_c, mut m) = model(&w);
    m.after_bl_verb(&w.project);

    let confirm = m.delete_confirmation(&w.ws_cobalt).unwrap();
    assert_eq!(confirm.ball_ids(), ["bl-work"], "live bound balls only");
    assert!(
        m.delete_confirmation(&w.ws_spare)
            .unwrap()
            .ball_ids()
            .is_empty(),
        "a workspace no ball claims releases nothing"
    );
    // Stamped with the workspace's own name — the claimant releases its own ball
    // (§3.2's ownership line, §8.2's rider).
    assert_eq!(
        plan(&confirm, std::path::Path::new("/w"))[0],
        Step::Release {
            project: w.project.clone(),
            id: "bl-work".to_owned(),
            name: "cobalt".to_owned(),
        }
    );
}
