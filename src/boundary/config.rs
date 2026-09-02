//! The §9 config family at the boundary (§8.5, bl-3f46): the config editors,
//! the §16.3 marks knob and the §9.4 model pick as real
//! [`Action`](super::Action) variants, executed here.
//!
//! Until this landed, §8.5 recorded these as *actions by the taxonomy but not
//! yet variants*: each already funnelled through its own single chokepoint
//! (`Editor::apply`, `edit::drive`, `world::marks::apply`) and logged to
//! `ops.jsonl`, so no gesture had two implementations meanwhile. They now enter
//! the enum through this one module, and the §8.5 compile gate covers them —
//! codec, line and dispatch are all exhaustive.
//!
//! **A config apply is a destination plus the full staged text.** That is the
//! whole reframe: [`ConfigFile`] enumerates *where* the bytes land, and the
//! pipeline follows from it — `bz` validates a brazen draft (§9.1), a
//! litany-global one is hash-guarded and renamed (§9.2), and a per-workspace
//! lineage is staged and committed by `litany config` (§9.3, the only lawful
//! writer of `config/*`). One variant, no per-file gesture.
//!
//! **One validator, at the one destination whose contents yog can judge.** §9.2
//! held a second between bl-53be and bl-3ffa, over `models.<id>.provider`; it is
//! retired with the field's last reader, so the three plain-file destinations run
//! the same unjudged pipeline rather than one of them being gated on a table's
//! shape (§9.2).
//!
//! **A deposit carries no hash guard, and needs none.** The §9 editors' guard
//! protects a *long-lived* RAM draft against a file that moved under it; a
//! gesture states its whole text in one atomic instruction, so the load and the
//! apply are microseconds apart and the guard degenerates to the must-not-exist
//! check a new file wants. Nothing here re-implements the pipeline: every write
//! is the same `stage → validate → hash-guard → atomic rename` the panes drive.

use crate::config_edit::branch::config_file;
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::brazen::{BrazenPaths, BzRunner, RealBzRunner, row_views};
use crate::config_edit::litany_global::LitanyGlobal;
use crate::model_pick::{BRANCH, PROVIDERS, Pick};
use crate::world::marks;
use std::path::Path;

use super::dispatch::Deps;
use super::reply::Reply;

/// The destination datum and its addressing — split at §12's cap (bl-f5f6).
mod file;
pub use file::ConfigFile;
pub use read::Read;

pub(crate) mod read;
pub(crate) mod write;
use read::{branch_text, text_at};
use write::{cadence_path, commit};

/// Run one config apply (§9). The reply says what landed: a file destination
/// answers with the path written, a lineage with `litany config`'s captured run
/// — the same distinction every other action makes between a write and a spawn.
///
/// `ws` is the destination's own workspace, resolved by the chokepoint's one
/// address resolution (REMOTE §8, bl-523f) — the empty path for the three
/// destinations that name no world, which is the general path with no input:
/// no arm that takes it reads the value.
pub(super) fn apply(
    deps: &Deps,
    ts: &str,
    ws: &Path,
    file: &ConfigFile,
    text: &str,
) -> Result<Reply, String> {
    match file {
        ConfigFile::Brazen { .. } => write::brazen(deps, ws, text),
        ConfigFile::LitanyModels => {
            write::write_file(LitanyGlobal::resolve(&deps.world).models(), text)
        }
        ConfigFile::LitanyWorkflow { name } => {
            let path = LitanyGlobal::resolve(&deps.world)
                .new_workflow(name)
                .map_err(|e| e.to_string())?;
            write::write_file(path, text)
        }
        ConfigFile::Cadence => write::write_file(cadence_path(&deps.world), text),
        ConfigFile::Branch {
            lineage,
            origin,
            path,
            ..
        } => commit(deps, ts, ws, lineage, origin, path, text),
    }
}

