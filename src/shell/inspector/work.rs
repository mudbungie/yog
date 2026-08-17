//! The §11 Work tab's read (§5.1 #32, §3.9), **over the wire** (REMOTE §1.2
//! and its §9.7 read-path residual; bl-f297, widened by bl-77bc).
//!
//! Coverage-excluded like the rest of `shell/*`: the reading is
//! [`crate::science`]'s (which composes [`crate::workdiff`]'s rows rather than
//! restating them), done at the engine and reached through [`Query::Science`],
//! and what lives here is the asks and the picking of their payloads out of
//! [`Reply`] variants.
//!
//! **The listing is the science answer since bl-77bc** — the fan group card
//! compares by the agent-side columns and the attempt rows drill into the diff
//! rows those same answers carry, so the two surfaces cannot disagree about
//! which attempts exist. What is still [`Query::WorkDiff`]'s is the **picked
//! file's patch**: a per-file question the science projection deliberately does
//! not answer, asked only while a file is picked — so an unpicked tab asks one
//! question, exactly as before.

use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::reply::Reply;
use crate::files_view::Preview;
use crate::keymap::InspectorTab;
use crate::shell::wire::Said;
use crate::workdiff::WorkFile;
use std::path::Path;

/// What the Work tab paints: the workspace's attempts as science reads them,
/// and the picked file's patch. Both empty is the resting state of a tab whose
/// question was asked a moment ago; the engine's sentence, if it refused, goes
/// to the caller's [`Said`] beside every other read's (bl-13f9).
#[derive(Default)]
pub(super) struct Work {
    pub(super) science: Vec<crate::science::Attempt>,
    pub(super) patch: Option<Preview>,
}

/// Declare the tab's questions and read whatever has landed. An inactive tab
/// reads nothing at all — the same rule the Files and Transcript reads keep.
pub(super) fn read(
    tab: InspectorTab,
    model: &mut AppModel,
    ws: &Path,
    picked: Option<WorkFile>,
    said: &mut Said,
) -> Work {
    if tab != InspectorTab::Work {
        return Work::default();
    }
    let science = Query::Science {
        workspace: model.snap.ws_name(ws),
    };
    let landed = crate::shell::wire::ask(model, science, |reply| match reply {
        Reply::Science(rows) => Some(rows),
        _ => None,
    });
    let rows = said.take(landed).unwrap_or_default();
    let patch = picked.and_then(|file| {
        let query = Query::WorkDiff {
            workspace: model.snap.ws_name(ws),
            file: Some(file),
        };
        let landed = crate::shell::wire::ask(model, query, |reply| match reply {
            Reply::WorkDiff { patch, .. } => Some(patch),
            _ => None,
        });
        said.take(landed).flatten()
    });
    Work {
        science: rows,
        patch,
    }
}
