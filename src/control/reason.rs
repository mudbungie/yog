//! **The sentence** (VISION §4.11 items 5–6, DESIGN §8.6): what a hold hands
//! the operator, and what a refusal hands the model. One file, because the two
//! are the same sentence with one paragraph more, and because the hold's text
//! is read back later to learn what class was parked.
//!
//! The hold's half names the tool, **what it was about to do**, the class it
//! landed in and the evidence that put it there. Never a section number — the
//! reader has the window, not the document. The input summary lives here rather
//! than at the attention item because the control is the only thing that sees
//! the invocation: the mark carries the sentence, so the parked drone's whole
//! story is one fact with one home and the operator never opens a transcript.
//!
//! **The refusal's half is stated, once, and it closes three loopholes**
//! (bl-94a5). What the model received before was the hold's sentence and
//! nothing else — a classification, which reads as a correctable error, and a
//! measured run answered it by probing the confinement over three steps and
//! finding a spelling that passed. So a refusal now says who decided, **how far
//! the decision stands** ([`Scope::stands_for`]), and that retrying,
//! rephrasing and reaching the same outcome another way are all inside it. It
//! never invites a rephrasing and it never offers a way round: the one way
//! forward is the operator's, and the model is told to say what it needed and
//! stop.
//!
//! **The class is read back out of the sentence** ([`class_of`]) because the
//! hold mark is litany's blob and carries three fields — id, tool, reason —
//! and the reason is the one of them yog wrote. A scope wider than the call
//! stands over a *class*, so the answer gesture must know which class it is
//! releasing; parsing our own sentence beats asking upstream for a fourth
//! field, and the two directions are one file with one test holding them equal.

use super::classify::{Classified, Effect};
use super::judge::{Scope, answers::Standing};
use super::wire::{Request, Verdict};

/// How many `char`s of the invocation's input the reason carries. Enough to
/// recognise the command; bounded because the sentence rides a git blob an
/// operator reads at a glance.
const SUMMARY_MAX: usize = 160;

/// The clause the class sits in, spelled once — [`reason`] writes it and
/// [`class_of`] reads it, so the two cannot drift.
fn clause(effect: Effect) -> String {
    format!(" classified {} (", effect.word())
}

/// The sentence a hold hands the operator: the tool, what it was about to do,
/// the class it landed in, and the evidence that put it there.
pub fn reason(request: &Request, classified: &Classified) -> String {
    format!(
        "{} {}{}{})",
        request.name,
        clip(&request.input.to_string()),
        clause(classified.effect),
        classified.why,
    )
}

/// The class a hold's sentence names, or `None` when no clause of ours is in
/// it. The **last** clause wins: an input summary or an evidence clause may
/// quote one, and the sentence's own is the one at the end.
pub fn class_of(reason: &str) -> Option<Effect> {
    Effect::every()
        .into_iter()
        .filter_map(|effect| reason.rfind(&clause(effect)).map(|at| (at, effect)))
        .max_by_key(|(at, _)| *at)
        .map(|(_, effect)| effect)
}

/// The verdict one adjudication answers with: a pass carries no reason at all
/// (litany's parser rejects one), a hold carries the operator's sentence, and a
/// refusal carries it plus the paragraph above.
pub fn verdict(standing: Standing, request: &Request, classified: &Classified) -> Verdict {
    let why = reason(request, classified);
    match standing.ruling {
        super::judge::Ruling::Pass => Verdict::Pass,
        super::judge::Ruling::Hold => Verdict::Hold(why),
        super::judge::Ruling::Refuse => Verdict::Refuse(format!(
            "{why}. {}",
            refusal(standing.scope, &request.name, classified.effect)
        )),
    }
}

/// The standing paragraph a refusal hands the model. Three loopholes named in
/// one breath — retry, rephrase, and the same outcome by another route — then
/// the one thing left to do.
pub fn refusal(scope: Scope, tool: &str, effect: Effect) -> String {
    format!(
        "The operator has not consented to this, and the refusal stands for {}. Do not retry it, \
         do not rephrase it, and do not reach the same outcome by another command, another tool, \
         another path or a change of working directory. Say what you needed it for, and stop.",
        scope.stands_for(tool, effect),
    )
}

/// `text` bounded to [`SUMMARY_MAX`] chars, saying so when it was cut.
fn clip(text: &str) -> String {
    let flat = text.replace(['\n', '\r'], " ");
    if flat.chars().count() > SUMMARY_MAX {
        flat.chars().take(SUMMARY_MAX).chain("…".chars()).collect()
    } else {
        flat
    }
}

#[cfg(test)]
mod tests;
