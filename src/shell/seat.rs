//! **The §11 selection, as the seat reads it** (REMOTE §9.7, bl-48ae) — the two
//! answers the window's own selection is made of, and the ruling about which of
//! them may land late.
//!
//! `AppModel::focused_conversation` used to answer both at once, in process, off
//! the window's own snapshot: the last read of REMOTE §11's residual and the one
//! bl-13f9 deliberately left, because four of its consumers read it *between the
//! click and the next frame*. It is gone, and what replaced it is a split by
//! **who needs it when** rather than a latch or a held stale answer (the ball's
//! shapes 1 and 2, both declined):
//!
//! - [`selection`] — the facts that name the target or gate a gesture, picked
//!   out of the **landed forest** the §11 list is already holding
//!   ([`super::convs::forest`], `Query::Conversations` since bl-44e9). Nothing
//!   new is asked and nothing is latched: a `ConvRow` has carried the §8.2 gates
//!   since bl-1eb0 and the answer is pre-order, so the composer's name, the
//!   ancestors' unfold, `x`'s `stoppable` and the §3.6 danger row's root are all
//!   *selections* out of an answer that is already on the frame. They change in
//!   the same frame the selection does, which is the whole reason this ball
//!   needed a ruling.
//! - [`detail`] — the selection's own **detail**, a standing `Query::Agent`
//!   answered by `answer::agent`. The config freeze the §9.4 model row reads,
//!   the §6 marks the centre says outright, the §8.6 park and the `Nudge` gate:
//!   facts about one agent rather than about the list, which is why they are not
//!   on a row and must not be pushed onto one.
//!
//! **The rendering ruling, stated: a fact that gates a gesture is read off the
//! forest; a fact that only paints may land an ask period later.** Everything in
//! [`detail`] paints an affordance rather than judging one — an unpainted button
//! cannot be clicked, so nothing here can refuse a gesture the operator just
//! made. What it costs is that a freshly-selected conversation shows its marks,
//! its model row and its `Nudge` button one ask period after its name, which is
//! the same half-second the transcript beside them already pays (bl-13f9) and is
//! not a case of its own.
//!
//! Coverage-excluded glue like the rest of `shell/*`: [`selection`] is `nav`'s
//! tested fold and [`detail`] is [`super::wire::ask`]'s one shape.

use std::path::Path;

use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::answer::agent::AgentView;
use crate::boundary::reply::Reply;
use crate::nav::convs::Selection;

use super::wire::{Landed, ask};

/// The focused conversation as the frame must know it *now*. `None` with
/// nothing selected — the resting state, and the one branch every seat below
/// takes; an id the answered forest does not carry is a `Some` whose fields say
/// so, exactly as the boundary answers it.
pub(super) fn selection(model: &mut AppModel) -> Option<Selection> {
    let agent = model.focused_agent_id()?;
    Some(crate::nav::convs::selection(
        &super::convs::forest(model).value.unwrap_or_default(),
        &agent,
    ))
}

/// The selection's own detail, over the wire — the tip the §9.4 row freezes on,
/// the §6 marks, the §8.6 park and the `Nudge` gate. A frame the engine has not
/// answered paints none of them, which is the collapsed-pane rule at one row.
pub(super) fn detail(model: &mut AppModel, ws: &Path, agent: &str) -> Landed<AgentView> {
    let workspace = model.snap.ws_name(ws);
    let agent = agent.to_owned();
    ask(
        model,
        Query::Agent { workspace, agent },
        |reply| match reply {
            Reply::Agent(view) => Some(view),
            _ => None,
        },
    )
}
