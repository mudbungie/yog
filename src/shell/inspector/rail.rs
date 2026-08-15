//! The step spine's glue (VISION V1): gather the focused agent's children off
//! the snapshot, fold each one's spend, and hand [`crate::rail::build`] what it
//! needs — the transcript included, since bl-1802, because a notch's seat is a
//! row in the chat — then apply the operator's pin to the tab builds.
//!
//! Coverage-excluded like the rest of `shell/*`: every decision here is a
//! call into a tested module ([`children_of`], [`crate::rail::build`],
//! [`crate::rail::pin`], [`crate::rail::transcript_as_of`],
//! [`crate::rail::files_at`]). Split out of `inspector/mod.rs` at §12's
//! per-file budget for the shell.

use std::path::Path;
use std::sync::Arc;

use crate::AppModel;
use crate::boundary::answer::inspector;
use crate::config_edit::branch::governing_config;
use crate::files_view::{self, FilesView, Preview};
use crate::rail::{self, Pin, Rail};
use crate::steps_view::StepsView;
use crate::transcript::Transcript;

use super::super::InspectorState;

/// Build (or re-read the memo for) the focused agent's spine. `tx` is the
/// **live** transcript, never the pinned cut: a notch's seat is a row key, and
/// keys are the entry filenames, so a rail derived once against the whole chat
/// answers for the cut one too — the rules past the cut simply match no row.
pub fn build(
    model: &AppModel,
    inspector_state: &mut InspectorState,
    ws: &Path,
    agent_id: &str,
    steps: &StepsView,
    tx: &Transcript,
) -> Rail {
    let snap = Arc::clone(model.derivation());
    inspector_state
        .rail_memo
        .read(&snap, (ws.to_path_buf(), agent_id.to_owned()), &mut || {
            inspector::rail(&snap, ws, agent_id, steps, tx)
        })
        .clone()
}

/// The operator's pin, resolved against the rail. A selection the rail no
/// longer carries resolves to `None`, which is today's read — a re-derivation
/// that drops steps can never strand the inspector at a notch that is gone.
pub fn pinned(rail: &Rail, notch_sel: Option<usize>) -> Option<Pin> {
    rail::pin(rail, notch_sel)
}

/// The transcript, cut to the pin. Unpinned it is the memoized live build,
/// handed on by pointer; pinned it is that build's prefix as of the notch's
/// read state, which costs one clone of the entries in front of the pin and
/// no disk read at all.
pub fn transcript(live: &Arc<Transcript>, pin: Option<&Pin>) -> Arc<Transcript> {
    match pin {
        None => Arc::clone(live),
        Some(pin) => Arc::new(rail::transcript_as_of(live, pin.cut)),
    }
}

/// The Files listing, live or as of the pin — one memo slot for both, keyed on
/// the commit so a re-pin re-reads and a scroll does not.
///
/// The **branch is the boundary's** since bl-44e9: `Query::Files` carries the
/// tree as a selection, so live-or-pinned is decided once, in
/// [`inspector::files`], and this is the memo around that one answer rather than
/// a second spelling of the same two arms.
pub fn files(
    model: &AppModel,
    inspector: &mut InspectorState,
    ws: &Path,
    agent_id: &str,
    pin: Option<&Pin>,
) -> FilesView {
    let commit = pin.map(|p| p.commit.clone());
    let snap = Arc::clone(model.derivation());
    let (root, id) = (ws.to_path_buf(), agent_id.to_owned());
    inspector
        .files_memo
        .read(&snap, (root, id, commit.clone()), &mut || {
            inspector::files(ws, agent_id, None, commit.as_deref()).0
        })
        .clone()
}

/// The selected file's preview, read from the pinned commit's tree when a pin
/// is up and from the live worktree otherwise.
pub fn preview(ws: &Path, agent_id: &str, rel_path: &str, pin: Option<&Pin>) -> Preview {
    match pin {
        Some(pin) => rail::preview_at(ws, &pin.commit, rel_path),
        None => files_view::preview(&files_view::agent_worktree(ws, agent_id).join(rel_path)),
    }
}

/// The governing config to show: the pinned commit's when a notch is pinned,
/// the agent's tip otherwise. Config-frozen-at needed no new code — it is the
/// same §5.1 #17 fold asked at a different commit.
pub fn governing(
    ws: &Path,
    tip: &str,
    pin: Option<&Pin>,
) -> Option<crate::config_edit::branch::GoverningConfig> {
    let at = pin.map_or(tip, |pin| pin.commit.as_str());
    governing_config(ws, at).ok()
}
