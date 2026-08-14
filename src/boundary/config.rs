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
//! pipeline follows from it — `bz` validates a brazen draft (§9.1), brazen's
//! provider table gates a lernie-global one (§9.2), and a per-workspace lineage
//! is staged and committed by `lernie config` (§9.3, the only lawful writer of
//! `config/*`). One variant, no per-file gesture.
//!
//! **The gate runs over every file destination, with no branch on which.** A
//! `workflows/*.yaml` and yog's own `cadence.yaml` declare no `models:` block,
//! so the §9.2 provider gate is always clean there — the general path with
//! nothing to judge, rather than three files' worth of special case.
//!
//! **A deposit carries no hash guard, and needs none.** The §9 editors' guard
//! protects a *long-lived* RAM draft against a file that moved under it; a
//! gesture states its whole text in one atomic instruction, so the load and the
//! apply are microseconds apart and the guard degenerates to the must-not-exist
//! check a new file wants. Nothing here re-implements the pipeline: every write
//! is the same `stage → validate → hash-guard → atomic rename` the panes drive.

use crate::config_edit::RealFileIo;
use crate::config_edit::branch::config_file;
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::brazen::{
    BrazenPaths, BzRunner, RealBzRunner, credential_presence, row_views,
};
use crate::config_edit::lernie_global::LernieGlobal;
use crate::model_pick::{BRANCH, PROVIDERS, Pick};
use crate::world::marks;
use std::path::{Path, PathBuf};

use super::dispatch::Deps;
use super::reply::Reply;

pub(crate) mod read;
pub(crate) mod write;
use read::{branch_text, text_at};
use write::{cadence_path, commit, editor_at, saved};

/// Where one [`ApplyConfig`](super::Action::ApplyConfig) lands (§9). The
/// destination decides the pipeline, so the gesture carries no mode flag: a
/// brazen draft is `bz`-validated, a lernie-global one is provider-gated, and a
/// lineage file is committed by `lernie config`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFile {
    /// One workspace's own brazen `config.toml` (§9.1, §16.2's wall) —
    /// validated by the linked `bz` before it lands, because `bz` is the only
    /// lawful parser of that schema.
    ///
    /// **It names its workspace, exactly as [`Branch`](ConfigFile::Branch)
    /// does** (bl-fcd5). Since the blast-radius ruling this file lives inside a
    /// wall, so the destination is not "brazen's config" but "*this sphere's*
    /// brazen config" — a fact the gesture must carry, because a headless seat
    /// has no focus to derive it from and a teleoperator could otherwise not
    /// reach provider config at all. The seat states it (`--ws`, or the
    /// envelope's `workspace`); the window fills it from focus; a gesture that
    /// names neither is refused where every other missing target is, at the
    /// edge.
    Brazen { workspace: PathBuf },
    /// lernie's global `models.yaml` (§9.2).
    LernieModels,
    /// One `workflows/<name>.yaml` (§9.2). The name must be a safe single-file
    /// basename; anything else is refused before a byte is staged.
    LernieWorkflow { name: String },
    /// yog's own `cadence.yaml` — the clock's periods (§7.2), on the same §9
    /// editor discipline because the file is a file.
    Cadence,
    /// One file on a per-workspace config lineage (§9.3): staged, then handed
    /// to `lernie config`, whose `$EDITOR` callback re-enters this binary and
    /// copies it over the checkout. lernie commits; yog never writes inside a
    /// workspace.
    Branch {
        workspace: PathBuf,
        /// The lineage's bare name — `lernie config <ws> <lineage>`.
        lineage: String,
        /// Advance it, fork it off a source, or start a fresh orphan (§9.3).
        origin: EditOrigin,
        /// The checkout-relative path this text is.
        path: String,
    },
}

