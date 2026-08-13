//! `/fork` — the attempt, typed (§8.5, VISION V2).
//!
//! ```text
//! /fork --from <ref> --role <role> [--skills a,b] --goal <the goal…>
//! ```
//!
//! **The goal is last and verbatim**, exactly as it is in the argv this
//! spells and in the start flow's own fire (§3.3, bl-6920): everything after
//! `--goal` is the payload, spacing and all, and no flag is read out of it.
//! That is why the flags lead — a line whose payload is the tail cannot carry
//! anything after it.
//!
//! **No count.** ×N is N lines, because a cohort is derived from the notch its
//! members hang on and not from a number anyone typed ([`crate::fork`]). A
//! `--times` flag here would be the fan verb the ruling forbids, wearing a
//! flag's clothes.
//!
//! The workspace and the dispatching parent are the seat's own selection, like
//! every other conversation verb's — a seat holding neither refuses by name.
//!
//! Its own grammar module rather than an arm of [`super::parse`]: the payload
//! is a verbatim tail behind leading flags, which is the one shape the shared
//! positional helpers cannot read.

use super::{Context, args};
use crate::boundary::{Action, Gesture};
use crate::fork::Attempt;

/// The words that open the payload. Everything after the first occurrence is
/// the goal, verbatim.
const GOAL_FLAG: &str = "--goal";
const FROM: &str = "from";
const ROLE: &str = "role";
const SKILLS: &str = "skills";
/// How a line lists several skills: one token, comma-separated, because the
/// line's own rule whitespace-normalizes every value but the payload.
const SKILL_SEP: char = ',';

/// Read `/fork`'s tail into the [`Action::Fork`] it spells.
pub(super) fn fork(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let (head, goal) = split_goal(tail, verb)?;
    let (extra, flags) = args::split_flags(&head);
    if !extra.is_empty() {
        return Err(format!("/{verb}: unexpected words {extra:?} before --goal"));
    }
    args::only(&flags, &[FROM, ROLE, SKILLS], verb)?;
    let from = args::flag(&flags, FROM, verb)?
        .ok_or_else(|| format!("/{verb}: --from is required — name the ref to fork off"))?;
    let role = args::flag(&flags, ROLE, verb)?
        .ok_or_else(|| format!("/{verb}: --role is required — it is what names the model"))?;
    Ok(Gesture::Act(Action::Fork {
        workspace: args::workspace(ctx, verb)?,
        parent: args::agent(ctx, verb)?,
        attempt: Attempt {
            from,
            role,
            skills: skills(args::flag(&flags, SKILLS, verb)?),
        },
        goal,
    }))
}

/// Split the tail at `--goal`: the flags before it, the payload after it
/// verbatim. A line with no `--goal`, or one whose payload is blank, is a
/// refusal naming what is missing — never an empty goal fired at a model.
fn split_goal(tail: &str, verb: &str) -> Result<(String, String), String> {
    let (head, payload) = tail.split_once(GOAL_FLAG).ok_or_else(|| {
        format!("/{verb}: --goal is required, and everything after it is the goal")
    })?;
    let goal = payload.trim();
    if goal.is_empty() {
        return Err(format!("/{verb}: --goal needs a value"));
    }
    Ok((head.to_owned(), goal.to_owned()))
}

/// A comma-separated skill list, emptied of blanks. Absent is no skills.
fn skills(value: Option<String>) -> Vec<String> {
    value
        .iter()
        .flat_map(|text| text.split(SKILL_SEP))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Spell one attempt back as its line — the flags, then the verbatim goal.
pub(super) fn spell(attempt: &Attempt, goal: &str) -> String {
    let list = if attempt.skills.is_empty() {
        String::new()
    } else {
        format!(" --skills {}", attempt.skills.join(","))
    };
    format!(
        "/fork --from {} --role {}{list} {GOAL_FLAG} {goal}",
        attempt.from, attempt.role
    )
}
