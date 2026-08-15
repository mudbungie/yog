//! The §11 Work tab's read (§5.1 #32), **over the wire** (REMOTE §1.2 and its
//! §9.7 read-path residual; bl-f297).
//!
//! Coverage-excluded like the rest of `shell/*`: the reading is
//! [`crate::workdiff`]'s, done at the engine and reached through
//! [`Query::WorkDiff`], and what lives here is one ask and the picking of its
//! payload out of one [`Reply`] variant.
//!
//! **The two memos went with the migration.** Every arm of this read forks
//! `git` against a *project* repo, which is why the in-process version ran at
//! most once per published snapshot (§7.2 `SnapMemo`) — but an answer *is* the
//! cached fold, refreshed at the asker's human cadence rather than per
//! derivation, so there is nothing left to memoize and no key to keep in step.
//! That is the same trade the §11 balls fold and the ops trail already made
//! (bl-adcb).
//!
//! **One question, not two.** The listing and the picked file's patch were two
//! reads with two memo keys; `Query::WorkDiff` carries the file and answers
//! both, so re-picking asks a different question and scrolling asks nothing.
//! An inactive tab declares no question at all, and the asker drops the answer
//! at the next settle — the same rule that makes a collapsed pane free.

use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::reply::Reply;
use crate::files_view::Preview;
use crate::keymap::InspectorTab;
use crate::workdiff::{Attempt, WorkFile};
use std::path::Path;

/// What the Work tab paints: the workspace's attempts, the picked file's patch,
/// and the engine's sentence if it refused. All three empty is the resting
/// state of a tab whose question was asked a moment ago.
#[derive(Default)]
pub(super) struct Work {
    pub(super) attempts: Vec<Attempt>,
    pub(super) patch: Option<Preview>,
    pub(super) refused: Option<String>,
}

/// Declare the tab's question and read whatever has landed. An inactive tab
/// reads nothing at all — the same rule the Files and Transcript builds keep.
pub(super) fn read(
    tab: InspectorTab,
    model: &mut AppModel,
    ws: &Path,
    picked: Option<WorkFile>,
) -> Work {
    if tab != InspectorTab::Work {
        return Work::default();
    }
    let query = Query::WorkDiff {
        workspace: model.snap.ws_name(ws),
        file: picked,
    };
    let landed = crate::shell::wire::ask(model, query, |reply| match reply {
        Reply::WorkDiff { attempts, patch } => Some((attempts, patch)),
        _ => None,
    });
    let (attempts, patch) = landed.value.unwrap_or_default();
    Work {
        attempts,
        patch,
        refused: landed.refused,
    }
}
