//! The model picker (DESIGN §9.4) — the pure half.
//!
//! yog's answer to *"I'm talking to gpt-5.4. How do I change that?"*. The
//! picker is not a fourth config editor: it is §9.2 and §9.3 **composed** by
//! one gesture, because lernie's cross-check makes a role assignment and a
//! model declaration two halves of one fact. Refuse either half and neither is
//! written.
//!
//! This module holds everything that can be decided without touching disk,
//! network or egui: the [`grammar`] the two files are read and written through,
//! the [`query`] view-model over the live `bz --list-models` run, the [`plan`]
//! one pick produces, the [`header`] row the conversation wears (the pair its
//! two dropdowns show, plus the §9.4 drift clause), the [`remedy`] an
//! auth-shaped roster failure routes to, and the
//! sentences the surface paints — kept here rather than in
//! the excluded shell so the scope claim the UI makes is testable, and so it
//! has exactly one home.

pub mod grammar;
pub mod header;
pub mod query;
pub mod remedy;

#[cfg(test)]
pub(crate) mod tests;

pub use grammar::{GrammarError, RoleModel};
pub use header::{
    ConfigPoint, ModelRow, NEW_CONVERSATION_EXIT, birth_row, conversation_row, row_role,
};
pub use remedy::{Remedy, remedy};

/// The role that talks to you (lernie's `WORKER_ROLE`) — the one the
/// conversation's model row shows and writes, because that is the question
/// being asked; the picker's role strip re-scopes the same two dropdowns.
pub const WORKER_ROLE: &str = "worker";

/// The config branch a pick advances — `lernie config <ws>`'s own default
/// (§9.3). A differently-named lineage is edited through the §9.3 surface; the
/// picker deliberately offers no branch chooser. Named here rather than in a
/// frontend because both seats that fire a pick — the §11 pane and the §8.5
/// [`PickModel`](crate::boundary::Action::PickModel) variant — must name the
/// same lineage or they are two pickers.
pub const BRANCH: &str = "default";

/// The file the role assignment lives in, relative to the config checkout.
pub const PROVIDERS: &str = "providers.yaml";

/// Why a pick was declined (§9.4). Two kinds, because two things can be wrong:
/// a file is not the block shape yog edits ([`GrammarError`]), or the pick
/// names a provider row brazen does not have.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PickError {
    #[error("{0}")]
    Grammar(GrammarError),
    /// bl-bd89. The pick's provider row is not in brazen's effective table, so
    /// both writes would land a config that dies at the first dispatch
    /// (`unknown provider`). Refused before either file is touched — the
    /// picker's whole promise is that a listed model yields a working config.
    #[error(
        "brazen's table has no provider row `{provider}` — pick a live row here, \
         or add the row to brazen's own config"
    )]
    UnknownProvider { provider: String },
    /// The model id is not a plain block key, so neither file's block form
    /// (§9.4's grammar) can hold it. Reachable only from the custom-id entry —
    /// every listed candidate is a string brazen itself printed.
    #[error(
        "`{model}` is not a plain model id — yog writes it as a block key in \
         both models.yaml and providers.yaml; use the Config editors for \
         anything else"
    )]
    NotAnId { model: String },
}

/// Whether a model id can be written as a two-space block key in both files.
/// Blank, or carrying whitespace / `:` / `#`, and the anchored grammar would
/// emit a line it could not read back — so the pick is refused instead.
fn is_plain_id(model: &str) -> bool {
    !model.is_empty() && !model.contains([' ', '\t', ':', '#'])
}

/// The provider row the picker queries and writes for a role (bl-bd89): the
/// role's own row when brazen has it, otherwise brazen's first row.
///
/// A role stranded on a row brazen no longer has — renamed, or dropped from
/// `config.toml` — is precisely the state the picker exists to leave, and
/// asking a dead row for its models can only re-report the strand. An **empty**
/// table is no answer rather than an empty one (brazen could not be asked), so
/// it steers nothing and the role's own row stands.
pub fn default_row(current: &str, rows: &[String]) -> String {
    match rows.first() {
        Some(first) if !rows.iter().any(|r| r == current) => first.clone(),
        _ => current.to_owned(),
    }
}

