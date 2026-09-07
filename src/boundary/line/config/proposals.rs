//! **The §9.6 learning-loop pair, read off a line** (§9.6; REMOTE §9.22,
//! bl-dd88): the listing and the settle.
//!
//! Its own file beside [`super::config`] at §12's budget, on the seam the pair
//! itself draws: everything there writes a config file or reads its bytes, and
//! these two are about a *candidate* commit — the one a reviewer staged and
//! nobody has taken yet.
//!
//! **Two words rather than one verb with a mode.** A read and a destructive act
//! must not be one typo apart, and the plural/singular pair says which is which
//! before an operator has finished typing.

use super::super::Context;
use super::super::args;
use crate::boundary::Action;
use crate::boundary::config::{Read, Write};
use crate::boundary::{Gesture, Query};
use crate::proposals::{Settle, Verdict};

/// `/proposals [<id>]` — the §9.6 read (bl-dd88): every staged proposal of the
/// seat's own workspace, and — when an id is named — that proposal whole. One
/// verb at two depths, the shape `/files` and `/work-diff` already take: the
/// listing is what a seat holds, and naming one is a question about a row it
/// was already answered.
pub(super) fn proposals(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let id = match tail.split_whitespace().collect::<Vec<_>>()[..] {
        [] => None,
        [id] => Some(id.to_owned()),
        _ => return Err(format!("/{verb}: usage: /{verb} [<proposal-id>]")),
    };
    Ok(Gesture::Ask(Query::Config(Read::Proposals {
        workspace: args::workspace(ctx, verb)?,
        id,
    })))
}

/// `/proposal <id> <accept|reject>` — the §9.6 settle (bl-dd88), the operator
/// act the learning loop turns on. **Both words are required**: a settle that
/// took "the only one" would do something different the day a second proposal
/// was staged, and a verdict that defaulted would make the destructive half the
/// easy one.
pub(super) fn proposal(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let usage = || format!("/{verb}: usage: /{verb} <proposal-id> <accept|reject>");
    let [id, word] = *tail.split_whitespace().collect::<Vec<_>>() else {
        return Err(usage());
    };
    let verdict = Verdict::parse(word).ok_or_else(usage)?;
    Ok(Gesture::Act(Action::Config(Write::Proposal(Settle {
        workspace: args::workspace(ctx, verb)?,
        id: id.to_owned(),
        verdict,
    }))))
}
