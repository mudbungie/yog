//! The model picker (DESIGN §9.4) — the pure half.
//!
//! yog's answer to *"I'm talking to gpt-5.4. How do I change that?"*. The
//! picker is not a fourth config editor: it is the §9.3 lineage write reached by
//! two dropdowns over brazen's own facts.
//!
//! **It was two writes and is one** (bl-d9cb). The pick used to compose §9.2 and
//! §9.3 in a normative order, because lernie's cross-check refused a config
//! whose `roles.<r>.model` was not declared in the global `models.yaml`. That
//! check is retired upstream (lernie's bl-35e2): a role's `providers.yaml`
//! assignment is the single home of its (provider row, model id) pointer, so
//! there is one file to write and no order to get wrong.
//!
//! This module holds everything that can be decided without touching disk,
//! network or egui: the [`grammar`] both config files are read and written
//! through,
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

use crate::config_edit::brazen::{ProviderRow, row_names};

#[cfg(test)]
pub(crate) mod tests;

pub use grammar::{GrammarError, RoleModel};
pub use header::{
    ConfigPoint, ConfigTip, ModelRow, NEW_CONVERSATION_EXIT, RETARGET_EXIT, RETARGET_HOVER,
    birth_row, conversation_row, row_role,
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

/// Why a pick was declined (§9.4). Three kinds, because three things can be
/// wrong: a file is not the block shape yog edits ([`GrammarError`]), the pick
/// names a provider row brazen does not have, or it names a row brazen HAS whose
/// protocol cannot carry a yog turn.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PickError {
    #[error("{0}")]
    Grammar(GrammarError),
    /// bl-bd89. The pick's provider row is not in brazen's effective table, so
    /// the write would land a config that dies at the first dispatch
    /// (`unknown provider`). Refused before the file is touched — the picker's
    /// whole promise is that a listed model yields a working config. Also the
    /// §9.4 role rows' one fault ([`role_fault`]), which is the same question
    /// asked of the assignment already in the file.
    #[error(
        "brazen's table has no provider row `{provider}` — pick a live row here, \
         or add the row to brazen's own config"
    )]
    UnknownProvider { provider: String },
    /// bl-3d22. The row EXISTS and still cannot serve the role: its protocol
    /// declines the request shape every yog turn has. Row existence never
    /// established compatibility — §9.4 used to reason that a custom id could be
    /// unserved *"but never an unroutable one, because the row beside it is
    /// still brazen's"*, and `/model worker claude-code <id>` falsified that by
    /// advancing the config and then failing the next worker start before any
    /// network call. `why` is
    /// [`ProviderRow::tools_blocked`](crate::config_edit::brazen::ProviderRow::tools_blocked)'s
    /// own sentence, so the verb and the picker's dropdown cannot phrase the
    /// same incapability differently.
    #[error("provider row `{provider}` cannot serve a role: {why} — pick a tool-capable row")]
    Incapable { provider: String, why: String },
    /// The model id is not a plain block key, so the `roles:` block form
    /// (§9.4's grammar) cannot hold it. Reachable only from the custom-id entry —
    /// every listed candidate is a string brazen itself printed.
    #[error(
        "`{model}` is not a plain model id — yog writes it as a block value in \
         providers.yaml; use the Config editors for anything else"
    )]
    NotAnId { model: String },
}

/// Why this role's assignment cannot be fired, or `None` when it can (§9.4's
/// role rows, bl-53be as amended by bl-d9cb). **One fault, over the live
/// pointer:** the row `roles.<r>.provider` dispatches through is not one
/// brazen's table has, so every step under it dies with `unknown provider`. The
/// sentence is [`PickError::UnknownProvider`]'s own — the same judgement the pick
/// gate makes one gesture later, so one wording serves both seats.
///
/// It used to judge the role's *model* against the global `models.yaml`: declared
/// there at all, and declared on a live row. Both arms are dead at the pin (see
/// [`grammar`]'s `models` half), while the actual pointer went unjudged — a role
/// on a row brazen had dropped was unmarked whenever its old model entry happened
/// to name a live one.
pub fn role_fault(providers: &[String], role: &RoleModel) -> Option<String> {
    grammar::is_unknown_row(&role.provider, providers).then(|| {
        PickError::UnknownProvider {
            provider: role.provider.clone(),
        }
        .to_string()
    })
}

