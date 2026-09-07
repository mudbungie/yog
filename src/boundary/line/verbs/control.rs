//! The **capability family's** line grammar (VISION §4.11, §8.6): `/answer`
//! and the floor's two verbs, split out of [`super`] at §12's per-file budget,
//! beside the codec's own half of the same family.

use super::super::{Context, args};
use super::act;
use crate::boundary::{Action, Gesture};
use crate::control::judge::{Answer, Ruling, Scope};

/// `/answer pass|hold|refuse [--scope call|conversation|workspace]` — the
/// §4.11 capability answer. The conversation is the seat's, like `/seen`'s;
/// the held `tool_use` id is derived at fire time, so the only words a line
/// carries are the verdict — required, because an answer with a default
/// verdict would be yog deciding — and how far it stands.
///
/// **The scope defaults to the call and the flag is how a wider one is said**
/// (bl-94a5). That direction round is the whole safety property: the narrow
/// answer is the one an operator gets without thinking about it, and standing
/// over a class of calls is a thing they type on purpose.
pub(in crate::boundary::line) fn answer(
    tail: &str,
    ctx: &Context,
    verb: &str,
) -> Result<Gesture, String> {
    let said = args::required(tail, verb, "pass, hold or refuse")?;
    let (word, scope) = match said.split_once("--scope") {
        Some((word, said)) => (word.trim().to_owned(), scope_of(said.trim(), verb)?),
        None => (said, Scope::Call),
    };
    let ruling = Ruling::of(&word)
        .ok_or_else(|| format!("/{verb}: unknown verdict {word:?}; usage: {ANSWER_USAGE}"))?;
    Ok(act(Action::AnswerHold {
        workspace: args::workspace(ctx, verb)?,
        agent: args::agent(ctx, verb)?,
        answer: Answer { ruling, scope },
    }))
}

/// The scope a word names, refusing with the usage where it names none.
fn scope_of(said: &str, verb: &str) -> Result<Scope, String> {
    Scope::of(said).ok_or_else(|| format!("/{verb}: unknown scope {said:?}; usage: {ANSWER_USAGE}"))
}

/// The inverse: one answer as the line that says it. The default scope is
/// spelled by leaving the flag off, which is how it is typed.
pub fn spell_answer(answer: Answer) -> String {
    match answer.scope {
        Scope::Call => format!("/answer {}", answer.ruling.word()),
        scope => format!("/answer {} --scope {}", answer.ruling.word(), scope.word()),
    }
}

/// `/answer`'s usage, said once — the refusals above and the help page read
/// this one string.
pub const ANSWER_USAGE: &str =
    "/answer pass | hold | refuse [--scope call | conversation | workspace]";

/// `/revoke` and `/restore` — VISION §4.9's fifth rung over the §4.11 fold.
/// Neither names anything but itself: the conversation is the seat's own, as
/// `/answer`'s and `/flag`'s are, and the direction is the verb rather than a
/// word after it, because raising and lowering are two instructions and a
/// gesture is never read out of an absence.
pub(in crate::boundary::line) fn floor(
    verb: &str,
    tail: &str,
    ctx: &Context,
) -> Result<Gesture, String> {
    args::none(tail, verb)?;
    Ok(act(Action::Floor {
        workspace: args::workspace(ctx, verb)?,
        agent: args::agent(ctx, verb)?,
        raised: verb == "revoke",
    }))
}
