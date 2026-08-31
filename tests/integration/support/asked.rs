//! **The §11 reads, asked the way a seat asks them** (REMOTE §9.7), split out
//! per the 300-line cap.
//!
//! Every helper here is one shape: put a `boundary::Query` to the engine, take
//! the reply, and hand it to the same `nav::*` fold a seat renders from. The
//! derivation is the engine's and the fold is the seat's, so a story asks the
//! boundary the question a seat asks it and cannot assert against a second
//! derivation nobody renders.
//!
//! **Every helper names its workspace** (bl-7942). They read it off the model's
//! focus until the severance; a server holds no focus — which workspace a seat
//! is looking at is the seat's own (REMOTE §7) and rides in the gesture — so
//! the story states it, exactly as a gesture does.
//!
//! That is why they are a tier and not a fixture: `mod.rs` beside this one
//! WRITES the world a story runs over (recorder binaries, workspaces on disk,
//! a hand-driven clock), and these only READ what the engine says about it.

use std::collections::HashSet;

/// **The §11 conversation list, asked the way a seat asks it** (bl-44e9).
///
/// Empty for a workspace name the snapshot cannot resolve — the refusal a seat
/// would render, which is no rows.
pub fn conversations(
    model: &yog::AppModel,
    workspace: &str,
    now_unix: i64,
    expanded: &HashSet<String>,
) -> Vec<yog::nav::convs::ConvRow> {
    let workspace = workspace.to_owned();
    let query = yog::boundary::Query::Conversations { workspace };
    match ask(model, &query, now_unix) {
        Ok(yog::boundary::reply::Reply::Conversations(rows)) => {
            yog::nav::convs::visible(&rows, expanded)
        }
        _ => Vec::new(),
    }
}

/// The all-collapsed list: the root subset of the forest above, which is what a
/// seat holding no viewport reads.
pub fn conversation_rows(
    model: &yog::AppModel,
    workspace: &str,
    now_unix: i64,
) -> Vec<yog::nav::convs::ConvRow> {
    conversations(model, workspace, now_unix, &HashSet::new())
}

/// **The altitude-0 chrome, asked the way a roster asks it** (bl-296f) — the
/// answered workspace rows, carrying the §6 rollups and the §4.1 pin rank.
pub fn ws_rows(model: &yog::AppModel) -> Vec<yog::boundary::reply::WsRow> {
    match ask(model, &yog::boundary::Query::Workspaces, 0) {
        Ok(yog::boundary::reply::Reply::Workspaces(view)) => view.rows,
        _ => Vec::new(),
    }
}

/// One workspace's bound balls with their §3.5 figures — the §11 balls
/// section's whole content (`Query::WorkspaceBalls`, bl-b4b5), asked the way a
/// seat asks it.
pub fn ws_balls(model: &yog::AppModel, ws: &std::path::Path) -> Vec<yog::nav::BoundBall> {
    let query = yog::boundary::Query::WorkspaceBalls {
        workspace: yog::naming::leaf(ws),
    };
    match ask(model, &query, 0) {
        Ok(yog::boundary::reply::Reply::WorkspaceBalls(rows)) => rows,
        _ => Vec::new(),
    }
}

/// The §6 attention-strip total, as the top bar folds it.
pub fn strip_total(model: &yog::AppModel) -> usize {
    yog::nav::tabs::strip_total(&ws_rows(model))
}

/// The workspace tab bar, as a roster folds it over the workspace the seat
/// states it is looking at.
pub fn tab_bar(model: &yog::AppModel, focused: Option<&str>) -> yog::nav::tabs::TabBar {
    yog::nav::tabs::build(&ws_rows(model), focused)
}

/// **The §11 header's conversation ball, as the seat folds it** (bl-296f): the
/// `ConvBall` the answered forest already carries on the root's row, picked out
/// by `nav::convs::selection`. There is no model accessor — the header reads
/// `Selection::ball` off the list it is already painting.
pub fn conversation_ball(
    model: &yog::AppModel,
    workspace: &str,
    root_id: &str,
    now_unix: i64,
) -> Option<yog::nav::convs::ConvBall> {
    yog::nav::convs::selection(&conversation_rows(model, workspace, now_unix), root_id).ball
}

/// **One act, run the way the engine runs it** — the model's own `Deps`, its
/// own `ui.json`, and `boundary::dispatch`. A story that used to reach into a
/// frame's click-glue fires the gesture instead (REMOTE §1.2): there is one
/// pipeline, and this is it.
pub fn act(
    model: &yog::AppModel,
    action: &yog::boundary::Action,
) -> Result<yog::boundary::reply::Reply, String> {
    let deps = model.boundary_deps(
        &yog::cli_outbound::Cli::new("litany"),
        &yog::cli_outbound::Cli::new("bl"),
    );
    let mut ui = yog::ui_state::UiState::open(model.ui_json_path());
    yog::boundary::dispatch::dispatch(&deps, &mut ui, "0", action)
}

/// **One query, answered the way the engine answers it** — `ui.json` opened
/// per gesture, exactly as `ConsumerCtx::answer` opens it. That is not a
/// detail: a durable UI fact an act just wrote (a `seen` watermark) is on disk,
/// and a reader holding a stale in-RAM document would answer as if it were not.
pub fn ask(
    model: &yog::AppModel,
    query: &yog::boundary::Query,
    now_unix: i64,
) -> Result<yog::boundary::reply::Reply, String> {
    let deps = model.boundary_deps(
        &yog::cli_outbound::Cli::new("litany"),
        &yog::cli_outbound::Cli::new("bl"),
    );
    let ui = yog::ui_state::UiState::open(model.ui_json_path());
    yog::boundary::answer::answer(query, &deps, &ui, now_unix)
}