/// Run one config apply (§9). The reply says what landed: a file destination
/// answers with the path written, a lineage with `lernie config`'s captured run
/// — the same distinction every other action makes between a write and a spawn.
pub(super) fn apply(deps: &Deps, ts: &str, file: &ConfigFile, text: &str) -> Result<Reply, String> {
    match file {
        ConfigFile::Brazen { workspace } => write::brazen(deps, workspace, text),
        ConfigFile::LernieModels => {
            write::write_file(deps, LernieGlobal::resolve(&deps.world).models(), text)
        }
        ConfigFile::LernieWorkflow { name } => {
            let path = LernieGlobal::resolve(&deps.world)
                .new_workflow(name)
                .map_err(|e| e.to_string())?;
            write::write_file(deps, path, text)
        }
        ConfigFile::Cadence => write::write_file(deps, cadence_path(&deps.world), text),
        ConfigFile::Branch {
            workspace,
            lineage,
            origin,
            path,
        } => commit(deps, ts, workspace, lineage, origin, path, text),
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
pub(super) fn read(deps: &Deps, file: &ConfigFile) -> Result<Reply, String> {
    let text = match file {
        ConfigFile::Brazen { workspace } => text_at(&brazen_paths(deps, workspace).config)?,
        ConfigFile::LernieModels => text_at(&LernieGlobal::resolve(&deps.world).models())?,
        ConfigFile::LernieWorkflow { name } => text_at(
            &LernieGlobal::resolve(&deps.world)
                .new_workflow(name)
                .map_err(|e| e.to_string())?,
        )?,
        ConfigFile::Cadence => text_at(&cadence_path(&deps.world))?,
        ConfigFile::Branch {
            workspace,
            lineage,
            path,
            ..
        } => branch_text(workspace, lineage, path)?,
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
    let space = marks::read(&deps.world, workspace);
    Reply::Marks {
        branch: space.branch(),
        space: space.state,
    }
}

/// One workspace's effective provider table with the §5.1 #22 credential
/// presence, rendered (§8.5, bl-0164) — the §8.3 login pane's `↻ providers +
/// credentials`. Rows and credentials are read **inside the named sphere's
/// wall** (bl-fcd5): a provider row reads *signed in* only where this
/// workspace signed it in, so the table is meaningless without the workspace
/// that scopes it. brazen unanswerable is an empty table, never an error: the
/// same "asked, never stored" contract [`Deps::provider_rows`] carries.
pub(super) fn providers(deps: &Deps, workspace: &Path) -> Reply {
    let wall = wall_env(deps, workspace);
    let rows = RealBzRunner::resolve(&wall).providers();
    let dir = BrazenPaths::of(&wall)
        .map(|p| p.credentials_dir)
        .unwrap_or_default();
    let creds = credential_presence(&dir, &rows, &RealFileIo);
    Reply::Providers(row_views(&rows, &creds))
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
    Ok(Reply::Marks {
        branch: landed,
        space: space.state,
    })
}

/// The §9.4 pick: §9.2 and §9.3 composed by one gesture, because lernie's
/// cross-check makes a role assignment and a model declaration two halves of
/// one fact. The plan is composed first, so a dead provider row or an
/// unreadable file refuses before either half is written; then `models.yaml`
/// lands **first** (a role naming an undeclared model bricks the workspace),
/// and `providers.yaml` goes through the §9.3 lineage write.
///
/// The provider gate reads the rows of **the workspace being picked for**
/// (bl-fcd5), not of whatever wall stood in `deps.world`: the pick already
/// names its sphere, and a row that is dead in one workspace may be live in
/// another, so judging with the wrong wall would refuse a valid pick — or,
/// headless with no wall at all, gate on an empty table and let anything
/// through.
pub(super) fn pick_model(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    pick: &Pick,
) -> Result<Reply, String> {
    let assigned = config_file(workspace, &format!("config/{BRANCH}"), PROVIDERS)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| e.to_string())?;
    let mut editor = editor_at(&LernieGlobal::resolve(&deps.world).models())?;
    let rows = crate::config_edit::brazen::row_names(
        &crate::config_edit::brazen::RealBzRunner::resolve(&wall_env(deps, workspace)).providers(),
    );
    let planned = crate::model_pick::plan(
        editor.draft(),
        &assigned,
        &rows,
        pick,
        served_window(deps, workspace, pick),
    )
    .map_err(|e| e.to_string())?;
    if let Some(text) = planned.models_yaml {
        editor.set_draft(text);
        saved(editor.apply(&rows, &RealFileIo))?;
    }
    commit(
        deps,
        ts,
        workspace,
        BRANCH,
        &EditOrigin::Advance,
        PROVIDERS,
        &planned.providers_yaml,
    )
}

/// The context window brazen itself served for the picked model (bl-848f),
/// read out of the model cache `bz --list-models` wholesale-writes inside
/// **this workspace's** wall — the roster the picker offered the id from, still
/// on disk, and the same one every other seat's pick would have read.
///
/// It seeds the declaration; it is not a second reader of the fact. bl-a48b's
/// ruling stands — the §5.1 #35 fullness figure reads the declared window in
/// `models.yaml` and nothing else. `None` (the provider serves no window, the
/// cache was never written, or it names some other model) is the honest miss
/// the §9.4 default is for.
fn served_window(deps: &Deps, workspace: &Path, pick: &Pick) -> Option<u32> {
    let cached = crate::config_edit::brazen::model_cache_at(
        &brazen_paths(deps, workspace).models_cache_dir,
        &pick.provider,
        &RealFileIo,
    )
    .ok()
    .flatten()?;
    crate::model_pick::query::served_window(&cached, &pick.model)
}

#[cfg(test)]
mod tests;
