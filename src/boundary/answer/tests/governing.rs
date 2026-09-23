//! **The governing read's answered arm** (bl-13f9; the mark beside it,
//! bl-b680), over a real-git workspace: the chokepoint joins the followed
//! commit with the §9.4 workflow mark read live off the agent's descent —
//! `None` for an unmarked conversation, the holder and commit for a marked
//! one — asserted at `answer` rather than at either body, because the arm IS
//! the claim. Its own file at §12's cap; the refusal arm is tabled in [`super`].

use super::{deps, ui};
use crate::boundary::Query;
use crate::boundary::answer::answer;
use crate::boundary::reply::Reply;
use crate::boundary::tests::snapshot;
use crate::git_tree::tests::fixture::Fixture;
use crate::git_tree::tests::git::run_git;
use crate::git_tree::{Agent, GitTree};

const ROOT: &str = "20260101T000000Z-r1";

fn agents(fx: &Fixture) -> Vec<Agent> {
    GitTree::from_repo(&fx.path).unwrap().agents
}

fn governing(fx: &Fixture) -> Reply {
    let d = deps(snapshot(
        &fx.path,
        &crate::naming::leaf(&fx.path),
        agents(fx),
        vec![],
    ));
    answer(
        &Query::Governing {
            workspace: crate::naming::leaf(&fx.path),
            agent: ROOT.to_owned(),
            at: None,
        },
        &d,
        &ui(),
        200,
    )
    .unwrap()
}

#[test]
fn the_governing_answer_carries_the_followed_commit_and_the_mark_beside_it() {
    let fx = Fixture::new();
    fx.agent_off(ROOT, "config/default");
    let Reply::Governing {
        config,
        workflow_mark,
    } = governing(&fx)
    else {
        panic!("the governing shape");
    };
    assert_eq!(config.followed_lineage().as_deref(), Some("default"));
    assert_eq!(workflow_mark, None, "an unmarked conversation");

    run_git(
        &fx.path.join("repo.git"),
        &[
            "update-ref",
            &format!("refs/litany/workflow/{ROOT}"),
            "config/default",
        ],
    );
    let Reply::Governing { workflow_mark, .. } = governing(&fx) else {
        panic!("the governing shape");
    };
    let mark = workflow_mark.expect("the mark just written");
    assert_eq!(mark.holder, ROOT);
    assert_eq!(mark.lineage.as_deref(), Some("default"));
}
