//! The composed one-pick plan (§9.4) and the sentences the surface paints.

use super::{SEEDED_MODELS, TEMPLATE_PROVIDERS};
use crate::model_pick::grammar::{GrammarError, MODELS_YAML, roles};
use crate::model_pick::{
    Pick, PickError, WORKER_ROLE, WRITE_NOTE, birth_sentence, default_row, plan, scope_sentence,
};

fn pick(model: &str) -> Pick {
    Pick {
        role: WORKER_ROLE.to_string(),
        provider: "codex".to_string(),
        model: model.to_string(),
    }
}

/// brazen's table with the row every fixture pick names — the ordinary case.
fn rows() -> Vec<String> {
    vec!["codex".to_string(), "google".to_string()]
}

/// A model the provider offers but `models.yaml` has never heard of produces
/// BOTH writes — the whole point of §9.4.
#[test]
fn a_model_unknown_to_models_yaml_plans_both_writes() {
    let planned = plan(
        SEEDED_MODELS,
        TEMPLATE_PROVIDERS,
        &rows(),
        &pick("gpt-5.6-sol"),
        None,
    )
    .unwrap();
    let declared = planned.models_yaml.expect("the id needed declaring");
    assert!(declared.contains("  gpt-5.6-sol:\n    provider: codex\n"));
    assert_eq!(
        roles(&planned.providers_yaml)
            .into_iter()
            .find(|r| r.role == WORKER_ROLE)
            .map(|r| r.model),
        Some("gpt-5.6-sol".to_string())
    );
}

/// bl-848f. The window the roster served rides the plan into the declaration,
/// so the denominator of §5.1 #35's fullness figure starts out TRUE for a
/// provider that publishes one — the same gesture, one guess fewer.
#[test]
fn a_served_window_reaches_the_declaration_the_plan_writes() {
    let planned = plan(
        SEEDED_MODELS,
        TEMPLATE_PROVIDERS,
        &rows(),
        &pick("gemini-3-pro"),
        Some(1_048_576),
    )
    .unwrap();
    let declared = planned.models_yaml.expect("the id needed declaring");
    assert!(
        declared.contains("    context_window: 1048576"),
        "{declared}"
    );
    assert_eq!(
        crate::model_pick::grammar::context_windows(&declared).get("gemini-3-pro"),
        Some(&1_048_576)
    );
    // The half that carries no window is untouched by the seed.
    assert!(planned.providers_yaml.contains("    model: gemini-3-pro"));
}

/// An already-declared id needs no models.yaml write — the operator's own
/// capabilities and context window stand.
#[test]
fn an_already_declared_model_plans_one_write() {
    let planned = plan(
        SEEDED_MODELS,
        &crate::model_pick::grammar::set_role_model(
            TEMPLATE_PROVIDERS,
            WORKER_ROLE,
            "codex",
            "gpt-5.6-sol",
        )
        .unwrap(),
        &rows(),
        &pick("gpt-5.4"),
        None,
    )
    .unwrap();
    assert_eq!(planned.models_yaml, None);
    assert!(planned.providers_yaml.contains("    model: gpt-5.4"));
}

/// The models.yaml half is decided first, so an unreadable one refuses the
/// whole gesture before anything is staged — there is no half-written state.
#[test]
fn an_unreadable_models_yaml_refuses_before_the_providers_half() {
    let refused = plan(
        "models: {}\n",
        TEMPLATE_PROVIDERS,
        &rows(),
        &pick("gpt-5.6-sol"),
        None,
    );
    assert_eq!(
        refused,
        Err(PickError::Grammar(GrammarError::Inline {
            file: MODELS_YAML,
            key: "models".into(),
        }))
    );
    // The grammar's own sentence rides through unchanged — the pick layer adds
    // a refusal kind, never a second phrasing of the file's.
    assert_eq!(
        refused.unwrap_err().to_string(),
        GrammarError::Inline {
            file: MODELS_YAML,
            key: "models".into(),
        }
        .to_string()
    );
}

#[test]
fn an_unreadable_providers_yaml_refuses_the_plan() {
    assert!(matches!(
        plan(
            SEEDED_MODELS,
            "roles: {}\n",
            &rows(),
            &pick("gpt-5.6-sol"),
            None
        ),
        Err(PickError::Grammar(GrammarError::Inline { .. }))
    ));
}

/// bl-bd89. A pick on a provider row brazen's table does not have is refused
/// outright — not warned about. Writing it would land the exact config that
/// dies at the first dispatch with `unknown provider`.
#[test]
fn a_pick_on_a_row_brazen_lacks_is_refused_before_either_file() {
    let live = vec!["openai-chatgpt".to_string()];
    let refused = plan(
        SEEDED_MODELS,
        TEMPLATE_PROVIDERS,
        &live,
        &pick("gpt-5.4"),
        None,
    );
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
            plan(SEEDED_MODELS, TEMPLATE_PROVIDERS, &rows(), &broken, None),
            Err(PickError::NotAnId {
                model: bad.to_string(),
            }),
            "{bad:?} must not reach either file"
        );
    }
    let said = PickError::NotAnId {
        model: "gpt 5".into(),
    }
    .to_string();
    assert!(said.contains("not a plain model id"), "{said}");
    // An unlisted but well-formed id is the operator's own declaration and is
    // written: the row is still brazen's, so it can never be unroutable.
    assert!(
        plan(
            SEEDED_MODELS,
            TEMPLATE_PROVIDERS,
            &rows(),
            &pick("my-local-tag"),
            None,
        )
        .is_ok()
    );
}

/// An empty table is brazen unanswered, not brazen answering "none" — it gates
/// nothing, on the same terms as the §9.2 Apply gate.
#[test]
fn an_unanswerable_brazen_gates_no_pick() {
    assert!(
        plan(
            SEEDED_MODELS,
            TEMPLATE_PROVIDERS,
            &[],
            &pick("gpt-5.4"),
            None
        )
        .is_ok()
    );
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

/// The write note names both files and their order — models.yaml first.
#[test]
fn the_write_note_names_both_files_in_order() {
    let (first, rest) = WRITE_NOTE.split_once("models.yaml").unwrap();
    assert!(first.contains("writes"));
    assert!(rest.contains("providers.yaml"));
    assert!(rest.contains("declared"));
}
