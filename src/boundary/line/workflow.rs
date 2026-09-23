//! **The workflow mark's line grammar** (REMOTE §9.24, bl-b680) — the two
//! verbs that pin a conversation's workflow to a lineage's head and let it go,
//! in their own family file for the reason every other family is: the grammar
//! lives beside itself, and the verb table stays a table.
//!
//! `/workflow <lineage>` and `/clear-workflow`: the conversation is the
//! seat's, exactly as `/retarget`'s is, and the lineage is the one word the
//! context cannot supply. The direction is the verb, the pin's own shape — a
//! bare `/workflow` is a refusal naming what it wanted, never a clear.

use super::parse::act;
use super::{Context, args};
pub(super) use crate::boundary::codec::workflow::{CLEAR, WORKFLOW};
use crate::boundary::{Action, Gesture};

/// The family's two lines, by the verb that named one.
pub(super) fn read(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    let config = if verb == WORKFLOW {
        Some(args::required(
            tail,
            verb,
            "the lineage whose workflow to run",
        )?)
    } else {
        args::none(tail, verb)?;
        None
    };
    Ok(act(Action::Workflow {
        workspace: args::workspace(ctx, verb)?,
        agent: args::agent(ctx, verb)?,
        config,
    }))
}

/// The line one mark gesture spells as.
pub(super) fn spell(config: Option<&str>) -> String {
    match config {
        Some(name) => format!("/{WORKFLOW} {name}"),
        None => format!("/{CLEAR}"),
    }
}
