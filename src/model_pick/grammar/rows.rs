//! The one provider-row judgement every §9 gate calls ([`is_unknown_row`]).
//!
//! **This file was the `models:` half of the grammar** — yog's own table in
//! litany's `models.yaml`, one fact wide: the `context_window` the §5.1 #35
//! fullness figure divided by, authored by a Declare control and edited by
//! §9.5 typed rows. bl-9c8a deleted the table and everything over it. The
//! window is the provider's fact, and brazen now states it in band on every
//! `Usage` event the engine already records per step (its model-discovery
//! §5.5), which is the number litany's own `window_percent` compaction trigger
//! divides by; a window yog declared on its own was a second representation of
//! that fact, one the engine compacting the context could never see. The
//! figure reads the window off the step record (`budgets::context_window`), a
//! row that states none renders no figure, and the seat to state one is
//! upstream on brazen's provider row — where for Ollama it is already the
//! `num_ctx` in force.

/// **The** provider-row judgement: does `provider` name no row in brazen's
/// effective table? Every site that asks it asks it here — the §9.4 pick gate
/// ([`plan`](crate::model_pick::plan)), the §9.4 role marks
/// ([`role_fault`](crate::model_pick::role_fault)), and the §9.5 pane's provider
/// control over `providers.yaml` ([`crate::config_edit::form`]) — so the three
/// can never disagree. Each judges the LIVE pointer, `roles.<r>.provider`, which
/// is the whole of a role's binding (bl-35e2), against the wall of the workspace
/// that holds it. Two sites are gone and both for the same reason, a judgement
/// made where the answer was not: the birth gate (bl-c3a9, retired bl-00ee)
/// asked before a wall existed, and the §9.2 Apply gate (bl-53be, retired
/// bl-3ffa) asked about `models.<id>.provider`, a field nothing dispatches
/// through.
///
/// `providers` is `bz --list-providers`' answer (built-ins included, which a
/// scan of `config.toml` would miss). An **empty** table is no answer rather
/// than an empty one — brazen could not be asked — so it judges nothing: no
/// surface may refuse on the strength of a question that went unanswered.
pub fn is_unknown_row(provider: &str, providers: &[String]) -> bool {
    !providers.is_empty() && !providers.iter().any(|p| p == provider)
}
