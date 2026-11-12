//! **The offer grammar** (REMOTE §15.1; bl-9ea6) — four verbs, in their own
//! file beside the other per-family grammars:
//!
//! - `/offer <common-name> [foot|thrall]` — mint an offer in the seat's
//!   workspace. Bare is operator grade, exactly as `/enroll`'s is.
//! - `/offers` — the workspace's pending offers.
//! - `/approve <common-name>` and `/decline <common-name>`.
//!
//! The common name is the one fact no seat's context holds; the workspace is
//! the seat's own.

use super::{Context, args};
use crate::boundary::codec::offer::{APPROVE, DECLINE, OFFER, OFFERS};
use crate::boundary::{Action, Gesture};
use crate::registry::Grade;
use crate::registry::offer::Verb;

/// Read one of the family's four lines, or hand a verb that is not one of them
/// on to the populating reads — [`spend::read`](super::spend::read)'s own
/// shape, and the next link of the same chain.
pub(super) fn read(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    if ![OFFER, OFFERS, APPROVE, DECLINE].contains(&verb) {
        return super::queries::queries(verb, tail, ctx);
    }
    parse(tail, ctx, verb).map(|verb| Gesture::Act(Action::Offer(verb)))
}

/// One of the four lines, read. `verb` is already known to be one of them.
fn parse(tail: &str, ctx: &Context, verb: &str) -> Result<Verb, String> {
    if verb == OFFERS {
        args::none(tail, verb)?;
        return Ok(Verb::List {
            workspace: args::workspace(ctx, verb)?,
        });
    }
    let (name, rest) = args::first_word(tail);
    if name.is_empty() {
        return Err(format!(
            "/{verb}: the common name the device's certificate carries is required — /{verb} \
             <common-name>"
        ));
    }
    if verb == OFFER {
        return Ok(Verb::Mint {
            grade: grade(&rest, verb)?,
            workspace: args::workspace(ctx, verb)?,
            name,
        });
    }
    args::none(&rest, verb)?;
    let workspace = args::workspace(ctx, verb)?;
    Ok(if verb == APPROVE {
        Verb::Approve { workspace, name }
    } else {
        Verb::Decline { workspace, name }
    })
}

/// **The optional grade word** — default-operator made typable (§4.2). Two
/// vocabularies read and one spells (bl-427b): the registry's own words
/// (`foot`, and `operator` said outright) and the brands an operator says out
/// loud (`thrall`, `lernie`). Anything else refuses naming the token rather
/// than rounding to either grade. `/enroll` reads its grade here too.
pub(super) fn grade(rest: &str, verb: &str) -> Result<Grade, String> {
    match args::optional_word(rest, verb)?.as_deref() {
        None | Some("lernie") => Ok(Grade::default()),
        Some("thrall") => Ok(Grade::Foot),
        Some(word) => crate::boundary::codec::grade_of(word).map_err(|e| {
            format!(
                "/{verb}: {e} — say thrall or foot for a tools-only device, lernie or \
                     operator (the bare default) for a seat"
            )
        }),
    }
}

/// One act as its line. Operator grade spells bare, because that is what
/// default-operator means at a keyboard.
pub(super) fn spell(verb: &Verb) -> String {
    match verb {
        Verb::Mint { name, grade, .. } => match grade {
            Grade::Operator => format!("/{OFFER} {name}"),
            Grade::Foot => format!("/{OFFER} {name} {}", grade.word()),
        },
        Verb::List { .. } => format!("/{OFFERS}"),
        Verb::Approve { name, .. } => format!("/{APPROVE} {name}"),
        Verb::Decline { name, .. } => format!("/{DECLINE} {name}"),
    }
}
