//! **The §11 reads, asked the way a seat asks them** (REMOTE §9.7), split out
//! per the 300-line cap.
//!
//! Every helper here is one shape: put a `boundary::Query` to the engine, take
//! the reply, and hand it to the same `nav::*` fold the window paints from.
//! `AppModel` carries no conversation, tab-bar, strip or ball accessor any
//! more — the derivation is the engine's and the fold is the seat's — so a
//! story test asks the boundary the question the window asks it, and cannot
//! assert against a second derivation nobody paints.
//!
//! That is why they are a tier and not a fixture: `mod.rs` beside this one
//! WRITES the world a story runs over (recorder binaries, workspaces on disk,
//! a hand-driven clock), and these only READ what the engine says about it.

use std::collections::HashSet;

/// **The §11 conversation list, asked the way a seat asks it** (bl-44e9).
///
/// Empty for no focus and for a workspace name the snapshot cannot resolve —
/// the refusal a seat would paint, which is no rows either way.
pub fn conversations(
    model: &yog::AppModel,
    now_unix: i64,
    expanded: &HashSet<String>,
) -> Vec<yog::nav::convs::ConvRow> {
    let Some(workspace) = model.focused_ws_name() else {
        return Vec::new();
    };
    let deps = model.boundary_deps(
        &yog::cli_outbound::Cli::new("litany"),
        &yog::cli_outbound::Cli::new("bl"),
    );
    let query = yog::boundary::Query::Conversations { workspace };
    match model.answer(&deps, &query, now_unix) {
        Ok(yog::boundary::reply::Reply::Conversations(rows)) => {
            yog::nav::convs::visible(&rows, expanded)
        }
        _ => Vec::new(),
    }
}

/// The all-collapsed list: the root subset of the forest above, which is what a
/// seat holding no viewport reads.
pub fn conversation_rows(model: &yog::AppModel, now_unix: i64) -> Vec<yog::nav::convs::ConvRow> {
    conversations(model, now_unix, &HashSet::new())
}

/// **The §11 altitude-0 chrome, asked the way the top bar asks it** (bl-296f) —
/// the answered workspace rows with the §3.4 raise claim folded on, which is
/// exactly what `shell::top_bar` paints from.
///
/// The derivation is the engine's (`Query::Workspaces`, carrying the §6 rollups
/// and the §4.1 pin rank) and both folds are the seat's.
pub fn ws_rows(model: &yog::AppModel) -> Vec<yog::boundary::reply::WsRow> {
    let deps = model.boundary_deps(
        &yog::cli_outbound::Cli::new("litany"),
        &yog::cli_outbound::Cli::new("bl"),
    );
    let answered = match model.answer(&deps, &yog::boundary::Query::Workspaces, 0) {
        Ok(yog::boundary::reply::Reply::Workspaces(view)) => view.rows,
        _ => Vec::new(),
    };
    model.raised_rows(answered)
}

/// One workspace's bound balls with their §3.5 figures — the §11 balls
/// section's whole content (`Query::WorkspaceBalls`, bl-b4b5), asked the way a
/// seat asks it.
pub fn ws_balls(model: &yog::AppModel, ws: &std::path::Path) -> Vec<yog::nav::BoundBall> {
    let deps = model.boundary_deps(
        &yog::cli_outbound::Cli::new("litany"),
        &yog::cli_outbound::Cli::new("bl"),
    );
    let query = yog::boundary::Query::WorkspaceBalls {
        workspace: yog::naming::leaf(ws),
    };
    match model.answer(&deps, &query, 0) {
        Ok(yog::boundary::reply::Reply::WorkspaceBalls(rows)) => rows,
        _ => Vec::new(),
    }
}

/// The §6 attention-strip total, as the top bar folds it.
pub fn strip_total(model: &yog::AppModel) -> usize {
    yog::nav::tabs::strip_total(&ws_rows(model))
}

/// The §11 workspace tab bar, as the top bar folds it.
pub fn tab_bar(model: &yog::AppModel) -> yog::nav::tabs::TabBar {
    yog::nav::tabs::build(&ws_rows(model), model.focused_ws_name().as_deref())
}

/// **The §11 header's conversation ball, as the seat folds it** (bl-296f): the
/// `ConvBall` the answered forest already carries on the root's row, picked out
/// by `nav::convs::selection`. There is no model accessor — the header reads
/// `Selection::ball` off the list it is already painting.
pub fn conversation_ball(
    model: &yog::AppModel,
    root_id: &str,
    now_unix: i64,
) -> Option<yog::nav::convs::ConvBall> {
    yog::nav::convs::selection(&conversation_rows(model, now_unix), root_id).ball
}
