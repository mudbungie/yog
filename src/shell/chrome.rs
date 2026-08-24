//! **The altitude-0 answer, and the §11 balls listing** (REMOTE §9.7, bl-296f,
//! bl-b4b5) — the shell's two remaining standing questions about a *workspace*,
//! beside [`super::convs`]' one about a conversation list.
//!
//! Both are asked here rather than at each surface for [`super::convs`]' own
//! reason: a standing question is keyed by its own encoded envelope, so the tab
//! bar, the attention strip, the §3.6 scope gate and the §9.4 picker's lineage
//! tip are **one ask** however many of them paint. What
//! each surface then does is a pure selection out of the landed rows
//! ([`nav::tabs`], [`nav::balls`]), which is what keeps two carriers of one
//! affordance from offering different things.
//!
//! Coverage-excluded glue like the rest of `shell/*`: every ask is
//! [`super::wire::ask`]'s one shape and every fold is `nav`'s tested derivation.

use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::reply::{Reply, WsRow};
use crate::nav::BoundBall;

use super::wire::{self, Landed};

/// The whole `Query::Workspaces` answer — the enumeration and the §7.2 notes
/// about how current it is. A frame the engine has not answered holds none of
/// it, which is the honest empty state.
pub(super) fn workspaces(model: &mut AppModel) -> Landed<crate::boundary::reply::Workspaces> {
    wire::ask(model, Query::Workspaces, |reply| match reply {
        Reply::Workspaces(view) => Some(view),
        _ => None,
    })
}

/// The enumerated rows with this window's own §3.4 raise claim folded on
/// ([`AppModel::raised_rows`]) — what every altitude-0 fold reads. The claim
/// rides here rather than at the paint for the echo's reason: a wall `lernie
/// new` has just founded must wear its tab and resolve its name from the frame
/// the receipt lands, not one derivation later.
pub(super) fn ws_rows(model: &mut AppModel) -> Vec<WsRow> {
    // **The union, not one channel's answer** (REMOTE §8.2, bl-028a): every
    // channel's slice, each row already named in this box's spelling. With zero
    // entries it is the `workspaces` answer above, row for row.
    let answered = model.wire_roster().into_iter().map(|r| r.row).collect();
    model.raised_rows(answered)
}

/// **Which §8.2 entry hosts `name`**, or `None` for this window's own engine
/// (bl-1fd0) — a selection out of the union roster this frame already holds,
/// exactly as the tab bar's is, so it costs no ask of its own.
///
/// The origin is dropped by [`ws_rows`] above, which is right for every seat
/// that paints a workspace *row*: a row is a row wherever it came from. The
/// §8.1 provider gate is the one reader for which the channel IS the question,
/// because the wall it must judge lives on the far side of it.
pub(super) fn ws_channel(model: &mut AppModel, name: &str) -> Option<String> {
    model
        .wire_roster()
        .into_iter()
        .find(|r| r.row.workspace == name)
        .and_then(|r| r.origin.label())
}

/// One workspace's bound balls with their §3.5 figures — the §11 balls
/// section's whole content, and the object its row menus act on. Nothing
/// focused is nothing asked.
pub(super) fn balls(model: &mut AppModel, workspace: &str) -> Landed<Vec<BoundBall>> {
    wire::ask(
        model,
        Query::WorkspaceBalls {
            workspace: workspace.to_owned(),
        },
        |reply| match reply {
            Reply::WorkspaceBalls(rows) => Some(rows),
            _ => None,
        },
    )
}

/// The focused workspace's bound balls, or nothing when no workspace is
/// focused — the shape every §11 balls surface takes.
pub(super) fn focused_balls(model: &mut AppModel) -> Vec<BoundBall> {
    let Some(workspace) = model.focused_ws_name() else {
        return Vec::new();
    };
    balls(model, &workspace).value.unwrap_or_default()
}
