//! **What a wide answer stands over** (bl-94a5): the class of the held call,
//! over a conversation's descent or over a workspace — and what a floor does
//! to both.

use super::{at, call, row, ruling};
use crate::control::classify::Effect;
use crate::control::judge::{Answers, Ruling, Scope, Standing};
use crate::opslog::YOG_CONTROL;

#[test]
fn a_conversation_answer_stands_for_the_class_over_the_whole_descent() {
    let answers = Answers::fold(&[row(&[
        YOG_CONTROL,
        "answer",
        "amber beta2_read_log@opaque",
        "pass",
        "conversation",
    ])]);
    for agent in ["amber", "amber-1", "amber-1-2"] {
        assert_eq!(
            ruling(
                &answers,
                &call("toolu_x", "beta2_read_log", agent),
                "w",
                Effect::Opaque
            ),
            Standing {
                ruling: Ruling::Pass,
                scope: Scope::Conversation
            },
            "{agent}"
        );
    }
    // Another tool, another reach, another conversation: none of them.
    for (tool, effect, agent) in [
        ("beta2_service_status", Effect::Opaque, "amber"),
        ("beta2_read_log", Effect::Destructive, "amber"),
        ("beta2_read_log", Effect::Opaque, "amberine"),
        ("beta2_read_log", Effect::Opaque, "other"),
    ] {
        assert_eq!(
            ruling(&answers, &call("toolu_x", tool, agent), "w", effect).scope,
            Scope::Workspace,
            "{tool} {effect:?} {agent}"
        );
    }
}

#[test]
fn the_nearest_conversation_answer_wins_over_a_looser_ancestor() {
    let answers = Answers::fold(&[
        row(&[
            YOG_CONTROL,
            "answer",
            "amber bash@open-world",
            "pass",
            "conversation",
        ]),
        row(&[
            YOG_CONTROL,
            "answer",
            "amber-1 bash@open-world",
            "refuse",
            "conversation",
        ]),
    ]);
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "amber-1-9"),
            "w",
            Effect::OpenWorld
        )
        .ruling,
        Ruling::Refuse,
        "the more specific statement is the one aimed here"
    );
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "amber-2"),
            "w",
            Effect::OpenWorld
        )
        .ruling,
        Ruling::Pass
    );
}

#[test]
fn a_workspace_answer_is_the_workspace_the_row_was_written_in() {
    let answers = Answers::fold(&[at(
        "ws-a",
        &[
            YOG_CONTROL,
            "answer",
            "beta2_read_log@opaque",
            "pass",
            "workspace",
        ],
    )]);
    assert_eq!(
        ruling(
            &answers,
            &call("t", "beta2_read_log", "anyone"),
            "ws-a",
            Effect::Opaque
        ),
        Standing {
            ruling: Ruling::Pass,
            scope: Scope::Workspace
        }
    );
    assert_eq!(
        ruling(
            &answers,
            &call("t", "beta2_read_log", "anyone"),
            "ws-b",
            Effect::Opaque
        )
        .ruling,
        Ruling::Hold,
        "another workspace never saw the answer"
    );
}

#[test]
fn a_conversation_answer_outranks_the_workspace_s() {
    let answers = Answers::fold(&[
        at(
            "ws",
            &[
                YOG_CONTROL,
                "answer",
                "bash@open-world",
                "pass",
                "workspace",
            ],
        ),
        at(
            "ws",
            &[
                YOG_CONTROL,
                "answer",
                "amber bash@open-world",
                "refuse",
                "conversation",
            ],
        ),
    ]);
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "amber"),
            "ws",
            Effect::OpenWorld
        )
        .scope,
        Scope::Conversation
    );
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "other"),
            "ws",
            Effect::OpenWorld
        )
        .scope,
        Scope::Workspace
    );
}

#[test]
fn a_raised_floor_suspends_every_standing_answer() {
    // …which is also how a standing answer is revoked: the floor parks the
    // next call of the class, and the operator answers it again at the same
    // scope.
    let answers = Answers::fold(&[
        at(
            "ws",
            &[
                YOG_CONTROL,
                "answer",
                "bash@open-world",
                "pass",
                "workspace",
            ],
        ),
        row(&[
            YOG_CONTROL,
            "answer",
            "amber bash@open-world",
            "pass",
            "conversation",
        ]),
        row(&[YOG_CONTROL, "floor", "amber", "raise"]),
    ]);
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "amber"),
            "ws",
            Effect::OpenWorld
        ),
        Standing {
            ruling: Ruling::Hold,
            scope: Scope::Conversation
        }
    );
    // Lower it and both stand again.
    let answers = Answers::fold(&[
        row(&[
            YOG_CONTROL,
            "answer",
            "amber bash@open-world",
            "pass",
            "conversation",
        ]),
        row(&[YOG_CONTROL, "floor", "amber", "raise"]),
        row(&[YOG_CONTROL, "floor", "amber", "lower"]),
    ]);
    assert!(!answers.floored("amber"));
    assert_eq!(
        ruling(
            &answers,
            &call("t", "bash", "amber"),
            "ws",
            Effect::OpenWorld
        )
        .scope,
        Scope::Conversation
    );
}
