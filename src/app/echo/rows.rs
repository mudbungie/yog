//! **The pending echo at the ROW altitude** (§3.4, §7.2; REMOTE §9.7, bl-44e9)
//! — the same optimism [`super::compose`] folds into a snapshot, folded instead
//! into an answered §11 conversation list.
//!
//! Why there are two projections of one fact, and why that is not two sources.
//! The §11 list reads a `Reply::Conversations` since bl-44e9, so it never sees
//! the composed snapshot — and bl-adcb's *"optimism is a seat's, and a seat that
//! reads over a wire has none"* would, taken as the last word, silently delete
//! §3.4 from the one surface it exists for: the operator's typed goal would
//! again have no representation in yog until the detached driver wrote a branch
//! (bl-915e, which is the defect this whole mechanism is). So the ruling is
//! narrower than the sentence read: **a seat's optimism reaches whatever that
//! seat actually reads**, and what this one reads is rows.
//!
//! Both projections live in [`super`] for the single-source reason [`compose`]
//! states about itself: one module owns every fold of the echo, so *"what does a
//! frame see that disk does not say?"* still has one place to read. Its own
//! file only because §12's cap says so.
//!
//! [`compose`]: super::compose
//! [`super::compose`]: super::compose

use std::path::Path;

use super::{Echo, Target};
use crate::nav::convs::{ConvBall, ConvRow, forest_rows};

/// The list a seat paints: `rows` as the boundary answered them, with `echo`
/// folded on. Nothing pending, or an echo belonging to another workspace, hands
/// the answer straight back.
///
/// Two cases, which are [`compose`](super::compose)'s own two read at this
/// altitude:
///
/// - **the target is in the answer** — the send is an action on a conversation
///   the world already carries, so its row is *freshened* to the echo's own age
///   and nothing else moves. The reorder `compose` earns by bumping
///   `last_action_unix` is deliberately not made here: the answer arrives
///   sorted, and a seat that re-sorted it would be deriving rather than
///   selecting. The derivation carries the true order one ask later.
/// - **it is not** — a start whose branch does not exist yet, which is the whole
///   of what the operator could not see. Its row is minted from the same
///   synthetic agent `compose` appends and **leads** the list, because a start
///   is by construction the newest thing that has happened.
///
/// A **follow-up** whose agent the answer does not carry adds nothing: the
/// conversation it named is gone, and inventing a row for it would be a false
/// definite. A resolved **start** whose id it does not carry yet is the
/// opposite case and mints its row anyway — the derivation is what said that
/// root exists, and this list lands an ask period behind it
/// ([`Echo::pending_identity`], bl-56c6).
pub(crate) fn with_echo(
    echo: Option<&Echo>,
    ws: &Path,
    rows: Vec<ConvRow>,
    now_unix: i64,
) -> Vec<ConvRow> {
    let Some(echo) = echo.filter(|e| e.ws == ws) else {
        return rows;
    };
    match at(echo, &rows) {
        Some(index) => freshen(rows, index, now_unix - echo.at_unix),
        None => lead(echo, rows, now_unix),
    }
}

/// Where the echo's target sits in the answer — by the minted §3.3 name while a
/// start has no id, by the id once it has one. The same two-armed identity
/// [`index_of`](super::index_of) reads off a snapshot.
fn at(echo: &Echo, rows: &[ConvRow]) -> Option<usize> {
    rows.iter().position(|row| match &echo.target {
        Target::Conversation(name) => row.name.as_deref() == Some(name.as_str()),
        Target::Agent(id) => &row.root_id == id,
    })
}

/// Date the echoed row by the send rather than by the last thing the derivation
/// saw. Clamped at zero for the same reason every other age is: a clock that
/// went backwards is not a row from the future.
fn freshen(mut rows: Vec<ConvRow>, index: usize, age: i64) -> Vec<ConvRow> {
    if let Some(row) = rows.get_mut(index) {
        row.age_secs = row.age_secs.min(age.max(0));
    }
    rows
}

/// Put the pending conversation at the head of the list. A follow-up whose
/// agent has no row is left alone.
fn lead(echo: &Echo, rows: Vec<ConvRow>, now_unix: i64) -> Vec<ConvRow> {
    let Some((id, name)) = echo.pending_identity() else {
        return rows;
    };
    pending(echo, &id, &name, now_unix)
        .into_iter()
        .chain(rows)
        .collect()
}

/// The pending conversation projected exactly as a derived one is — through
/// [`forest_rows`], the one row derivation, over a forest of the single
/// synthetic agent [`compose`](super::compose) appends. Nothing about the row is
/// hand-built here, which is what keeps a faded row and a real one the same
/// anatomy (§11).
///
/// It hands back a **list**, which is a one-row list, because that is what the
/// derivation hands back and unwrapping it would be a case to answer where there
/// is none: a forest of one root is one row, and chaining says so without a
/// branch.
///
/// The two injected readers are **named rather than inlined**, because neither
/// is reachable from a pending conversation and a lambda nobody calls is a claim
/// nobody checks. Stated as functions, each is a total answer this module's own
/// tests hold it to.
fn pending(echo: &Echo, id: &str, name: &str, now_unix: i64) -> Vec<ConvRow> {
    let agent = echo.pending_conversation(id, name);
    forest_rows(
        std::slice::from_ref(&agent),
        &crate::nav::ws_key(&echo.ws),
        &unseen,
        now_unix,
        &stray_ball,
        &[],
    )
}

/// The §6 watermark reader a pending conversation gets: **nothing is
/// acknowledged**. It has no evidence oid to have been acknowledged *about*, so
/// its attention count is zero whatever this says — and `false` is the honest
/// answer to "has the operator seen this?" about a thing that does not exist.
fn unseen(_: crate::ui_state::SeenKind, _: &str, _: &str, _: &str) -> bool {
    false
}

/// The §3.5 ball resolver it gets: **the stray-id answer**, which is
/// `answer::conv_ball`'s own miss arm — the id renders, the join supplies
/// nothing. A start stamps no ball on the row until the driver writes its
/// `goal.md`, so this is never asked; if it ever is, it says what every
/// unjoinable stamp says.
fn stray_ball(id: &str) -> ConvBall {
    ConvBall {
        id: id.to_owned(),
        state: None,
        title: None,
        badge: None,
    }
}

#[cfg(test)]
mod tests;
