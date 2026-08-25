//! The bodies the [`dispatch`](super::dispatch) table's arms call — split off
//! at §12's budget (bl-c088) on the seam the chokepoint's own doc already
//! draws: everything left in `dispatch` is the resolutions and the `Action`
//! table, and each of these is a body that stands *beside* the table because
//! its arm is a call rather than table work.
//!
//! They are one subject: the fold from an executor's `io::Result` onto a
//! [`Reply`], said once per shape so the table above stays rows.

use crate::actions::verbs;
use crate::ui_state::UiState;

use super::super::reply::Reply;
use super::super::{answer, control};
use super::{Deps, prepare};

/// One of the three one-shape §8.2 `bl` verbs — project, id, `--as` name —
/// routed by the function that spells it. A bare `fn` pointer, not a generic:
/// the table's three rows are one body with one instantiation, exactly as
/// [`crate::binding`]'s classifier is, and three copies of it would be three
/// places for the §3.2 identity rider to drift.
pub(super) fn spend(
    verb: fn(
        &crate::cli_outbound::Cli,
        &std::path::Path,
        &str,
        &std::path::Path,
        &str,
        &str,
    ) -> std::io::Result<verbs::Outcome>,
    deps: &Deps,
    ts: &str,
    project: &std::path::Path,
    id: &str,
    name: &str,
) -> Result<Reply, String> {
    outcome(verb(&deps.bl, &deps.state_root, ts, project, id, name))
}

/// The §8.1 prepare door as a reply. A body beside the table for the reason
/// [`retarget`] is one: the arm is a call, and the mapping onto a [`Reply`] is
/// not table work — the door itself answers the frame's start glue in the raw
/// [`Prepared`](crate::start::Prepared), which is what that seat needs.
pub(super) fn staged(
    deps: &Deps,
    ts: &str,
    workspace: &std::path::Path,
    repo: &std::path::Path,
    payload: &crate::start::Payload,
) -> Result<Reply, String> {
    prepare(deps, ts, workspace, repo, payload).map(Reply::Prepared)
}

/// A **short verb's** answer: its captured run as a reply, or its launch
/// failure as a refusal. A free function beside its twin [`wrote`] rather than
/// a closure inside the table — the two fold the same shape and there is no
/// reason for one of them to be a body and the other a local.
pub(super) fn outcome(ran: std::io::Result<verbs::Outcome>) -> Result<Reply, String> {
    ran.map(Reply::Outcome).map_err(|e| e.to_string())
}

/// A write-only executor's answer: the reply it earns, or its failure as a
/// refusal. Said once so the trail's two operator verbs stay *rows* in the
/// table above rather than two little bodies inside it.
pub(super) fn wrote(written: std::io::Result<()>, reply: Reply) -> Result<Reply, String> {
    written.map(|()| reply).map_err(|e| e.to_string())
}

/// The §9.4 exit from the config freeze (bl-2d19): mark this conversation to be
/// re-forked onto the config lineage's head, which its own executor lands at
/// the next step boundary. A body beside [`fork`]'s rather than an arm inside
/// the table, because the table is at §12's per-function budget — the routing
/// is one call, and it belongs to the bound family the arms above it do.
pub(super) fn retarget(
    deps: &Deps,
    ts: &str,
    workspace: &std::path::Path,
    agent: &str,
) -> std::io::Result<verbs::Outcome> {
    verbs::retarget(&deps.bound(workspace), &deps.state_root, ts, agent)
}

/// One **attempt** (VISION §5 V2): the §4.11 item-8 confinement refusal, then
/// the fork. A body rather than an arm because a birth is gated, and the
/// chokepoint's match is a table — the second door every drone yog births
/// passes through, beside [`prompt`]'s.
pub(super) fn fork(
    deps: &Deps,
    ts: &str,
    workspace: &std::path::Path,
    parent: &str,
    attempt: &crate::fork::Attempt,
    goal: &str,
) -> Result<Reply, String> {
    control::confinement_gate(workspace)?;
    verbs::fork(
        &deps.bound(workspace),
        &deps.state_root,
        ts,
        &crate::fork::Fire::at(workspace, parent, attempt, goal, &deps.yog_data_root),
    )
    .map(Reply::Outcome)
    .map_err(|e| e.to_string())
}

/// The §6 decision queue's answer (VISION §5 V5.2): write the watermarks the
/// window writes by focusing, then hand back **the queue that remains** —
/// re-derived against the `ui.json` this very call just moved, so one gesture
/// per decision is the whole teleoperator loop. `ts` is the wall clock every
/// boundary caller already mints (§4.2 unix seconds); a clock that states no
/// wall time simply ages every row from zero.
pub(super) fn acknowledge(
    deps: &Deps,
    ui: &mut UiState,
    ts: &str,
    workspace: &std::path::Path,
    agent: &str,
) -> Result<Reply, String> {
    answer::queue::mark_seen(&deps.snapshot, ui, workspace, agent)?;
    let rows = answer::queue::queue(&deps.snapshot, ui, ts.parse().unwrap_or(0));
    Ok(Reply::Attention(rows))
}
