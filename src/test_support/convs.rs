//! **The §11 conversation list, asked the way a seat asks it** (REMOTE §9.7,
//! bl-44e9).
//!
//! `AppModel` holds no conversation accessor any more: the derivation is the
//! engine's (`Query::Conversations` answers the whole descent forest) and the
//! fold is the seat's (`nav::convs::visible`). So a test that wants the list
//! goes through the boundary exactly as the window does, and cannot accidentally
//! assert against a second derivation nobody paints.
//!
//! Its own file for the reason every other fixture here has one: the cap is a
//! tree-wide invariant, and this is a self-contained door rather than part of
//! the spawn discipline.

use std::collections::HashSet;

use crate::AppModel;
use crate::boundary::reply::Reply;
use crate::boundary::{Query, dispatch::Deps};
use crate::cli_outbound::Cli;
use crate::nav::convs::{self, ConvRow};

/// The whole-forest answer for the focused workspace **as a seat holds it** —
/// the boundary's rows with the window's own §3.4 echo folded on, exactly as
/// `shell::convs::forest` does it. Empty for no focus and for a name the
/// snapshot cannot resolve — the refusal a seat would paint, which is no rows
/// either way.
pub(crate) fn forest(model: &AppModel, now_unix: i64) -> Vec<ConvRow> {
    let Some(workspace) = model.focused_ws_name() else {
        return Vec::new();
    };
    let deps: Deps = model.boundary_deps(&Cli::new("lernie"), &Cli::new("bl"));
    let answered = match model.answer(&deps, &Query::Conversations { workspace }, now_unix) {
        Ok(Reply::Conversations(rows)) => rows,
        _ => Vec::new(),
    };
    model.echoed(answered, now_unix)
}

/// The rows a seat holding `expanded` paints — the answer above, folded.
pub(crate) fn visible(model: &AppModel, now_unix: i64, expanded: &HashSet<String>) -> Vec<ConvRow> {
    convs::visible(&forest(model, now_unix), expanded)
}

/// The all-collapsed list: the root subset of the forest, which is what a seat
/// with no viewport at all reads.
pub(crate) fn conversations(model: &AppModel, now_unix: i64) -> Vec<ConvRow> {
    visible(model, now_unix, &HashSet::new())
}