/// Read one §9 destination's current bytes (§8.5, bl-0164): [`apply`]'s
/// read-only twin, and the file editors' Reload spelled headless. A file
/// destination that is not there yet answers empty text — the same "new
/// file" reading every editor's own load already gives — so only a real I/O
/// failure refuses. A **lineage** answers the pane's own Load (bl-dff8): `git
/// show config/<lineage>:<path>`, the very bytes an Apply on that destination
/// would be diffed against. It carries the write's `origin` and ignores it,
/// because where the next commit lands is not where the current bytes are;
/// [`Query::Lineages`](super::Query::Lineages) is the browse that says which
/// paths a lineage holds.
pub(super) fn read(deps: &Deps, ws: &Path, file: &ConfigFile) -> Result<Reply, String> {
    let text = match file {
        ConfigFile::Brazen { .. } => text_at(&brazen_paths(deps, ws).config)?,
        ConfigFile::LitanyModels => text_at(&LitanyGlobal::resolve(&deps.world).models())?,
        ConfigFile::LitanyWorkflow { name } => text_at(
            &LitanyGlobal::resolve(&deps.world)
                .new_workflow(name)
                .map_err(|e| e.to_string())?,
        )?,
        ConfigFile::Cadence => text_at(&cadence_path(&deps.world))?,
        ConfigFile::Branch { lineage, path, .. } => branch_text(ws, lineage, path)?,
    };
    Ok(Reply::Config { text })
}

/// The §9.3 browse (§8.5, bl-dff8): the workspace's lineages, each with the
/// files its tip holds — the pane's dropdowns, as one answer.
pub(super) fn lineages(workspace: &Path) -> Result<Reply, String> {
    read::browse(workspace).map(Reply::Lineages)
}

/// The §9.4 roster (§8.5, bl-dff8): what `provider` offers **in this
/// workspace's wall** — the picker's own read, aimed by the gesture rather
/// than by a focus a headless seat does not have (bl-fcd5).
pub(super) fn models(deps: &Deps, workspace: &Path, provider: &str) -> Result<Reply, String> {
    read::models(&wall_env(deps, workspace), provider).map(Reply::Models)
}

/// **Which branch this agent tracks on** (§8.5, bl-0164): the marks pane's
/// `Read current`, over the same space [`set_marks`] re-reads after it writes.
/// Infallible — a space with nothing written reads as balls' own default, which
/// is the general path with no input rather than a refusal to render.
pub(super) fn read_marks(deps: &Deps, workspace: &Path) -> Reply {
    Reply::Marks {
        branch: marks::read(&deps.world, workspace).branch(),
    }
}

/// One workspace's effective provider table, rendered (§8.5, bl-0164) — the
/// §8.3 login pane's `↻ providers + credentials`. The table is read **inside
/// the named sphere's wall** (bl-fcd5): a provider row reads *signed in* only
/// where this workspace signed it in, so the table is meaningless without the
/// workspace that scopes it. The credential fact rides the listing's own
/// `credential` column (bl-dba3) — one ask, and no second derivation over the
/// credentials directory. brazen unanswerable is an empty table, never an
/// error: the same "asked, never stored" contract [`Deps::provider_rows`]
/// carries.
pub(super) fn providers(deps: &Deps, workspace: &Path) -> Reply {
    let wall = wall_env(deps, workspace);
    Reply::Providers(row_views(&RealBzRunner::resolve(&wall).providers()))
}

/// This workspace's **role assignments** (§9.4, §5.1 #27; bl-2410) — what
/// `/model`, `/effort` and `/priority` have actually set, read back.
///
/// It reads exactly where those three write: `providers.yaml` at the tip of the
/// lineage §9.3 writes, through the one anchored grammar this tree has. A read
/// and a write naming the place the same way is the discipline the §9 family
/// already keeps, and here it is also what makes the answer *current* — under
/// follow-the-tip that tip is what every conversation in this workspace
/// resolves at its next step, so a control showing it is showing what governs.
///
/// **A lineage it cannot read declares no role**, and that is an answer rather
/// than a refusal: a workspace with no config yet, or one whose `roles:` block
/// is absent or inline, has nothing set — which is exactly what a control
/// opening on it should show. That differs from the §11 `Governing` read, which
/// refuses, and the difference is real: *this conversation has no policy* is
/// never true, while *this workspace has assigned no role* is an ordinary state
/// a fresh world passes through.
pub(super) fn roles(workspace: &Path) -> Reply {
    let text = config_file(workspace, &format!("config/{BRANCH}"), PROVIDERS)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    Reply::Roles(crate::model_pick::grammar::roles(&text))
}

/// The named workspace's brazen locations (§16.2 as amended). **The gesture's
/// own workspace is the single source** (bl-fcd5): the executor lenses on it
/// rather than trusting whatever wall happened to stand in `deps.world`, so a
/// windowless seat reaches exactly the sphere it named and a window reaches
/// exactly the one it has focused. Infallible by construction — a wall is a
/// pure function of the world anchor and the workspace's leaf, and a gesture
/// with no workspace never got this far (the line reader and the codec each
/// refuse it by name).
pub(super) fn brazen_paths(deps: &Deps, workspace: &Path) -> BrazenPaths {
    BrazenPaths::in_wall(&crate::world::wall::root_of(&deps.world, workspace))
}

