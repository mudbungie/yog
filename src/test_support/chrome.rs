//! **The §11 accessories, asked the way a seat asks them** (REMOTE §9.7,
//! bl-296f) — [`convs`](super::convs)' door, for the surfaces that crossed with
//! this ball.
//!
//! `AppModel` holds none of these any more. The altitude-0 chrome is a fold
//! over `Query::Workspaces` (which gained the §4.1 pin rank) and `Query::Ops`;
//! the live mark and the in-flight strip are fields on `Query::Agent`'s answer.
//! So a test asks the boundary exactly as the window does, and cannot assert
//! against a second derivation nobody paints.
//!
//! Its own file for the reason every other fixture here has one: the cap is a
//! tree-wide invariant, and this is a self-contained door rather than part of
//! the spawn discipline.

use std::path::Path;

use crate::AppModel;
use crate::boundary::answer::agent::AgentView;
use crate::boundary::reply::{Reply, WsRow};
use crate::boundary::{Query, dispatch::Deps};
use crate::cli_outbound::Cli;
use crate::nav::tabs::TabBar;
use crate::opslog::{Activity, OPS_TAIL};

/// One answered query, through the model's own chokepoint. A refusal reads as
/// the resting `None` a seat would paint nothing for.
fn ask(model: &AppModel, query: &Query, now_unix: i64) -> Option<Reply> {
    let deps: Deps = model.boundary_deps(&Cli::new("litany"), &Cli::new("bl"));
    model.answer(&deps, query, now_unix).ok()
}

/// The enumerated workspaces with their §6 rollups and §4.1 pin ranks — the one
/// answer both altitude-0 surfaces below are folded from.
pub(crate) fn ws_rows(model: &AppModel) -> Vec<WsRow> {
    // With the §3.4 raise claim folded on, exactly as `shell::chrome` does it.
    // The listing itself comes off `AppModel::ws_listing`, which is the one
    // call `answer` makes for this query: it names no workspace and reads no
    // world, so unlike the reads below it has no refusal to stand in for.
    model.raised_rows(model.ws_listing().rows)
}

/// The §7.2 instrumentation the same answer carries (bl-b4b5): how stale the
/// derivation behind it is, and what grew in it — the two lines the §11
/// activity accessory paints above its chip.
pub(crate) fn notes(model: &AppModel) -> (Option<String>, Option<String>) {
    let view = model.ws_listing();
    (view.stale, view.growth)
}

/// One workspace's bound balls with their §3.5 figures — the §11 balls
/// section's whole content, asked the way a seat asks it.
pub(crate) fn ws_balls(model: &AppModel, ws: &Path) -> Vec<crate::nav::BoundBall> {
    let query = Query::WorkspaceBalls {
        workspace: model.snap.ws_name(ws),
    };
    match ask(model, &query, 0) {
        Some(Reply::WorkspaceBalls(rows)) => rows,
        _ => Vec::new(),
    }
}

/// The §6 attention-strip total, as the top bar folds it.
pub(crate) fn strip_total(model: &AppModel) -> usize {
    crate::nav::tabs::strip_total(&ws_rows(model))
}

/// The §11 workspace tab bar, as the top bar folds it — the focus is the
/// window's own, exactly as the seat passes it.
pub(crate) fn tab_bar(model: &AppModel) -> TabBar {
    crate::nav::tabs::build(&ws_rows(model), model.focused_ws_name().as_deref())
}

/// The §11 activity chip's counts, folded off the same `Query::Ops` answer the
/// expanded trail paints.
pub(crate) fn activity(model: &AppModel) -> Activity {
    match ask(model, &Query::Ops { max: OPS_TAIL }, 0) {
        Some(Reply::Ops(rows)) => crate::opslog::activity(&rows),
        _ => Activity {
            total: 0,
            errors: 0,
            drifts: 0,
        },
    }
}

/// One conversation's own detail — the §6 marks, the live mark's seats and the
/// in-flight strip. `now_unix` is the clock the strip's elapsed segment is
/// stamped against.
pub(crate) fn detail(model: &AppModel, ws: &Path, agent: &str, now_unix: i64) -> Option<AgentView> {
    let query = Query::Agent {
        workspace: model.snap.ws_name(ws),
        agent: agent.to_owned(),
    };
    match ask(model, &query, now_unix) {
        Some(Reply::Agent(view)) => Some(view),
        _ => None,
    }
}