/// The two file texts one pick produces (§9.4). `models_yaml` is `None` when
/// the id is already declared on the picked provider row and nothing needs
/// writing there.
///
/// **Order is normative:** the caller writes `models_yaml` first. A model
/// declared with no role naming it is inert; a role naming an undeclared model
/// bricks every step in the workspace, because the config load fails before the
/// model is ever called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub models_yaml: Option<String>,
    pub providers_yaml: String,
}

/// One operator choice: give `role` this `model` on this provider row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    pub role: String,
    pub provider: String,
    pub model: String,
}

impl Pick {
    /// One choice, owned. A constructor rather than a literal at the call site
    /// for the same reason [`crate::actions::verbs::Update::of`] is one: the
    /// boundary's caller is a match arm with a line budget (§12).
    pub fn of(role: &str, provider: &str, model: &str) -> Self {
        Self {
            role: role.to_owned(),
            provider: provider.to_owned(),
            model: model.to_owned(),
        }
    }
}

/// Compose the two writes for one pick, or decline (§9.4). The provider row is
/// checked **first** (bl-bd89) and the `models.yaml` half second, so neither a
/// dead row nor a file the grammar cannot read leaves half-written state to
/// recover from.
///
/// `rows` is brazen's effective provider table; an empty one is no answer and
/// gates nothing, on the same terms as [`grammar::unknown_rows`].
///
/// `served` is the context window brazen published for this model on this row
/// (`None` where the provider publishes none) — the seed for the declaration,
/// so a window the roster already carried is never overwritten with a guess
/// (bl-848f). It reaches only the `models.yaml` half, and only a new entry.
pub fn plan(
    models_yaml: &str,
    providers_yaml: &str,
    rows: &[String],
    pick: &Pick,
    served: Option<u32>,
) -> Result<Plan, PickError> {
    if grammar::is_unknown_row(&pick.provider, rows) {
        return Err(PickError::UnknownProvider {
            provider: pick.provider.clone(),
        });
    }
    if !is_plain_id(&pick.model) {
        return Err(PickError::NotAnId {
            model: pick.model.clone(),
        });
    }
    let declared = grammar::declare_model(models_yaml, &pick.model, &pick.provider, served)
        .map_err(PickError::Grammar)?;
    let assigned = grammar::set_role_model(providers_yaml, &pick.role, &pick.provider, &pick.model)
        .map_err(PickError::Grammar)?;
    Ok(Plan {
        models_yaml: declared,
        providers_yaml: assigned,
    })
}

/// The scope sentence the picker paints **at the point of change** (§9.4). It
/// says the thing the operator would otherwise get wrong: this advances a
/// workspace-wide config branch, and the conversation in front of them keeps
/// the policy it forked off.
pub fn scope_sentence(workspace_leaf: &str, branch: &str, short_oid: &str) -> String {
    format!(
        "changes config/{branch} for the whole {workspace_leaf} workspace — \
         it governs the NEXT conversation started here; this one stays frozen \
         at {short_oid}"
    )
}

/// The scope sentence the same picker paints when it is opened from the §11
/// **birth-config block** — the surface for a conversation not started yet.
///
/// [`scope_sentence`] is about a conversation already frozen ("this one stays
/// frozen at …"), which is not a fact the birth block has. What the birth block
/// must say instead is the one thing the operator would otherwise get wrong:
/// **there is no per-conversation pick to make.** lernie 0.0.3's `lernie prompt`
/// takes no config argument and resolves the head of `config/<branch>` itself,
/// so a start-time pick *is* the workspace default moving — the same write the
/// §9.4 picker always did, made one gesture before the start instead of after it.
pub fn birth_sentence(workspace_leaf: &str, branch: &str) -> String {
    format!(
        "this moves the {workspace_leaf} workspace default too: picking here \
         advances config/{branch}, and every conversation started next — \
         including the one you are about to start — is born on it"
    )
}

/// The note beside the write, naming both files and their order (§9.4), so the
/// operator can see what a click is about to do before it does it. It says
/// which of the two generated fields is a guess and which is not (bl-848f): the
/// context window is the provider's own wherever the roster served one, and a
/// declared default only where none was.
pub const WRITE_NOTE: &str = "writes models.yaml first (capabilities are a declared default; the \
     context window is the number this provider served, or a declared default \
     where it serves none), then providers.yaml through `lernie config`";
