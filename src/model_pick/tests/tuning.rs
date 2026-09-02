//! The §9.4 tuning pair's pure half (bl-23bd): the two knobs' rewrites over the
//! role assignment `/model` writes, both directions of each, and the one gate
//! that stands ahead of them.
//!
//! Every case reads the template `providers.yaml` litany itself seeds
//! ([`TEMPLATE_PROVIDERS`]), which carries **no** `effort:` and no `priority:`
//! — so the set arm is exercised as an *insert*, which is the arm the grammar
//! could not do before this ball, and the clear arm is exercised against a file
//! that never had the line, which is the arm that must be a no-op rather than a
//! fault.

use super::TEMPLATE_PROVIDERS;
use crate::model_pick::{Effort, Tuning, tuning::plan};

/// The template with `worker` already tuned both ways — the other starting
/// state, where a set is a *replace* and a clear has something to take away.
fn tuned() -> String {
    TEMPLATE_PROVIDERS.replace(
        "    model: gpt-5.4\n    tools:",
        "    model: gpt-5.4\n    effort: low\n    priority: true\n    tools:",
    )
}

fn effort(role: &str, level: Option<Effort>) -> Tuning {
    Tuning::Effort {
        workspace: "ws".to_owned(),
        role: role.to_owned(),
        level,
    }
}

fn priority(role: &str, on: bool) -> Tuning {
    Tuning::Priority {
        workspace: "ws".to_owned(),
        role: role.to_owned(),
        on,
    }
}

/// **The insert arm.** A role with no `effort:` line gets one, and everything
/// else survives: the pointer, the `tools:` flow sequence, and the sibling role
/// the rewrite must not touch. This is the case `set_field` refused outright
/// before bl-23bd — the trap being that an optional field's first write is
/// always an insert.
#[test]
fn a_level_is_added_to_a_role_that_had_none_and_nothing_else_moves() {
    let out = plan(TEMPLATE_PROVIDERS, &effort("worker", Some(Effort::High))).expect("planned");
    assert!(out.contains("    effort: high\n"), "{out}");
    assert!(out.contains("    provider: codex\n"), "{out}");
    assert!(
        out.contains("    tools: [bash, read_file, load_skill]\n"),
        "{out}"
    );
    assert!(out.contains("  compactor:\n"), "{out}");
    // The line lands inside the role that asked for it, never in its sibling.
    let worker = out.split("  compactor:").next().expect("worker block");
    assert!(worker.contains("effort: high"), "{out}");
}

/// **The replace arm**, and the proof the insert did not double up: a role that
/// already carries the field gets its value moved, not a second line.
#[test]
fn a_level_on_a_role_that_had_one_is_moved_not_duplicated() {
    let out = plan(&tuned(), &effort("worker", Some(Effort::Medium))).expect("planned");
    assert_eq!(out.matches("effort:").count(), 1, "{out}");
    assert!(out.contains("    effort: medium\n"), "{out}");
    // The other knob is untouched by the one being written.
    assert!(out.contains("    priority: true\n"), "{out}");
}

/// **Off is the absent line.** Not `effort: off`, which the engine would read
/// as an unknown value, and not `effort: none`, which would be a second
/// spelling of an absence the engine already has one spelling for.
#[test]
fn off_removes_the_line_rather_than_writing_a_word_for_absence() {
    let out = plan(&tuned(), &effort("worker", None)).expect("planned");
    assert!(!out.contains("effort"), "{out}");
    assert!(out.contains("    priority: true\n"), "{out}");
    assert!(out.contains("    model: gpt-5.4\n"), "{out}");
}

/// Turning off what was never on is the same world — the idempotence
/// `remove_entry` already keeps, so a seat can send the gesture without first
/// reading the file to find out whether it needs to.
#[test]
fn clearing_a_knob_a_role_never_carried_is_the_file_unchanged() {
    let out = plan(TEMPLATE_PROVIDERS, &effort("worker", None)).expect("planned");
    assert_eq!(out, TEMPLATE_PROVIDERS);
    let out = plan(TEMPLATE_PROVIDERS, &priority("worker", false)).expect("planned");
    assert_eq!(out, TEMPLATE_PROVIDERS);
}

/// The priority knob is a checkbox: `on` writes the one value litany reads,
/// `off` takes the line away, and there is no third thing either can produce.
#[test]
fn priority_writes_true_or_nothing_at_all() {
    let on = plan(TEMPLATE_PROVIDERS, &priority("worker", true)).expect("planned");
    assert!(on.contains("    priority: true\n"), "{on}");
    let off = plan(&on, &priority("worker", false)).expect("planned");
    assert_eq!(off, TEMPLATE_PROVIDERS);
    assert!(!off.contains("priority"), "{off}");
}

