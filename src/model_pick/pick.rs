//! **One pick** (DESIGN §9.4): the operator's choice, the three gates it must
//! pass, and the `providers.yaml` text it produces.
//!
//! One write since bl-d9cb — the gates all refuse *before* it, so a dead
//! provider row, an incapable protocol or a file the grammar cannot read leaves
//! nothing half-written to recover from, and there is no order to get wrong.
//! The same row judgement is asked of the assignment already in the file
//! ([`role_fault`]), so the §9.4 role rows and the pick gate one gesture later
//! phrase one fault one way.
//!
//! Split off [`super`] at §12's pre-split band: that module is the picker's
//! vocabulary — the role and branch it writes, where its dropdown lands and
//! what it had to leave behind to get there, and the sentences the surface
//! paints — and this is the gesture itself.

use super::{GrammarError, RoleModel, grammar};
use crate::config_edit::brazen::{ProviderRow, row_names};

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

/// One operator choice: give `role` this `model` on this provider row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    pub role: String,
    pub provider: String,
    pub model: String,
}

impl Pick {
    /// One choice, owned. A constructor rather than a literal at the call site
    /// because the boundary's caller is a match arm with a line budget (§12) —
    /// the pressure [`edit::Create`](crate::actions::verbs::edit::Create)
    /// answers by carrying its family's whole payload instead (bl-dbde).
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
