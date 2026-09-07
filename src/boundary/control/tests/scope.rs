//! **What an answer stands over** (bl-94a5): the row a widened answer writes,
//! the class it is keyed on, and the two refusals that keep a wide one from
//! being given where permanence is wrong.

use super::{AGENT, Answer, Answers, Effect, Ruling, Scope, Standing, World, answer_hold, request};
use crate::control::judge::class_key;
use crate::control::policy::Policy;
use crate::opslog::tail;

/// The sentence litany's mark carries for an opaque routed call — the shape
/// bl-b65d's eleven holds all wore.
const OPAQUE: &str = "beta2_read_log {\"path\":\"/var/log/syslog\"} classified opaque \
                      (beta2_read_log is not a tool this control implements)";

/// Answer the park in `world` at `scope`, and hand back the trail.
fn answer(world: &World, scope: Scope) -> Result<Vec<crate::opslog::OpEntry>, String> {
    answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        AGENT,
        Answer {
            ruling: Ruling::Pass,
            scope,
        },
    )?;
    Ok(tail(&world.state(), usize::MAX))
}

#[test]
fn a_conversation_answer_is_keyed_on_the_conversation_and_the_class() {
    let world = World::new();
    world.repo();
    world.park_as(AGENT, "toolu_1", "beta2_read_log", OPAQUE);
    let rows = answer(&world, Scope::Conversation).expect("something is parked");
    let class = class_key("beta2_read_log", Effect::Opaque);
    assert_eq!(
        rows[0].argv,
        vec![
            "yog-control".to_owned(),
            "answer".to_owned(),
            format!("{AGENT} {class}"),
            "pass".to_owned(),
            "conversation".to_owned(),
        ]
    );
    // And it stands: the *next* call of that class, with another id and
    // another path, is answered without the operator being asked again.
    assert_eq!(
        Answers::fold(&rows).ruling(
            &request("toolu_2", "beta2_read_log", AGENT),
            &crate::nav::ws_key(&world.workspace()),
            Effect::Opaque,
            &Policy::default(),
        ),
        Standing {
            ruling: Ruling::Pass,
            scope: Scope::Conversation
        }
    );
}

#[test]
fn a_workspace_answer_is_keyed_on_the_class_and_the_row_s_own_workspace() {
    let world = World::new();
    world.repo();
    world.park_as(AGENT, "toolu_1", "beta2_read_log", OPAQUE);
    let rows = answer(&world, Scope::Workspace).expect("something is parked");
    assert_eq!(
        rows[0].argv,
        vec![
            "yog-control",
            "answer",
            "beta2_read_log@opaque",
            "pass",
            "workspace"
        ]
    );
    assert_eq!(rows[0].cwd, crate::nav::ws_key(&world.workspace()));
    // A conversation that never held anything is covered by it.
    assert_eq!(
        Answers::fold(&rows)
            .ruling(
                &request("toolu_9", "beta2_read_log", "somebody-else"),
                &crate::nav::ws_key(&world.workspace()),
                Effect::Opaque,
                &Policy::default(),
            )
            .ruling,
        Ruling::Pass
    );
}

#[test]
fn loss_is_answered_for_the_call_in_front_of_you_and_never_for_a_class() {
    let world = World::new();
    world.repo();
    world.park_as(
        AGENT,
        "toolu_1",
        "alpha2_Bash",
        "alpha2_Bash {\"command\":\"rm -rf /srv\"} classified destructive (`rm` reaches /srv)",
    );
    for scope in [Scope::Conversation, Scope::Workspace] {
        let refused = answer(&world, scope).expect_err("a class of destructive calls, forever");
        assert!(refused.contains("--scope call only"), "{refused}");
        assert!(refused.contains("capability.yaml"), "{refused}");
    }
    // Nothing was written: a refused gesture is not a partial one.
    assert!(tail(&world.state(), usize::MAX).is_empty());
    // The call itself still answers.
    assert_eq!(answer(&world, Scope::Call).expect("the one call").len(), 2);
}

#[test]
fn a_mark_whose_class_cannot_be_read_takes_the_call_alone() {
    let world = World::new();
    world.repo();
    world.park_as(AGENT, "toolu_1", "bash", "held, and nothing more is said");
    let refused = answer(&world, Scope::Conversation).expect_err("no class, no class grant");
    assert!(refused.contains("does not say which class"), "{refused}");
    assert!(refused.contains("--scope call"), "{refused}");
    assert!(tail(&world.state(), usize::MAX).is_empty());
    // …and the narrow answer is unaffected, because it stands over an id.
    assert!(answer(&world, Scope::Call).is_ok());
}
