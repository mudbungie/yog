//! The judgment vocabulary: a ruling, a scope, and the answer they make.

use super::*;

#[test]
fn a_ruling_spells_itself_the_same_way_both_directions() {
    for ruling in [Ruling::Pass, Ruling::Hold, Ruling::Refuse] {
        assert_eq!(Ruling::of(ruling.word()), Some(ruling));
    }
    assert_eq!(Ruling::of("maybe"), None);
}

#[test]
fn a_scope_spells_itself_the_same_way_both_directions() {
    for scope in [Scope::Call, Scope::Conversation, Scope::Workspace] {
        assert_eq!(Scope::of(scope.word()), Some(scope));
    }
    assert_eq!(Scope::of("everywhere"), None);
    assert_eq!(Scope::of(""), None);
}

#[test]
fn an_answer_with_no_scope_stands_over_the_call() {
    assert_eq!(
        Answer::once(Ruling::Pass),
        Answer {
            ruling: Ruling::Pass,
            scope: Scope::Call
        }
    );
}

#[test]
fn a_scope_says_what_it_covers_and_never_offers_a_way_round() {
    assert_eq!(
        Scope::Call.stands_for("bash", Effect::OpenWorld),
        "this one call"
    );
    let conversation = Scope::Conversation.stands_for("beta2_read_log", Effect::Opaque);
    assert_eq!(
        conversation,
        "every beta2_read_log call classified opaque in this conversation and its descent"
    );
    let workspace = Scope::Workspace.stands_for("beta2_read_log", Effect::Opaque);
    assert!(workspace.ends_with("in this workspace"), "{workspace}");
    for said in [&conversation, &workspace] {
        assert!(!said.contains("instead"), "{said}");
        assert!(!said.contains("try"), "{said}");
    }
}

#[test]
fn loss_and_credentials_take_the_call_alone() {
    for effect in [Effect::Destructive, Effect::Secret] {
        assert_eq!(Scope::Call.permits(effect), Ok(()));
        for scope in [Scope::Conversation, Scope::Workspace] {
            let refused = scope
                .permits(effect)
                .expect_err("a class of calls, forever");
            assert!(refused.contains("--scope call only"), "{refused}");
            // The refusal names the remedy rather than only the wall.
            assert!(refused.contains("capability.yaml"), "{refused}");
        }
    }
}

#[test]
fn every_other_class_takes_every_scope() {
    for effect in [
        Effect::Read,
        Effect::TargetWrite,
        Effect::Process,
        Effect::OpenWorld,
        Effect::Opaque,
    ] {
        for scope in [Scope::Call, Scope::Conversation, Scope::Workspace] {
            assert_eq!(scope.permits(effect), Ok(()), "{effect:?} {scope:?}");
        }
    }
}

#[test]
fn a_class_key_is_the_tool_and_its_reach() {
    assert_eq!(class_key("bash", Effect::Read), "bash@read");
    // The reach is half the key: a tool released for one is not released for
    // another, and the token carries no space.
    assert_eq!(
        class_key("box2_Bash", Effect::TargetWrite),
        "box2_Bash@target-write"
    );
    assert_ne!(
        class_key("box2_Bash", Effect::Read),
        class_key("box2_Bash", Effect::Destructive)
    );
}

/// The §4.9 fifth rung over the fold this vocabulary is spoken into — its own
/// file, where main put it.
mod floor;