/// The world with `workspace`'s wall standing — the lens every brazen fold in
/// this module reads through. Idempotent, so re-lensing a `deps.world` the
/// window already lensed on its focus replaces the wall rather than stacking
/// one.
pub(super) fn wall_env(deps: &Deps, workspace: &Path) -> crate::xdg::Env {
    crate::world::wall::env(&deps.world, workspace)
}

/// **Amend this agent's tracking branch** (§16.3): write `branch` into the
/// workspace's own balls space (logged, §4.2), then answer with the **re-read**
/// branch — what actually landed, not what was asked for. An unlawful branch
/// refuses at the write in the words the grammar already refused it with.
pub(super) fn set_marks(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    branch: &str,
) -> Result<Reply, String> {
    let space = marks::read(&deps.world, workspace);
    let landed = marks::apply(&space, &deps.state_root, ts, branch).map_err(|e| e.to_string())?;
    Ok(Reply::Marks { branch: landed })
}

/// The §9.4 pick: **one write** (bl-d9cb) — the §9.3 lineage write of
/// `providers.yaml`, which is the single home of a role's (provider row, model
/// id) pointer. The text is composed first, so a dead provider row, an incapable
/// protocol or a file the grammar cannot read refuses before anything is
/// written.
///
/// It used to apply `models.yaml` through the §9.2 pipeline first, in that order,
/// because litany's cross-check refused a config naming an undeclared model.
/// litany retired that check and the table with it (its bl-35e2), so the first
/// write reached nothing that reads it.
///
/// The provider gate reads the rows of **the workspace being picked for**
/// (bl-fcd5), not of whatever wall stood in `deps.world`: the pick already
/// names its sphere, and a row that is dead in one workspace may be live in
/// another, so judging with the wrong wall would refuse a valid pick — or,
/// headless with no wall at all, gate on an empty table and let anything
/// through.
///
/// The table is handed to [`plan`](crate::model_pick::plan) **whole** since
/// bl-3d22: the gate asks whether the row exists AND whether its protocol can
/// carry a yog turn, and the second question is not answerable from a name.
pub(super) fn pick_model(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    pick: &Pick,
) -> Result<Reply, String> {
    let assigned = config_file(workspace, &format!("config/{BRANCH}"), PROVIDERS)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| e.to_string())?;
    let table =
        crate::config_edit::brazen::RealBzRunner::resolve(&wall_env(deps, workspace)).providers();
    let assigned = crate::model_pick::plan(&assigned, &table, pick).map_err(|e| e.to_string())?;
    commit(
        deps,
        ts,
        workspace,
        BRANCH,
        &EditOrigin::Advance,
        PROVIDERS,
        &assigned,
    )
}

/// The §9.4 **tuning pair** (bl-23bd) — a role's effort level or its priority
/// lane, written into the same `providers.yaml`, on the same lineage, through
/// the same commit [`pick_model`] spends.
///
/// Beside it rather than inside it because the two answer different questions
/// of the same file: a pick moves the (row, id) pointer and must be gated
/// against brazen's live table, while a tuning knob is a value the config is
/// always free to carry — the capability decides which control a seat *offers*
/// (`ProviderRowView`'s two booleans), never whether a write is allowed. So
/// this reads no provider table at all, which is also why it cannot be a wider
/// pick: the pick's own gates would have nothing to judge.
///
/// The read → plan → commit shape is [`pick_model`]'s verbatim, and
/// deliberately so: one staging path, one `litany config` drive, one
/// [`Reply::Outcome`], so a tuning gesture and a pick fail the same way when
/// the lineage will not take an edit.
pub(super) fn tune(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    tuning: &crate::model_pick::Tuning,
) -> Result<Reply, String> {
    let assigned = config_file(workspace, &format!("config/{BRANCH}"), PROVIDERS)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| e.to_string())?;
    let assigned = crate::model_pick::tuning::plan(&assigned, tuning).map_err(|e| e.to_string())?;
    commit(
        deps,
        ts,
        workspace,
        BRANCH,
        &EditOrigin::Advance,
        PROVIDERS,
        &assigned,
    )
}

#[cfg(test)]
mod tests;
