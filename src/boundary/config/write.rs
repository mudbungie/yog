//! The pipelines each §9 destination runs (bl-3f46), split from the gestures
//! that name them per §12's line budget: the §9.1 `bz`-validated brazen write,
//! the §9.2 provider-gated file write, the §9.3 staged `lernie config` commit,
//! and the two verdict folds that turn an editor's terminal state into the
//! boundary's `Ok`/`Err`.
//!
//! Nothing here re-implements a pipeline. Each is the same one the §11 panes
//! drive, entered with the deposit's whole text instead of a live RAM draft.

use crate::actions::verbs::Outcome;
use crate::config_edit::RealFileIo;
use crate::config_edit::branch::edit::{
    DraftFile, EditOrigin, EditPlan, drive, next_nonce, stage_files,
};
use crate::config_edit::brazen::{Applied, BrazenEditor, RealBzRunner};
use crate::config_edit::lernie_global::{Editor, Saved};
use crate::opslog::Origin;
use crate::xdg::Env;
use std::path::{Path, PathBuf};

use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;

/// The refusal a moved-underneath file earns, said once for both editors.
pub(super) const CONFLICT: &str =
    "the file changed since it was read — re-read it and restate the apply";

/// The §9.1 pipeline: stage, hand the temp to the linked `bz`, and rename only
/// if it accepts. A malformed config never lands.
///
/// Both halves resolve inside **the workspace the gesture named** (bl-fcd5):
/// the file staged is that sphere's own, and the `bz` that validates it is
/// resolved through the same wall — so the validator reads the very config it
/// is judging rather than another workspace's, or none.
pub(super) fn brazen(deps: &Deps, workspace: &Path, text: &str) -> Result<Reply, String> {
    let paths = super::brazen_paths(deps, workspace);
    let dest = paths.config.clone();
    let io = RealFileIo;
    let mut editor = BrazenEditor::load(paths, &io).map_err(|e| e.to_string())?;
    editor.set_draft(text.to_owned());
    applied(editor.apply(
        &RealBzRunner::resolve(&super::wall_env(deps, workspace)),
        &io,
    ))?;
    Ok(landed(&dest))
}

/// Fold a §9.1 Apply outcome into the boundary's verdict.
pub(super) fn applied(applied: Applied) -> Result<(), String> {
    match applied {
        Applied::Ok => Ok(()),
        Applied::Rejected { stderr } => Err(format!("bz refused the draft: {stderr}")),
        Applied::Conflict => Err(CONFLICT.to_owned()),
        Applied::Io { error } => Err(error),
    }
}

/// The §9.2 pipeline over any plain config file: brazen's effective provider
/// table gates it, then the shared hash-guard + atomic rename. A file that
/// declares no models is clean by construction, so no destination is special.
pub(super) fn write_file(deps: &Deps, dest: PathBuf, text: &str) -> Result<Reply, String> {
    let mut editor = editor_at(&dest)?;
    editor.set_draft(text.to_owned());
    saved(editor.apply(&deps.provider_rows(), &RealFileIo))?;
    Ok(landed(&dest))
}

/// Load one §9.2 editor — the single read both the file apply and the §9.4
/// pick's `models.yaml` half enter through.
pub(super) fn editor_at(dest: &Path) -> Result<Editor, String> {
    Editor::load(dest.to_path_buf(), &RealFileIo).map_err(|e| e.to_string())
}

/// Fold a §9.2 Apply outcome into the boundary's verdict: refusals are the
/// `Err` beside the reply, exactly as a gate refusal is anywhere else.
pub(super) fn saved(saved: Saved) -> Result<(), String> {
    match saved {
        Saved::Ok => Ok(()),
        Saved::Rejected { unknown } => Err(format!(
            "brazen has no provider row for {}",
            unknown
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )),
        Saved::Conflict => Err(CONFLICT.to_owned()),
        Saved::Io { error } => Err(error),
    }
}

/// yog's own clock file (§7.2) under the world's state root.
pub(super) fn cadence_path(world: &Env) -> PathBuf {
    let root = world.yog_state_root();
    root.join(crate::app::cadence::CADENCE_YAML)
}

/// The §9.3 write: stage the text under a fresh nonce and drive `lernie config`
/// with the `$EDITOR` shim standing. The drive's own `ops.jsonl` row is the
/// audit; a non-zero exit rides back as the captured outcome, as every other
/// spawned verb's does.
pub(super) fn commit(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    lineage: &str,
    origin: &EditOrigin,
    path: &str,
    text: &str,
) -> Result<Reply, String> {
    let files = [DraftFile {
        rel_path: path.to_owned(),
        bytes: text.as_bytes().to_vec(),
    }];
    let dir = stage_files(&deps.world.yog_stage_root(), &next_nonce(), &files)
        .map_err(|e| e.to_string())?;
    let plan = EditPlan::compose(&deps.yog_binary, workspace, lineage, origin, &dir);
    let entry = drive(
        &deps.lernie,
        workspace,
        &plan,
        ts,
        &deps.state_root,
        Origin::World,
    );
    Ok(Reply::Outcome(Outcome {
        exit: entry.exit,
        stdout: entry.stdout,
        stderr: entry.stderr,
    }))
}

/// The receipt a file destination earns: the path that now holds the text.
pub(super) fn landed(dest: &Path) -> Reply {
    Reply::Applied {
        file: dest.display().to_string(),
    }
}
