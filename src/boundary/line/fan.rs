//! The §3.8 **mutating fan**'s line grammar (§8.5) — its own file beside
//! [`super::fork`]'s, on the same seam: a family whose gestures read an
//! obligation off the seat rather than a bare tail.

use super::{Context, args};
use crate::boundary::{Action, Gesture};
use crate::fan::Verb;

/// The family's one door from the reader's roster: which of the three a verb
/// word names. The roster matched the word already, so the fallthrough arm is
/// the delivery — the same shape [`codec::balls`](crate::boundary::codec)'s
/// reader keeps.
pub(super) fn read(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    match verb {
        "fan" => fan(tail, ctx, verb),
        "retire" => retire(tail, ctx, verb),
        _ => deliver(tail, ctx, verb),
    }
}

/// `/fan <n>` — the §4.10 mutating fan. The obligation is the seat's own: the
/// focused project and the focused ball, exactly as `/close`'s is, so the only
/// word a line carries is **N**, which is the one thing that is yog's policy
/// rather than a derived fact. The prepared start is the seat's too — a fan
/// spreads a `/prepare`, so it refuses for the same reason `/prompt` does when
/// nothing is prepared.
///
/// A **bare project-repo** fan (no ball, the integration branch as target,
/// §4.10 item 8) has no line spelling on purpose: the line supplies what a seat
/// has selected and refuses what it cannot, and reading "no ball selected" as
/// "fan the integration branch" would be a guess at a different gesture. The
/// envelope stays the spelling for that.
fn fan(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let n = args::required(tail, verb, "how many candidates")?;
    Ok(Gesture::Act(Action::Fan(Verb::Spread {
        prepared: prepared(ctx, verb)?,
        obligation: obligation(ctx, verb)?,
        n: n.parse()
            .map_err(|_| format!("/{verb}: {n:?} is not a count; usage: /fan <n>"))?,
    })))
}

/// `/retire <handle>` — release one candidate, per the project's retention
/// policy. The handle is balls' own opaque name, read off the cohort, and it is
/// required: there is no "the current candidate" for a seat to mean.
fn retire(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    Ok(Gesture::Act(Action::Fan(Verb::Retire {
        obligation: obligation(ctx, verb)?,
        handle: args::required(tail, verb, "the candidate handle")?,
    })))
}

/// `/deliver <handle> <summary…>` — **Deliver candidate** (VISION V3.2): accept
/// one candidate by the ordinary source-to-target delivery. The summary is the
/// whole tail after the handle, verbatim — it becomes the delivery subject,
/// which balls tags `[<handle>]` — and it is required: a delivery subject is
/// the operator's statement of what landed, and yog does not invent one.
fn deliver(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let tail = tail.trim();
    let missing =
        |what: &str| format!("/{verb}: {what} is required; usage: /deliver <handle> <summary…>");
    if tail.is_empty() {
        return Err(missing("the candidate handle"));
    }
    let (handle, summary) = tail
        .split_once(char::is_whitespace)
        .map(|(h, s)| (h, s.trim_start()))
        .filter(|(_, s)| !s.is_empty())
        .ok_or_else(|| missing("the delivery summary"))?;
    Ok(Gesture::Act(Action::Fan(Verb::Deliver {
        obligation: obligation(ctx, verb)?,
        handle: handle.to_owned(),
        summary: summary.to_owned(),
    })))
}

/// The seat's delivery obligation: its focused project and its focused ball.
/// The ball is **required** here even though the type allows none — a bare
/// project-repo obligation is a different gesture (§4.10 item 8) and reading
/// "nothing selected" as "the integration branch" would be a guess at it.
fn obligation(ctx: &Context, verb: &str) -> Result<crate::fan::Obligation, String> {
    Ok(crate::fan::Obligation {
        project: args::project(ctx, verb)?,
        ball: Some(super::balls::id("", ctx, verb)?),
    })
}

/// The seat's prepared start, or the refusal naming it — `/prompt`'s own
/// context read, shared with `/fan` because both spend one `/prepare`.
fn prepared(ctx: &Context, verb: &str) -> Result<crate::start::Prepared, String> {
    ctx.prepared
        .clone()
        .ok_or_else(|| format!("/{verb}: nothing is prepared — /prepare first"))
}
