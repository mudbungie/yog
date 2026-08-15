//! **The §11 conversation list, read over the wire** (REMOTE §1.2, §9.7;
//! bl-44e9) — the shell's one ask for it, and the seat's own fold over what
//! landed.
//!
//! The list used to be an `AppModel` accessor that took the viewport's expanded
//! set and handed back the rows it made visible. Both halves moved, in opposite
//! directions, and that is the altitude ruling made structural:
//!
//! - the **derivation** went out to the boundary, which now answers the whole
//!   descent forest with its per-row rollups (`Query::Conversations`); and
//! - the **fold** stayed here, where a viewport lives — [`nav::convs::visible`]
//!   is a pure selection out of an answer, so DESIGN §8.5's *views gain no
//!   boundary representation* is kept to the letter: no expanded set crosses,
//!   and no row carries one.
//!
//! Everything that reads the list reads it through here — the paint, the ↑/↓
//! walk, the ← that pages to a parent — so the rows the operator sees, the rows
//! the keyboard steps and the rows a fold hides can never be three answers.
//! Asking twice in one frame is asking once ([`link`](crate::wire::link): a
//! standing question is keyed by its own envelope), so a second call inside a
//! frame costs an encode and a map read.
//!
//! Coverage-excluded glue like the rest of `shell/*`: the ask is
//! [`super::wire::ask`]'s one shape and the fold is `nav`'s tested derivation.

use std::collections::HashSet;

use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::reply::Reply;
use crate::nav::convs::{self, ConvRow};

use super::wire::{self, Landed};

/// The focused workspace's whole descent forest, as the wire answered it, with
/// this window's own §3.4 echo folded on ([`AppModel::echoed`]). Nothing focused
/// is nothing asked — the resting state, not a refusal.
///
/// The echo rides here rather than at the paint because *every* reader below
/// must see the same list: a start's pending row is a row the ↓ key steps and
/// the ← key pages out of, not a decoration the paint adds.
pub(super) fn forest(model: &mut AppModel) -> Landed<Vec<ConvRow>> {
    let Some(workspace) = model.focused_ws_name() else {
        return Landed::default();
    };
    let mut landed = of(model, workspace);
    landed.value = landed
        .value
        .map(|rows| model.echoed(rows, super::now_unix()));
    landed
}

/// The descent forest of a **named** workspace, which need not be the focused
/// one: the §3.6 delete dialog is opened from a tab menu and confirms whatever
/// wall that menu named (bl-b4b5). No echo — the echo is this window's optimism
/// about its own next start, and a dialog about another wall is not that seat.
///
/// Aimed at the focused workspace it is the *same envelope* [`forest`] declares,
/// so the two are one ask.
pub(super) fn of(model: &mut AppModel, workspace: String) -> Landed<Vec<ConvRow>> {
    wire::ask(
        model,
        Query::Conversations { workspace },
        |reply| match reply {
            Reply::Conversations(rows) => Some(rows),
            _ => None,
        },
    )
}

/// The rows this seat's fold makes visible — the one derivation every gesture
/// and the paint itself read, so the walk, the fold and the glass can never
/// disagree about which rows exist. An unanswered wire has no rows, which is the
/// honest empty state and not an empty workspace.
pub(super) fn visible(model: &mut AppModel, expanded: &HashSet<String>) -> Vec<ConvRow> {
    convs::visible(&forest(model).value.unwrap_or_default(), expanded)
}