/// Whether a model id can be written as a `model:` value the anchored grammar
/// reads back. Blank, or carrying whitespace / `:` / `#`, and it would emit a
/// line it could not parse — so the pick is refused instead.
fn is_plain_id(model: &str) -> bool {
    !model.is_empty() && !model.contains([' ', '\t', ':', '#'])
}

/// Where the picker's provider dropdown lands, **and what it had to leave
/// behind to get there** (bl-bd89, amended by bl-dd7f).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scoped {
    /// The row the model list is asked of and a pick is written for.
    pub row: String,
    /// The role's own row, when brazen does not have it — the row the
    /// conversation actually dispatched through, and the reason its first turn
    /// died. `None` whenever the dropdown shows the role's own row, which is
    /// every healthy case.
    pub stranded: Option<String>,
}

impl Scoped {
    /// The sentence the picker paints when it had to steer (bl-dd7f): the row
    /// that failed, named, beside the live one now selected. The wording lives
    /// here, beside the substitution that makes it true, so the dropdown and
    /// the sentence cannot state different rows.
    pub fn strand_note(&self) -> Option<String> {
        self.stranded.as_ref().map(|was| {
            format!(
                "this conversation was dispatched through {was}, which brazen does not \
                 have — {} is selected instead; add or rename the row in config.toml to \
                 keep it",
                self.row
            )
        })
    }
}

/// The provider row the picker queries and writes for a role (bl-bd89): the
/// role's own row when brazen has it, otherwise brazen's first row.
///
/// A role stranded on a row brazen no longer has — renamed, or dropped from
/// `config.toml` — is precisely the state the picker exists to leave, and
/// asking a dead row for its models can only re-report the strand. An **empty**
/// table is no answer rather than an empty one (brazen could not be asked), so
/// it steers nothing and the role's own row stands.
///
/// **The substitution is a fact the caller is told, not one it has to notice**
/// (bl-dd7f, ruled at bl-9b52). It used to return the bare row, so a
/// conversation whose first turn died on `unknown provider \`openai-chatgpt\``
/// showed a picker reading `anthropic` — brazen's first row — and nothing said
/// the swap had happened: the operator read the picker as a report of what ran.
/// Steering is still right; steering silently was not. So the answer carries
/// both halves ([`Scoped`]) and the seat says the second one.
pub fn default_row(current: &str, rows: &[String]) -> Scoped {
    match rows.first() {
        Some(first) if !rows.iter().any(|r| r == current) => Scoped {
            row: first.clone(),
            stranded: Some(current.to_owned()),
        },
        _ => Scoped {
            row: current.to_owned(),
            stranded: None,
        },
    }
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

/// The `providers.yaml` text one pick produces, or a decline (§9.4). **One
/// write** since bl-d9cb — the three gates below all refuse before it, so a
/// dead row, an incapable protocol or a file the grammar cannot read leaves
/// nothing to recover from, and there is no half-written state to order.
///
/// `rows` is brazen's effective provider table, carried **whole** since bl-3d22:
/// the row gate asks two questions of it, and only one of them is answerable
/// from a name. An empty table is no answer and gates nothing, on the same terms
/// as [`grammar::unknown_rows`] — and a row the table does not carry has no
/// protocol to judge, so the capability gate dissolves into the same rule with
/// no case of its own.
pub fn plan(providers_yaml: &str, rows: &[ProviderRow], pick: &Pick) -> Result<String, PickError> {
    if grammar::is_unknown_row(&pick.provider, &row_names(rows)) {
        return Err(PickError::UnknownProvider {
            provider: pick.provider.clone(),
        });
    }
    if let Some(why) = rows
        .iter()
        .find(|row| row.name == pick.provider)
        .and_then(ProviderRow::tools_blocked)
    {
        return Err(PickError::Incapable {
            provider: pick.provider.clone(),
            why,
        });
    }
    if !is_plain_id(&pick.model) {
        return Err(PickError::NotAnId {
            model: pick.model.clone(),
        });
    }
    grammar::set_role_model(providers_yaml, &pick.role, &pick.provider, &pick.model)
        .map_err(PickError::Grammar)
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

/// The note beside the write, naming the one file a click touches (§9.4), so the
/// operator can see what it is about to do before it does it.
///
/// It named two files and their order until bl-d9cb, and both halves of that
/// sentence were false at the pin: lernie reads no global `models:` table, so the
/// declaration was inert and its ordering rule protected nothing.
pub const WRITE_NOTE: &str = "writes the role's provider and model into providers.yaml on this workspace's \
     config branch, through `lernie config`";