/// Both knobs on one role coexist: writing the second does not disturb the
/// first, which is the whole reason this is a field-wise edit rather than the
/// whole-entry rewrite `set_entry` would have made of it.
#[test]
fn the_two_knobs_are_written_independently_and_both_survive() {
    let out = plan(TEMPLATE_PROVIDERS, &effort("worker", Some(Effort::Low))).expect("planned");
    let out = plan(&out, &priority("worker", true)).expect("planned");
    assert!(out.contains("    effort: low\n"), "{out}");
    assert!(out.contains("    priority: true\n"), "{out}");
    assert!(
        out.contains("    tools: [bash, read_file, load_skill]\n"),
        "{out}"
    );
}

/// **The entry is required and the field is not**, on both arms. A role the
/// file does not declare refuses — including on the *clear* path, which would
/// otherwise report success for a write that reached nothing, since removing an
/// absent field from an absent entry is indistinguishable from a no-op. Both
/// refusals are the grammar's own, so the two paths cannot be told apart.
#[test]
fn a_role_the_file_does_not_declare_refuses_on_both_arms() {
    let set =
        plan(TEMPLATE_PROVIDERS, &effort("nonesuch", Some(Effort::Low))).expect_err("no such role");
    assert!(set.to_string().contains("nonesuch"), "{set}");
    assert!(set.to_string().contains("providers.yaml"), "{set}");
    let clear = plan(TEMPLATE_PROVIDERS, &effort("nonesuch", None)).expect_err("no such role");
    assert_eq!(
        clear.to_string(),
        set.to_string(),
        "one sentence, both paths"
    );
    let toggle = plan(TEMPLATE_PROVIDERS, &priority("nonesuch", false)).expect_err("no such role");
    assert_eq!(toggle.to_string(), set.to_string());
}

/// A file with no `roles:` block at all declares no role, so every gesture
/// refuses by the same rule rather than creating the block — inventing a role
/// is the YAML transform this grammar never is. Both arms, because they reach
/// the missing block by different routes: the set arm through the replace
/// primitive's own prelude, the clear arm through the span lookup beside it.
#[test]
fn a_file_with_no_roles_block_declares_no_role() {
    for gesture in [
        priority("worker", true),
        priority("worker", false),
        effort("worker", Some(Effort::Low)),
        effort("worker", None),
    ] {
        let said = plan("models: {}\n", &gesture).expect_err("nothing declared");
        assert!(said.to_string().contains("worker"), "{said}");
        assert!(said.to_string().contains("providers.yaml"), "{said}");
    }
}

/// The level vocabulary round-trips through its own two halves, and nothing
/// else parses — `off` included, which is the caller's word for an absence and
/// deliberately not a fourth level.
#[test]
fn the_level_vocabulary_is_closed_and_round_trips() {
    for level in [Effort::Low, Effort::Medium, Effort::High] {
        assert_eq!(Effort::parse(&level.as_str()), Some(level));
    }
    for word in ["off", "none", "LOW", "highest", ""] {
        assert_eq!(Effort::parse(word), None, "{word}");
    }
}

/// The carrier answers its own address and its own subject, which is what lets
/// the §8.5 table delegate instead of matching on the pair.
#[test]
fn the_carrier_answers_the_workspace_and_the_role_for_both_members() {
    let mut one = effort("worker", Some(Effort::Low));
    let mut two = priority("compactor", true);
    assert_eq!(one.workspace_slot(), "ws");
    assert_eq!(two.workspace_slot(), "ws");
    "elsewhere".clone_into(two.workspace_slot());
    assert_eq!(two.workspace_slot(), "elsewhere");
    assert_eq!(one.role(), "worker");
    assert_eq!(two.role(), "compactor");
}

/// A blank line inside the entry's span is stepped over, not inserted after:
/// the new field joins the run of fields, and the file keeps the spacing its
/// author gave it. The arm the scan's `if` takes when a line carries nothing.
#[test]
fn a_blank_line_inside_the_entry_keeps_its_place_and_the_field_joins_the_run() {
    let spaced = "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\n\n  compactor:\n    \
                  provider: codex\n    model: gpt-5.4-mini\n";
    let out = plan(spaced, &effort("worker", Some(Effort::High))).expect("planned");
    assert_eq!(
        out,
        "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\n    effort: high\n\n  \
         compactor:\n    provider: codex\n    model: gpt-5.4-mini\n"
    );
}

/// An **inline** `roles:` key is the one shape this grammar never rewrites —
/// doing so would make it a YAML transform — and it refuses that way on every
/// arm: setting a knob, and clearing one. Both sentences are the grammar's own.
#[test]
fn an_inline_roles_key_refuses_on_every_arm() {
    let inline = "roles: {}\n";
    for gesture in [
        effort("worker", Some(Effort::Low)),
        effort("worker", None),
        priority("worker", true),
        priority("worker", false),
    ] {
        let said = plan(inline, &gesture).expect_err("inline is never rewritten");
        assert!(said.to_string().contains("roles"), "{said}");
    }
}
