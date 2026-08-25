//! **What a conversation is called** (DESIGN §3.3) and when it started
//! (bl-16da) — the §11 display ladder, its floor spelling, and the one stamp
//! grammar both read.
//!
//! Split off [`super`] at §12's pre-split band on the seam §3.3 already draws:
//! the parent folds the descent forest into conversations, and this names what
//! the fold produced. Every seat falls through the same rungs here — the §11
//! row title, the center header, the §3.6 deletion confirmation — so no two of
//! them can come to disagree about a name.

use super::row::preview;
use crate::git_tree::Agent;

/// What a conversation is called (DESIGN §3.3) — **the one function**, a ladder:
/// the agent's name fact, else the first payload line (`preview`), else the id's
/// [`id_floor`] — the terminal generation only (bl-63a1). `name` is [`Agent::name_fact`]'s fold — the lernie-stored `name`
/// blob (rung one), else the legacy `You are <x>.` goal-stamp parse covering
/// pre-0.0.4 roots until retention ages them out. Every seat reads it and falls
/// through together — the §11 row title, the §11 center header, the §3.6
/// deletion confirmation. A named conversation (or a named descent child — same
/// rung, no special case) stops on rung one; a foreign or hand-typed root lands
/// on the payload line or the id. The rungs are **mutually exclusive by
/// construction**: the interim stamp comes off the payload at its source
/// ([`crate::start::strip_identity_stamp`], applied in `git_tree::detect` before
/// the cap), so the identity line reaches the name fact and nothing else.
pub fn display_name(name: Option<&str>, preview: &str, root_id: &str) -> String {
    match (name, preview) {
        (Some(name), _) => name.to_owned(),
        (None, "") => id_floor(root_id).to_owned(),
        (None, payload_line) => payload_line.to_owned(),
    }
}

/// The ladder's floor spelling (bl-63a1). A lernie child id embeds the full
/// ancestry chain — one `<stamp>-<hash>` pair per generation — and the descent
/// tree's indentation already states the lineage, so a row re-spelling the
/// whole chain is a second spelling of a derivable fact (the operator:
/// "unparseable"). When the ladder bottoms out at the id, it spells only the
/// **terminal generation**: the substring from the last stamp segment on. A
/// root id is one generation, so it is its own terminal segment, and an id the
/// stamp grammar does not recognize (foreign, hand-made) is spelled whole —
/// the general path, no special case. The full id's display seat stays the
/// hover, exactly as before.
///
/// `pub(crate)` rather than `pub` deliberately: it hands back a borrow of its
/// argument, which the boundary forbids (AGENTS rule 2), and cloning to own it
/// would buy nothing — every caller is in this crate. It is `pub(crate)` at
/// all because the floor has **more than one seat**: a deposit's `from` is an
/// agent id too, and the inbox rows spelled the whole ancestry chain until
/// bl-3aa1 routed them here.
pub(crate) fn id_floor(id: &str) -> &str {
    let mut start = 0;
    let mut at = 0;
    for segment in id.split('-') {
        if is_stamp(segment) {
            start = at;
        }
        at += segment.len() + 1;
    }
    // `start` sits on a `split('-')` boundary, so the slice always holds; the
    // fallback is clippy's string-slice discipline, not a reachable path.
    id.get(start..).unwrap_or(id)
}

/// The §11 header's when-seat: what a conversation id says to a human, and the
/// raw id that hovers behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartedAt {
    pub label: String,
    pub hover: String,
}

/// Explain the id in the hover, so dropping it from the headline costs nothing:
/// it is the branch name and the on-disk key, and that is why it is still here.
const ID_HOVER: &str = "the conversation's id — its branch name and on-disk key";

/// When a conversation started, read out of its own id (bl-16da). Operator:
/// *"the timestamp at the top of the chat is unconsumable. make it still
/// ISO8601, but less built for the machine."* — the id is
/// `20260801T225418Z-2286254c`, lernie's compact ISO 8601 basic form plus a
/// discriminator, and the headline seat wants the extended form
/// (`2026-08-01 22:54:18Z`) with the hash suffix gone: a hash is not a
/// timestamp.
///
/// **Derived at render, never stored** — the id IS the storage, exactly as the
/// §3.3 stamp is for the name — and it is the same ladder discipline as
/// [`display_name`]: an id the stamp grammar does not recognize (a foreign or
/// hand-made branch) is its own label rather than a special case, and the raw
/// id hovers either way.
pub fn started_at(root_id: &str) -> StartedAt {
    StartedAt {
        label: iso_extended(root_id).unwrap_or_else(|| root_id.to_owned()),
        hover: format!("{root_id} — {ID_HOVER}"),
    }
}

