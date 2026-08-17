//! The composed one-pick plan (§9.4) and the sentences the surface paints.

use super::{TEMPLATE_PROVIDERS, table};
use crate::model_pick::grammar::{GrammarError, RoleModel, roles};
use crate::model_pick::{
    Pick, PickError, WORKER_ROLE, WRITE_NOTE, birth_sentence, default_row, plan, role_fault,
    scope_sentence,
};

fn pick(model: &str) -> Pick {
    Pick {
        role: WORKER_ROLE.to_string(),
        provider: "codex".to_string(),
        model: model.to_string(),
    }
}

/// brazen's table with the row every fixture pick names — the ordinary case.
fn rows() -> Vec<crate::config_edit::brazen::ProviderRow> {
    table(&["codex", "google"])
}

/// bl-d9cb. One pick, ONE write: the role's assignment in `providers.yaml`, and
/// nothing else. lernie retired the global `models:` table (its bl-35e2), so the
/// declaration that used to lead this gesture reached nothing that reads it.
#[test]
fn a_pick_writes_the_role_assignment_and_only_that() {
    let assigned = plan(TEMPLATE_PROVIDERS, &rows(), &pick("gpt-5.6-sol")).unwrap();
    assert_eq!(
        roles(&assigned)
            .into_iter()
            .find(|r| r.role == WORKER_ROLE)
            .map(|r| (r.provider, r.model)),
        Some(("codex".to_string(), "gpt-5.6-sol".to_string()))
    );
    // Nothing models.yaml-shaped comes back — the one text IS the one file.
    assert!(!assigned.contains("models:"), "{assigned}");
    assert!(!assigned.contains("context_window"), "{assigned}");
    // And the pick is total over ids the file never heard of: there is no
    // declaration to have made first, so nothing to be missing.
    assert!(plan(TEMPLATE_PROVIDERS, &rows(), &pick("my-local-tag")).is_ok());
}

/// The one file the grammar cannot read refuses the gesture, and the grammar's
/// own sentence rides through unchanged — the pick layer adds a refusal kind,
/// never a second phrasing of the file's.
#[test]
fn an_unreadable_providers_yaml_refuses_the_plan() {
    let refused = plan("roles: {}\n", &rows(), &pick("gpt-5.6-sol"));
    let expected = GrammarError::Inline {
        file: crate::model_pick::grammar::PROVIDERS_YAML,
        key: "roles".into(),
    };
    assert_eq!(refused, Err(PickError::Grammar(expected.clone())));
    assert_eq!(refused.unwrap_err().to_string(), expected.to_string());
}

/// bl-bd89. A pick on a provider row brazen's table does not have is refused
/// outright — not warned about. Writing it would land the exact config that
/// dies at the first dispatch with `unknown provider`.
#[test]
fn a_pick_on_a_row_brazen_lacks_is_refused_before_the_file_is_touched() {
    let live = table(&["openai-chatgpt"]);
    let refused = plan(TEMPLATE_PROVIDERS, &live, &pick("gpt-5.4"));
    assert_eq!(
        refused,
        Err(PickError::UnknownProvider {
            provider: "codex".to_string(),
        })
    );
    let said = refused.unwrap_err().to_string();
    assert!(said.contains("no provider row `codex`"), "{said}");
    assert!(said.contains("pick a live row here"), "{said}");
}

/// bl-bd89. The custom-id entry lets the operator type a model brazen does not
/// list, so `plan` is where an id that cannot be a block key is stopped —
/// blank, or carrying whitespace / `:` / `#`. A listed candidate is a string
/// brazen itself printed and never trips this.
#[test]
fn a_model_id_the_block_grammar_cannot_hold_is_refused() {
    for bad in ["", "   ", "gpt 5", "gpt:5", "gpt#5", "a\tb"] {
        let mut broken = pick(bad);
        broken.model = bad.to_string();
        assert_eq!(
            plan(TEMPLATE_PROVIDERS, &rows(), &broken),
            Err(PickError::NotAnId {
                model: bad.to_string(),
            }),
            "{bad:?} must not reach the file"
        );
    }
    let said = PickError::NotAnId {
        model: "gpt 5".into(),
    }
    .to_string();
    assert!(said.contains("not a plain model id"), "{said}");
    // One file is named now, not two (bl-d9cb).
    assert!(said.contains("providers.yaml"), "{said}");
    assert!(!said.contains("models.yaml"), "{said}");
}

/// An empty table is brazen unanswered, not brazen answering "none" — it gates
/// nothing, on the same terms as the §9.2 Apply gate.
#[test]
fn an_unanswerable_brazen_gates_no_pick() {
    assert!(plan(TEMPLATE_PROVIDERS, &[], &pick("gpt-5.4")).is_ok());
}

