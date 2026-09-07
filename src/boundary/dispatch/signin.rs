//! **The first thing to say about a wall that would reach no model is what to
//! do about it** (DESIGN §8.1, bl-1fd0) — the rung, at the one door every fire
//! passes (bl-2291), asking about the row the fire will actually resolve
//! (bl-58e7).
//!
//! The ruling was that a goal typed into an unsigned wall works zero percent of
//! the time: the conversation is born, dies on no-models, and the operator
//! learns it from a dead row. bl-7cc8 deleted the callerless fold that judged
//! it and left the predicate in the doc, honourable by any seat that asked
//! `/providers` first — and no seat did, because a seat that re-judged the rows
//! would be a second implementation of a settled decision, and there are two
//! seats. So the decision has one home now, and it is the `Prompt` door
//! ([`super::doors::prompt`]): a click, a line and a deposit all pass it, the
//! refusal is the envelope every act already answers with, and a refused fire
//! spends nothing — the caller still holds its goal.
//!
//! # The question is the ROLE's provider, not the wall's best row (bl-58e7)
//!
//! The gate's own sentence used to be *"no provider in this workspace's wall
//! holds a credential"* — **any** provider — and that is not what a fire needs.
//! What it needs is a credential for the provider the governing lineage's
//! `roles` name, because that is the row the loop resolves. One credentialed
//! row the roles do not name satisfied the old predicate and readied nothing:
//! reproduced on two fresh workspaces, where a wall carrying exactly one
//! operator-written row let the start through and the conversation was dead on
//! arrival, having spent a model call to learn a fact the engine held before it
//! fired. In a multi-seat workspace the operator who finds the corpse is not
//! the one who started it.
//!
//! So [`verdict`] reads the lineage's roles first, and the row this fire will
//! resolve must be one the wall can answer with. Three outcomes, and they do
//! not share a remedy — which is the other half of the defect (bl-21e9): a row
//! the wall **does not declare at all** is not a sign-in, and telling its
//! operator to `/login` a row that is not there is a loop with no exit.
//!
//! **The role is the one the fire names, and [`WORKER_ROLE`] when it names
//! none** (bl-9ced). It was `worker` unconditionally, on the true premise that
//! *roots are workers* — until litany 0.0.12 made a root's role an input
//! (`litany prompt --role`, upstream bl-946c) and `Prepared` gained the field
//! (REMOTE §9.21). A door that kept reading `worker` would then read a row the
//! fire does not resolve, in both directions: passing a start whose `planner`
//! points at an empty row, and refusing one whose `planner` points at a good
//! row because `worker` does not. So the role rides in, and the sentences name
//! it — `/model <that role> …` is the remedy, and naming `worker` there would
//! send the operator to fix a row the start never touches.
//! The other declared roles are not the fire's question and refusing on
//! one would be the conservatism §8.1 warns about: litany's shipped template
//! declares a `reviewer` **unbound** — nothing dispatches it until a workflow
//! binds it — so a wall that would run for hours would be refused for a role
//! that never fires. A `compactor` pointing at a dead row does fail, later, at
//! its own moment and through the §8.3 step failure and the row's `failure`
//! clause; that is a corpse with a cause, which is what those surfaces are for,
//! and not the zero-percent birth this rung exists to stop.
//!
//! **A lineage that declares no such role falls back to the wall predicate**,
//! which is the general path with an empty input rather than a special case: a
//! config that could not be read declares nothing (the `Roles` read's own
//! answer rather than a refusal), and the only thing left to ask is whether any
//! row here could answer at all.
//!
//! **The keyless clause survives, moved into the role arm.** brazen merges its
//! built-in table under every config, so `ollama` and `claude-code` read `not
//! required` on every wall there can be and a predicate they satisfied would be
//! one nothing ever fails. A role pointing at one is the operator's own hand,
//! not the merge — so a named keyless row readies the wall and an unnamed one
//! does not. `missing` is the one spelling that refuses, whoever named the row.
//!
//! **An unanswerable table refuses nothing.** brazen unable to answer is an
//! empty table, never an error (the `Providers` read's own contract), and no
//! surface refuses on the strength of a question that went unanswered.

use std::path::Path;

use crate::config_edit::branch::config_file;
use crate::config_edit::brazen::{BzRunner, MISSING, ProviderRow, RealBzRunner};
use crate::model_pick::grammar::RoleModel;
use crate::model_pick::{BRANCH, PROVIDERS, WORKER_ROLE};

use super::super::config::wall_env;
use super::Deps;

/// The sentences, one per [`Unready`] — split off here at §12's line budget on
/// the seam the module already had: this file decides, that one words it.
mod refusal;

#[cfg(test)]
mod tests;

/// The rung: `Ok` when the fire would reach a model, the refusal otherwise.
/// `lineage` is the `Prepared`'s §8.7 birth lineage — `None` is
/// `config/default` — and `role` is its §8.1 birth role — `None` is
/// [`WORKER_ROLE`] — both read exactly as the fire itself reads them.
pub(crate) fn gate(
    deps: &Deps,
    workspace: &Path,
    lineage: Option<&str>,
    role: Option<&str>,
) -> Result<(), String> {
    let rows = RealBzRunner::resolve(&wall_env(deps, workspace)).providers();
    if rows.is_empty() {
        return Ok(());
    }
    let role = role.unwrap_or(WORKER_ROLE);
    match verdict(&rows, &roles(workspace, lineage), role) {
        None => Ok(()),
        Some(unready) => Err(refusal::say(&unready, &rows, role)),
    }
}

/// The role assignments the lineage declares (§9.4's `roles.<r>.provider`). A
/// lineage that cannot be read declares none, which is the `Roles` read's own
/// answer rather than a refusal: the wall predicate alone then decides.
fn roles(workspace: &Path, lineage: Option<&str>) -> Vec<RoleModel> {
    let refspec = format!("config/{}", lineage.unwrap_or(BRANCH));
    let text = config_file(workspace, &refspec, PROVIDERS)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    crate::model_pick::grammar::roles(&text)
}

/// Why this wall would reach no model, or `None` when it would. The role the
/// verdict was taken for is **not** carried: it is the caller's own parameter
/// one line up, so the sentence reads it from where it was decided rather than
/// from a copy — the same reason it was not carried when it could only ever be
/// [`WORKER_ROLE`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Unready {
    /// The fired role points at a row this wall's table does not carry at all —
    /// the bl-21e9 case, whose remedy is the row table and never a sign-in.
    Undeclared { provider: String },
    /// The fired role points at a row that is here and holds no credential. It
    /// carries the row rather than its name because the remedy is the row's own
    /// credential model, and looking it up again would be a second read with a
    /// no-such-row arm nothing can reach.
    Uncredentialed { row: ProviderRow },
    /// The lineage declares no such role, and no row here could answer at all.
    Wall,
}

/// The fold itself, pure over the table, the roles the lineage declares, and
/// the role this fire will be born on.
pub(crate) fn verdict(rows: &[ProviderRow], roles: &[RoleModel], fired: &str) -> Option<Unready> {
    let Some(named) = roles.iter().find(|role| role.role == fired) else {
        return (!rows.iter().any(ProviderRow::credentialed)).then_some(Unready::Wall);
    };
    match rows.iter().find(|row| row.name == named.provider) {
        None => Some(Unready::Undeclared {
            provider: named.provider.clone(),
        }),
        Some(row) if row.credential == MISSING => {
            Some(Unready::Uncredentialed { row: row.clone() })
        }
        Some(_) => None,
    }
}