/// `20260801T225418Z-<any>` → `2026-08-01 22:54:18Z`. `None` unless the id
/// opens with exactly lernie's stamp: 8 digits, `T`, 6 digits, `Z`, then either
/// the end or the `-` before the discriminator. Assembly is
/// [`crate::ui_state::format_iso8601`], the same call the activity row's
/// epoch-derived timestamp goes through (bl-61db) — one spelling either way.
fn iso_extended(root_id: &str) -> Option<String> {
    let (date, time) = stamp_halves(root_id.split('-').next()?)?;
    let at = |s: &str, a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    Some(crate::ui_state::format_iso8601(
        at(date, 0, 4)?,
        at(date, 4, 6)?,
        at(date, 6, 8)?,
        at(time, 0, 2)?,
        at(time, 2, 4)?,
        at(time, 4, 6)?,
    ))
}

/// The `<date>`/`<time>` halves of a lernie stamp segment, or `None` when the
/// segment is not one: exactly 8 digits, `T`, 6 digits, `Z` — the one grammar
/// both the header's when-label ([`iso_extended`]) and the ladder's
/// [`id_floor`] read, so the two seats can never disagree on what a stamp is.
fn stamp_halves(segment: &str) -> Option<(&str, &str)> {
    let (date, rest) = segment.split_once('T')?;
    let time = rest.strip_suffix('Z')?;
    (date.len() == 8
        && time.len() == 6
        && date.bytes().all(|b| b.is_ascii_digit())
        && time.bytes().all(|b| b.is_ascii_digit()))
    .then_some((date, time))
}

/// **Is this run of text a lernie stamp segment?** — [`stamp_halves`] asked for
/// the answer alone, which is all [`id_floor`] ever wanted of it.
///
/// `pub(crate)` because the §3.3 naming invariant is asserted on *values* rather
/// than on field names since bl-45c7: the acceptance scan reads the painted
/// window and asks this of every token in it. Sharing the predicate with the
/// floor is the point — what counts as an agent id has one definition in the
/// tree, so the scan cannot come to disagree with the seats it polices.
pub(crate) fn is_stamp(segment: &str) -> bool {
    stamp_halves(segment).is_some()
}

/// The same ladder for a seat holding agents rather than a [`ConvRow`] (the §11
/// center header, the §3.6 deletion gate): `root_id`'s own rungs off the
/// snapshot. An id no agent here carries lands on the floor — the same
/// [`id_floor`] spelling every other seat gets (rung three).
pub fn display_name_of(agents: &[Agent], root_id: &str) -> String {
    agents
        .iter()
        .find(|a| a.agent_id == root_id)
        .map_or_else(|| id_floor(root_id).to_owned(), member_title)
}

/// The same ladder for the one agent in hand — the §11 descent-tree member row
/// (bl-df72: that seat painted the raw id, the operator's "incoherent
/// timestamp") and the in-flight strip: the agent's own rungs, no snapshot
/// search. A nameless member is titled by its payload line; the id stays the
/// floor — an id is a fact — but **only the ladder may spell it**: no seat
/// formats an agent id as a display name (the acceptance naming scan holds
/// this), and the floor's spelling is [`id_floor`]'s terminal generation
/// (bl-63a1) — the full id's seat is the hover.
pub fn member_title(agent: &Agent) -> String {
    display_name(
        agent.name_fact().as_deref(),
        &preview(Some(agent)),
        &agent.agent_id,
    )
}

/// One agent's first payload line, capped — the weak text the §11 row paints
/// beside its title, and the same text the §8.5 decision queue carries so a
/// reader of the queue sees what a looker at the strip sees. The [`preview`]
/// fold with the agent in hand, exactly as [`member_title`] is
/// [`display_name`] with the agent in hand.
pub fn preview_of(agent: &Agent) -> String {
    preview(Some(agent))
}
