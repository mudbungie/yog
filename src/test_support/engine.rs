//! **Where the engine stands, for a test that drives a gesture** (REMOTE §1.2,
//! §9.8; bl-1747).
//!
//! The window holds no dispatch of its own any more: every act it fires is
//! posted over the wire and executed by the engine, which builds its own
//! [`Deps`](crate::boundary::dispatch::Deps) and opens `ui.json` **fresh** per
//! gesture (`boundary::consumer::ConsumerCtx`). A unit test that used to reach
//! `AppModel::dispatch` was standing in the window's shoes; it stands in the
//! engine's now, and this is the one spelling of that — so a test cannot
//! quietly re-acquire the second execution path §1.2 exists to refuse.
//!
//! The fresh `UiState` is the point, not an incidental: it is answer 3's
//! ordering (*the engine writes and the window adopts*), and a helper that
//! handed out `&mut model.ui` would be testing a code path that no longer
//! exists.

use crate::AppModel;
use crate::boundary::dispatch::Deps;
use crate::boundary::{Action, reply::Reply};
use crate::ui_state::UiState;

/// Run one gesture the way the engine runs it, against `model`'s own world.
pub(crate) fn act(
    model: &AppModel,
    deps: &Deps,
    ts: &str,
    action: &Action,
) -> Result<Reply, String> {
    let mut ui = UiState::open(model.ui_json_path());
    crate::boundary::dispatch::dispatch(deps, &mut ui, ts, action)
}
