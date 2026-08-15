//! **The §11 inspector family, over the wire** (REMOTE §1.2 and its §9.7
//! read-path residual; bl-13f9) — the seat's half of the seven reads
//! [`answer::inspector`](crate::boundary::answer::inspector) owns.
//!
//! Each is one [`Query`] and one [`Reply`] arm through
//! [`crate::shell::wire::ask`], which is the shell's one spelling of a wire
//! read. Nothing here derives anything: the transcript's live tail, the spine's
//! child cards, the worktree listing and the config freeze are all folded at
//! the engine, so the window paints a decoded answer exactly as a phone seat
//! would (§8.5's parity discipline reaching the last surface that had escaped
//! it).
//!
//! **The memos came out with the accessors.** The four per-snapshot
//! [`SnapMemo`](crate::app::SnapMemo) slots — transcript, steps, rail, files —
//! existed because each build read disk on the paint path; an answer *is* a
//! cached fold, refreshed at the asker's human cadence
//! ([`ASK_PERIOD`](crate::wire::asker::ASK_PERIOD)), so there is nothing left
//! to memoize and no key to keep in step. That is the Work tab's own trade
//! (bl-f297) taken across the family.
//!
//! **A question nobody asks is not asked**: an inactive tab simply does not
//! call, and its answer is dropped at the next settle — the collapsed-pane
//! rule. Two of these are asked on *every* tab, and deliberately: the steps
//! view feeds the centre's auth and wound banners as well as its own tab, and
//! the transcript feeds the spine (a notch's seat is a row in the chat), which
//! is the pin every pinnable tab reads through.
//!
//! Coverage-excluded glue like the rest of `shell/*`.

use std::path::Path;

use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::reply::Reply;
use crate::config_edit::branch::GoverningConfig;
use crate::files_view::{FilesView, Preview};
use crate::inboxview::InboxEntry;
use crate::rail::Rail;
use crate::shell::wire::{Landed, ask};
use crate::steps_view::{StepDetail, StepsView};
use crate::transcript::Transcript;

/// The address every one of these carries: the workspace's §3.1 name and the
/// conversation inside it — the same two halves the envelope requires, read
/// back off the snapshot's own mapping (REMOTE §8's two directions).
fn at(model: &AppModel, ws: &Path, agent: &str) -> (String, String) {
    (model.snap.ws_name(ws), agent.to_owned())
}

/// **The conversation** — the committed `messages/` entries with the in-flight
/// tail folded on by the engine (bl-6233's ruling, unmoved).
///
/// The tail now moves at the asker's cadence rather than the derivation's,
/// which is what a migrated read costs and is stated in REMOTE §9.7: half a
/// second of streamed text arrives as one row rather than as characters. It is
/// still the *same* fold, so the two seats cannot describe one moment
/// differently — which was always the point of putting it at the boundary.
pub(in crate::shell) fn transcript(
    model: &mut AppModel,
    ws: &Path,
    agent: &str,
) -> Landed<Transcript> {
    let (workspace, agent) = at(model, ws, agent);
    ask(
        model,
        Query::Transcript { workspace, agent },
        |reply| match reply {
            Reply::Transcript(tx) => Some(tx),
            _ => None,
        },
    )
}

/// **Every step the conversation has taken** — the Steps tab's list, and the
/// centre's auth/wound banners' input. Its liveness is the engine's read off
/// the published snapshot, never a parameter this seat could contradict.
pub(in crate::shell) fn steps(model: &mut AppModel, ws: &Path, agent: &str) -> Landed<StepsView> {
    let (workspace, agent) = at(model, ws, agent);
    ask(
        model,
        Query::Steps { workspace, agent },
        |reply| match reply {
            Reply::Steps(view) => Some(view),
            _ => None,
        },
    )
}

/// **One step's records** — the drill-in, named by the sequence the list
/// answered. Two questions rather than one, unlike Files: the list is standing
/// anyway, so the seat resolves its row index against the landed list in the
/// same frame and the detail lands one period later.
pub(super) fn detail(
    model: &mut AppModel,
    ws: &Path,
    agent: &str,
    seq: &str,
) -> Landed<StepDetail> {
    let (workspace, agent) = at(model, ws, agent);
    let seq = seq.to_owned();
    ask(
        model,
        Query::Step {
            workspace,
            agent,
            seq,
        },
        |reply| match reply {
            Reply::Step(detail) => Some(detail),
            _ => None,
        },
    )
}

/// **The worktree listing and the picked file's bytes, in one question** — the
/// two arms bl-6233's containment rule left the window branching on, dissolved
/// (REMOTE §9.7's own residual). `at` is the pin's commit, which is a
/// *selection* naming which tree, and `path` is the selection the last answer's
/// listing made nameable.
pub(super) fn files(
    model: &mut AppModel,
    ws: &Path,
    agent: &str,
    path: Option<String>,
    commit: Option<String>,
) -> Landed<(FilesView, Option<Preview>)> {
    let (workspace, agent) = at(model, ws, agent);
    ask(
        model,
        Query::Files {
            workspace,
            agent,
            path,
            at: commit,
        },
        |reply| match reply {
            Reply::Files { view, preview } => Some((view, preview)),
            _ => None,
        },
    )
}

/// **The step spine** (VISION V1) — the notches and the child cards, each notch
/// carrying its budget as a rollup since bl-44e9, so the pin the seat resolves
/// against it derives nothing.
pub(super) fn rail(model: &mut AppModel, ws: &Path, agent: &str) -> Landed<Rail> {
    let (workspace, agent) = at(model, ws, agent);
    ask(
        model,
        Query::Rail { workspace, agent },
        |reply| match reply {
            Reply::Rail(rail) => Some(rail),
            _ => None,
        },
    )
}

/// **The undelivered mail** (§11 Inbox).
pub(super) fn inbox(model: &mut AppModel, ws: &Path, agent: &str) -> Landed<Vec<InboxEntry>> {
    let (workspace, agent) = at(model, ws, agent);
    ask(
        model,
        Query::Inbox { workspace, agent },
        |reply| match reply {
            Reply::Inbox(entries) => Some(entries),
            _ => None,
        },
    )
}

/// **Config-frozen-at** (VISION V1.2, bl-13f9) — the policy this conversation
/// runs, at the pinned commit or at its own tip. `commit` absent is the tip,
/// resolved by the engine off the snapshot, so the unpinned Config tab asks the
/// question a headless `/governing` asks and the pinned one asks `--at`.
pub(super) fn governing(
    model: &mut AppModel,
    ws: &Path,
    agent: &str,
    commit: Option<String>,
) -> Landed<GoverningConfig> {
    let (workspace, agent) = at(model, ws, agent);
    ask(
        model,
        Query::Governing {
            workspace,
            agent,
            at: commit,
        },
        |reply| match reply {
            Reply::Governing(gov) => Some(gov),
            _ => None,
        },
    )
}
