//! **The §9 config WRITE family, as one gesture** (bl-dd88) and the pipelines
//! each destination runs (bl-3f46).
//!
//! The five *reads* folded onto [`Read`](super::Read) in bl-719a and the four
//! writes did not, so `action.rs` carried the family four times and rested on
//! the 300 wall — the inversion §12 names, firing on whoever touches it next.
//! [`Write`] is the matching carrier: one variant on the roster, one arm at the
//! chokepoint, one address-table row, and each member keeps its own slash verb,
//! envelope `op` and help page. **The fold is in the carrier, never in the
//! surface**, so no protocol version moves and the corpus regenerates
//! byte-identical — the same check the questions' own fold was made under.
//!
//! Nothing here re-implements a pipeline. Each is the same one the §11 panes
//! drive, entered with the deposit's whole text instead of a live RAM draft.

use crate::actions::verbs::Outcome;
use crate::config_edit::RealFileIo;
use crate::config_edit::branch::edit::{
    DraftFile, EditOrigin, EditPlan, drive, next_nonce, stage_files,
};
use crate::config_edit::brazen::{Applied, BrazenEditor, RealBzRunner};
use crate::config_edit::litany_global::{Editor, Saved};
use crate::opslog::Origin;
use crate::xdg::Env;
use std::path::{Path, PathBuf};

use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;

/// One mutating §9 config gesture — the write family's carrier.
///
/// **A proposal settle is a config write and belongs here** (bl-dd88): accept
/// fast-forwards a `config/*` lineage onto a staged commit, which is the same
/// subject `Apply` writes by another route. Nothing about it is a fifth kind of
/// act; it is the operator's veto on the learning loop, and the loop's whole
/// design turns on that veto being reachable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Write {
    /// One §9 config apply, carrying the **full staged text** (bl-3f46): the
    /// destination decides the pipeline it goes through, so the four config
    /// editors are one gesture rather than four
    /// ([`ConfigFile`](super::ConfigFile)).
    Apply {
        file: super::ConfigFile,
        text: String,
    },
    /// **Amend an agent's own tracking branch** (§16.3, the per-agent ruling):
    /// point `workspace`'s balls space at `branch`. The
    /// launched-then-told-to-work-on-a-project case, and the same verb a launch
    /// spends — clause 2 and clause 4 are one gesture differing only in when it
    /// fires. It writes balls' own layer-2 config key in that space and stores
    /// nothing of yog's own shape; the reply is the branch **re-read** after the
    /// write.
    Marks { workspace: String, branch: String },
    /// The §9.4 model pick: give `role` this `model` on this provider row, for
    /// `workspace`. **One gesture, one file** since bl-d9cb: litany retired the
    /// cross-check that made this §9.2 and §9.3 composed, so the role assignment
    /// is the whole binding and `providers.yaml` is the only thing written.
    Pick {
        workspace: String,
        role: String,
        provider: String,
        model: String,
    },
    /// **The §9.4 tuning pair** (bl-23bd): a role's reasoning-effort level and
    /// its priority-lane request, the two optional fields of the same assignment
    /// [`Pick`](Self::Pick) writes (litany ARCH §4.3, upstream bl-acba and
    /// bl-f587). One member over
    /// [`model_pick::Tuning`](crate::model_pick::Tuning) rather than two here —
    /// that type's own doc says why each is a separate gesture rather than a
    /// wider `/model`, and why `off` is a removed line rather than a written
    /// `false`.
    Tune(crate::model_pick::Tuning),
    /// **Settle a staged proposal** (§9.6, bl-dd88) — one member over
    /// [`Settle`](crate::proposals::Settle), whose own doc carries the accept's
    /// compare-and-swap and why a stale one is rejected rather than merged.
    Proposal(crate::proposals::Settle),
}

impl Write {
    /// The workspace slot REMOTE §8.2's name→path rewrite borrows — [`Read`'s
    /// own](super::Read::workspace_slot) shape and `Option` for its reason:
    /// [`Apply`](Self::Apply) names a *destination* whose workspace is nested
    /// and itself optional (a §9 file may be the engine's own), and widening the
    /// carrier to match keeps one rule where there would otherwise be a table
    /// arm that knows about one member.
    pub(crate) fn workspace_slot(&mut self) -> Option<&mut String> {
        match self {
            Self::Apply { file, .. } => file.workspace_slot(),
            Self::Marks { workspace, .. } | Self::Pick { workspace, .. } => Some(workspace),
            Self::Tune(tuning) => Some(tuning.workspace_slot()),
            Self::Proposal(settle) => Some(settle.workspace_slot()),
        }
    }
}

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
    let file = paths.config.clone();
    let io = RealFileIo;
    let mut editor = BrazenEditor::load(paths, &io).map_err(|e| named(&file, &e))?;
    editor.set_draft(text.to_owned());
    applied(editor.apply(
        &RealBzRunner::resolve(&super::wall_env(deps, workspace)),
        &io,
    ))?;
    Ok(Reply::Applied)
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

/// The §9.2 pipeline over any plain config file: the shared hash-guard + atomic
/// rename, and nothing else. It judged the draft's `models.<id>.provider` fields
/// until bl-3ffa; that gate is retired with the field's last reader, so a
/// deposit's bytes are the operator's — the same risk the §11 pane's raw editor
/// carries, said once for both faces.
pub(super) fn write_file(dest: PathBuf, text: &str) -> Result<Reply, String> {
    let mut editor = editor_at(&dest)?;
    editor.set_draft(text.to_owned());
    saved(editor.apply(&RealFileIo))?;
    Ok(Reply::Applied)
}

/// Load one §9.2 editor — the single read every file apply enters through.
pub(super) fn editor_at(dest: &Path) -> Result<Editor, String> {
    Editor::load(dest.to_path_buf(), &RealFileIo).map_err(|e| named(dest, &e))
}

/// An IO fault at a config destination, **naming the file** (bl-8c06) — the
/// load half of the sentence [`Draft::io_fault`](crate::config_edit) spells for
/// the apply half. A bare [`std::io::Error`] Display names neither the file nor
/// the act, and every other refusal on this surface names its subject.
fn named(dest: &Path, e: &std::io::Error) -> String {
    format!("{}: {e}", dest.display())
}

/// Fold a §9.2 Apply outcome into the boundary's verdict: refusals are the
/// `Err` beside the reply, exactly as a gate refusal is anywhere else.
pub(super) fn saved(saved: Saved) -> Result<(), String> {
    match saved {
        Saved::Ok => Ok(()),
        Saved::Conflict => Err(CONFLICT.to_owned()),
        Saved::Io { error } => Err(error),
    }
}

/// yog's own clock file (§7.2) under the world's state root.
pub(super) fn cadence_path(world: &Env) -> PathBuf {
    let root = world.yog_state_root();
    root.join(crate::app::cadence::CADENCE_YAML)
}

/// The §9.3 write: stage the text under a fresh nonce and drive `litany config`
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
        &deps.litany,
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
