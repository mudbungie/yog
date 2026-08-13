//! The Config tab's three governing-config arms: absent, frozen past a branch
//! tip, and sitting on one.

use super::{empty_tab_data, paint};
use crate::config_edit::branch::{GoverningConfig, governing_config};
use crate::files_view::FilesView;
use crate::git_tree::GitTree;
use crate::git_tree::tests::fixture::Fixture;
use crate::keymap::InspectorTab;
use crate::steps_view::StepsView;
use crate::transcript::Transcript;

#[test]
fn config_tab_without_governing_shows_a_note() {
    let (transcript, steps, inbox) = (Transcript::default(), StepsView::default(), Vec::new());
    let files = FilesView::default();
    let data = empty_tab_data(transcript, steps, inbox, files, None);
    assert!(
        paint(InspectorTab::Config, &data).contains("no governing config"),
        "absent governing note missing"
    );
}

#[test]
fn config_tab_frozen_past_a_branch_tip_omits_the_tip_line() {
    // Fork the agent off config/default, then advance the lineage: the
    // governing commit is now an ancestor, not any branch tip.
    let fx = Fixture::new();
    fx.build_agent("c-1", "hi");
    let forked_tip = {
        let tree = GitTree::from_repo(&fx.path).unwrap();
        tree.agents
            .iter()
            .find(|a| a.agent_id == "c-1")
            .unwrap()
            .tip_oid
            .clone()
    };
    fx.commit_other("providers.yaml", "advanced\n");
    let governing = governing_config(&fx.path, &forked_tip).unwrap();
    assert!(
        governing.branch_name_if_tip_of_one.is_none(),
        "governing should be frozen past the tip"
    );
    let (transcript, steps, inbox) = (Transcript::default(), StepsView::default(), Vec::new());
    let files = FilesView::default();
    let data = empty_tab_data(transcript, steps, inbox, files, Some(governing));
    let cfg = paint(InspectorTab::Config, &data);
    assert!(cfg.contains("policy frozen at"), "config:\n{cfg}");
    assert!(
        !cfg.contains("tip of config/"),
        "no tip line expected:\n{cfg}"
    );
}

#[test]
fn config_tab_at_a_branch_tip_shows_the_tip_line() {
    let governing = GoverningConfig {
        oid: "a".repeat(40),
        short_oid: "aaaaaaaa".into(),
        branch_name_if_tip_of_one: Some("default".into()),
        files: vec!["version".into()],
    };
    let (transcript, steps, inbox) = (Transcript::default(), StepsView::default(), Vec::new());
    let files = FilesView::default();
    let data = empty_tab_data(transcript, steps, inbox, files, Some(governing));
    let cfg = paint(InspectorTab::Config, &data);
    assert!(cfg.contains("tip of config/default"), "config:\n{cfg}");
}
