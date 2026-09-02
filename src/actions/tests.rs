use super::*;
use crate::git_tree::{Agent, AgentState};
use crate::projects::join::JoinState;

mod conversation;

fn branch(name: &str, state: AgentState) -> Agent {
    Agent {
        branch_name: format!("agents/{name}"),
        agent_id: name.to_string(),
        tip_oid: "0".repeat(40),
        tip_short_oid: "0000000".to_string(),
        tip_timestamp_unix: 0,
        last_action_unix: 0,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state,
        state_uncertain: false,
        truncated: false,
        failure: None,
        flagged: None,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: None,
        goal_name: None,
        call_start_unix: None,
    }
}

/// bl-9acf: the one reading of "blank" every goal-fire site shares. A goal made
/// of whitespace is nothing said — firing it spends the wire on the identity
/// preamble alone — and a rung whose prefill is blank has no draft to open.
#[test]
fn a_blank_goal_is_not_a_goal_anywhere() {
    assert!(goal_present("fix the gate"));
    assert!(goal_present("  surrounded by spaces  "));
    // §3.4's bare rung composes exactly this prefill (pinned in
    // `start::tests::goal`), which is why the raise opens no draft.
    assert!(!goal_present(""));
    assert!(!goal_present("   \t\n"));
}

#[test]
fn new_prompt_enabled_when_text_present() {
    assert!(new_prompt_enabled("hi", ""));
    assert!(new_prompt_enabled("  surrounded by spaces  ", ""));
}

#[test]
fn new_prompt_disabled_for_empty_input() {
    assert!(!new_prompt_enabled("", ""));
}

#[test]
fn new_prompt_disabled_for_whitespace_only() {
    assert!(!new_prompt_enabled("   \t\n", ""));
}

/// bl-6191: something to say is not enough — Enter also needs somewhere lawful
/// to say it, so the start never reaches a fork that would misname the fault.
#[test]
fn new_prompt_disabled_when_the_work_directory_does_not_exist() {
    let dir = tempfile::tempdir().unwrap();
    assert!(new_prompt_enabled("hi", &dir.path().display().to_string()));
    assert!(!new_prompt_enabled(
        "hi",
        &dir.path().join("nope").display().to_string()
    ));
}

/// The field's own sentence (bl-6191): the refusal names the directory, an
/// existing one says nothing — and so does an empty box, which is the bare rung
/// rather than a bad path.
#[test]
fn work_dir_refusal_names_a_directory_that_is_not_there() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(work_dir_refusal(""), None);
    assert_eq!(work_dir_refusal("   "), None);
    assert_eq!(work_dir_refusal(&dir.path().display().to_string()), None);
    let missing = dir.path().join("nonexistent-uat-dir");
    assert_eq!(
        work_dir_refusal(&format!("  {}  ", missing.display())),
        Some(format!(
            "work directory does not exist: {}",
            missing.display()
        ))
    );
    // A file is refused by the same one question — but never told it is absent.
    let file = dir.path().join("a-file");
    std::fs::write(&file, b"x").unwrap();
    assert_eq!(
        work_dir_refusal(&file.display().to_string()),
        Some(format!(
            "work directory is not a directory: {}",
            file.display()
        ))
    );
}

#[test]
fn create_ball_enabled_requires_a_nonblank_title() {
    assert!(create_ball_enabled("Fix the bug"));
    assert!(!create_ball_enabled(""));
    assert!(!create_ball_enabled("  \t "));
}

#[test]
fn new_ball_hints_name_both_boxes() {
    // bl-b2ed: the form's two dark boxes are indistinguishable empty. Each
    // carries a hint, and the body's says what to write, not just its name —
    // the composer's own idiom ("say what you want done"), not a bare noun.
    let hints = crate::actions::new_ball_hints();
    assert_eq!(hints.title, "title");
    assert_eq!(hints.body, "body — what done looks like");
}

#[test]
fn draft_clears_only_on_a_clean_send() {
    use crate::actions::verbs::Outcome;
    let clean = Ok(Outcome {
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
    });
    let ran_nonzero = Ok(Outcome {
        exit: 3,
        stdout: String::new(),
        stderr: "boom".into(),
    });
    let never_launched: std::io::Result<Outcome> = Err(std::io::Error::other("no binary"));
    assert!(draft_clears(&clean), "a clean send clears the draft");
    assert!(
        !draft_clears(&ran_nonzero),
        "a ran-but-failed verb keeps the draft"
    );
    assert!(
        !draft_clears(&never_launched),
        "a spawn failure keeps the draft"
    );
}

#[test]
fn close_enabled_only_for_bound() {
    assert!(close_enabled(JoinState::Bound));
    assert!(!close_enabled(JoinState::ClaimedElsewhere));
    assert!(!close_enabled(JoinState::ReadyStartable));
    assert!(!close_enabled(JoinState::Delivered));
}

#[test]
fn unclaim_enabled_only_for_bound() {
    assert!(unclaim_enabled(JoinState::Bound));
    assert!(!unclaim_enabled(JoinState::ClaimedElsewhere));
    assert!(!unclaim_enabled(JoinState::ReadyStartable));
    assert!(!unclaim_enabled(JoinState::UnassignedWorkspace));
}

#[test]
fn assign_enabled_only_for_a_ready_ball() {
    // Assign binds an unbound ball → only ReadyStartable (what `bl claim` allows).
    assert!(assign_enabled(JoinState::ReadyStartable));
    assert!(!assign_enabled(JoinState::Bound), "already bound");
    assert!(!assign_enabled(JoinState::Blocked));
    assert!(!assign_enabled(JoinState::ClaimedElsewhere));
    assert!(!assign_enabled(JoinState::Delivered));
    assert!(!assign_enabled(JoinState::UnassignedWorkspace));
    assert!(!assign_enabled(JoinState::OrphanedProject));
}

#[test]
fn actions_state_default_is_empty() {
    let s = ActionsState::default();
    assert!(s.drafts.is_empty());
    assert!(s.path_dir.is_empty());
    assert!(s.selected_branch.is_none());
    assert!(!s.stop_children);
}

#[test]
fn actions_state_clone_eq() {
    let s = ActionsState {
        drafts: {
            let mut d = Drafts::default();
            d.set(DraftKey::Message("b".to_string()), "hi".to_string());
            d
        },
        selected_branch: Some("b".to_string()),
        stop_children: true,
        ..Default::default()
    };
    let s2 = s.clone();
    assert_eq!(s, s2);
}