/// bl-bd89. The row the picker queries is the role's own while brazen has it,
/// and brazen's first row once brazen does not — a role stranded on a renamed
/// row is never left asking the dead row for its models.
///
/// bl-dd7f amends it: the substitution is **reported**, never silent. A picker
/// that swapped `openai-chatgpt` for `anthropic` without a word read as a
/// report of what the conversation ran on, which is the falsifying screenshot
/// of bl-9b52 — so the answer carries the row it left, and the note names both.
#[test]
fn the_default_row_leaves_a_row_brazen_dropped_and_says_so() {
    let live = vec!["openai-chatgpt".to_string(), "google".to_string()];
    let steered = default_row("codex", &live);
    assert_eq!(steered.row, "openai-chatgpt");
    assert_eq!(steered.stranded.as_deref(), Some("codex"));
    let note = steered.strand_note().expect("a strand is named");
    assert!(note.contains("codex"), "{note}");
    assert!(note.contains("openai-chatgpt"), "{note}");
    assert!(note.contains("config.toml"), "{note}");

    // A row brazen has is no strand, and neither is an unaskable table: both
    // stand on the role's own row, and neither has anything to report.
    for scoped in [default_row("google", &live), default_row("codex", &[])] {
        assert_eq!(scoped.stranded, None);
        assert_eq!(scoped.strand_note(), None);
    }
    assert_eq!(default_row("google", &live).row, "google");
    assert_eq!(default_row("codex", &[]).row, "codex");
}

/// The scope sentence must say all three things the operator would otherwise
/// get wrong: which branch, which blast radius, and that THIS conversation does
/// not move.
#[test]
fn the_scope_sentence_states_branch_workspace_and_the_frozen_conversation() {
    let s = scope_sentence("mauve-tapir", "default", "1a2b3c4d");
    assert!(s.contains("config/default"));
    assert!(s.contains("mauve-tapir workspace"));
    assert!(s.contains("NEXT conversation"));
    assert!(s.contains("frozen at 1a2b3c4d"));
}

/// The birth block's sentence carries the one admission the §11 block owes the
/// operator (bl-824e): the start-time pick is not scoped to the conversation —
/// it moves the workspace default, because lernie offers no other semantic.
#[test]
fn the_birth_sentence_admits_it_moves_the_workspace_default() {
    let s = birth_sentence("mauve-tapir", "default");
    assert!(s.contains("mauve-tapir workspace default"));
    assert!(s.contains("config/default"));
    assert!(s.contains("about to start"));
    // And it must NOT borrow the frozen-conversation clause, which names a
    // conversation the birth block does not have.
    assert!(!s.contains("frozen"));
}

/// bl-d9cb. The write note names the ONE file a click touches. It named two and
/// their order until the cross-check that justified the pair turned out to be
/// retired upstream, so the note has to be checked for what it no longer says.
#[test]
fn the_write_note_names_the_one_file_and_no_second_one() {
    assert!(WRITE_NOTE.contains("writes"));
    assert!(WRITE_NOTE.contains("providers.yaml"));
    assert!(WRITE_NOTE.contains("lernie config"));
    assert!(!WRITE_NOTE.contains("models.yaml"), "{WRITE_NOTE}");
    assert!(!WRITE_NOTE.contains("first"), "{WRITE_NOTE}");
}

/// bl-d9cb, amending bl-53be. A role row's fault is over the LIVE pointer —
/// `roles.<r>.provider` against brazen's table — in the pick gate's own words, so
/// the mark and the refusal one gesture later cannot phrase it differently.
///
/// The old judgement went through the global `models.yaml` and had the defect the
/// other way round: a role sitting on a row brazen had dropped was unmarked
/// whenever its model entry happened to name a live one.
#[test]
fn a_role_on_a_row_brazen_lacks_is_marked_in_the_pick_gates_words() {
    let stranded = RoleModel {
        role: WORKER_ROLE.to_string(),
        provider: "codex".to_string(),
        model: "gpt-5.4".to_string(),
    };
    let live = vec!["openai-chatgpt".to_string()];
    assert_eq!(
        role_fault(&live, &stranded),
        Some(
            PickError::UnknownProvider {
                provider: "codex".to_string(),
            }
            .to_string()
        )
    );
    // A live row is no fault, and an unanswerable table judges nothing — the
    // same rule `is_unknown_row` applies everywhere else.
    let mut alive = stranded.clone();
    alive.provider = "openai-chatgpt".to_string();
    assert_eq!(role_fault(&live, &alive), None);
    assert_eq!(role_fault(&[], &stranded), None);
}
