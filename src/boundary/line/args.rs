//! The line's argument grammar (§8.5): the small, total helpers every verb arm
//! shares — the context reads that refuse by name, the positional words, and
//! the `--flag value…` split.
//!
//! Every refusal here says *which* verb refused and *what* was missing, because
//! a typed control obeys §11's discoverability rule exactly as a clicked one
//! does: the operator must learn what the gesture needed without reading the
//! source.
//!
//! **What is missing is stated here; how to supply it is the seat's** (bl-e66f).
//! These sentences are shared by every seat — the line parser is one
//! implementation and the window's composer and the argv terminal both reach
//! it — so a remedy written into one of them is a remedy asserted at seats that
//! do not have it. The workspace refusal used to end *"focus one, or use the
//! envelope"*, and at `yog gesture` there is nothing to focus: the module doc
//! of [`crate::boundary::sugar::argv`] opens by saying so (*"it holds no
//! selection … a line typed here states its targets outright"*), and the flag
//! that does supply it, `--ws`, went unnamed. The argv seat appends its own
//! usage to every refusal it hands back, which is where a remedy can be true.

use super::Context;
use crate::start::BallSpec;

/// The focused workspace's name, or the refusal naming it.
pub(super) fn workspace(ctx: &Context, verb: &str) -> Result<String, String> {
    ctx.workspace
        .clone()
        .ok_or_else(|| format!("/{verb}: no workspace in context"))
}

/// The selected conversation's agent, or the refusal naming it.
pub(super) fn agent(ctx: &Context, verb: &str) -> Result<String, String> {
    ctx.agent
        .clone()
        .ok_or_else(|| format!("/{verb}: no conversation is selected — there is nothing to act on"))
}

/// The focused project's name, or the refusal naming it.
pub(super) fn project(ctx: &Context, verb: &str) -> Result<String, String> {
    ctx.project
        .clone()
        .ok_or_else(|| format!("/{verb}: no project in context — this seat has no repo to act in"))
}

/// The §3.2 `--as` stamp every `bl` verb carries, or the refusal naming it.
pub(super) fn name(ctx: &Context, verb: &str) -> Result<String, String> {
    ctx.name
        .clone()
        .ok_or_else(|| format!("/{verb}: no workspace name in context — a ball verb is stamped"))
}

/// The ball a verb acts on: the word if one was typed, else the focused ball's
/// id. A focused ball that has no id yet (a §3.4 *new* spec) is not one.
pub(super) fn ball_id(typed: Option<String>, ctx: &Context, verb: &str) -> Result<String, String> {
    typed
        .or_else(|| match ctx.ball.as_ref() {
            Some(BallSpec::Existing { id, .. }) => Some(id.clone()),
            Some(BallSpec::New { .. }) | None => None,
        })
        .ok_or_else(|| format!("/{verb}: no ball id — type one, or select a ball first"))
}

/// A tail that must carry text, trimmed of the whitespace that separated it
/// from the verb. `what` names the missing thing in the refusal.
pub(super) fn required(tail: &str, verb: &str, what: &str) -> Result<String, String> {
    let text = tail.trim();
    if text.is_empty() {
        return Err(format!("/{verb}: {what} is required"));
    }
    Ok(text.to_owned())
}

/// **The goal `/prompt` fires: what was typed, or the prepared prefill when
/// nothing was** (§3.3, bl-06a1).
///
/// `goal` stays the **whole payload, never a tail** — that is bl-6920's ruling
/// (*"the operator sees and edits exactly the whole payload that is sent …
/// fired verbatim"*) and a composer seat depends on it: such a seat pre-loads
/// its box with the prefill and sends the edited whole back, so a `/prompt`
/// that CONCATENATED would fire every composer-typed prefill twice, and no
/// test on the wire can tell one seat's send from the other's without a new
/// flag. Concatenation is therefore not yog's to take alone.
///
/// What was actually missing is the **empty input**, not composition. A seat
/// with no composer — a terminal, where each `yog gesture` is its own process
/// — had no way to say *"the prepared prefill is my payload"*: `/prompt` made
/// the goal mandatory, so a path rung fired without its `Working directory:`
/// headline and a **ball rung lost the ball entirely**, including the `Ball
/// <id>:` header §3.2 calls the conversation→ball join. So the prefill is what
/// fires when nothing was typed, and the *"the goal is required"* refusal is
/// unchanged where there is neither — the bare rung, whose prefill is empty by
/// construction, is that same refusal rather than an arm of its own.
pub(super) fn goal_or_prefill(tail: &str, prefill: &str, verb: &str) -> Result<String, String> {
    match tail.trim() {
        "" => required(prefill, verb, "the goal"),
        typed => Ok(typed.to_owned()),
    }
}

/// A tail that must be empty: the no-argument verbs' whole grammar.
pub(super) fn none(tail: &str, verb: &str) -> Result<(), String> {
    let extra = tail.trim();
    if extra.is_empty() {
        return Ok(());
    }
    Err(format!("/{verb}: takes no arguments, got {extra:?}"))
}

/// A tail that is at most one word — the optional-id verbs' grammar.
pub(super) fn optional_word(tail: &str, verb: &str) -> Result<Option<String>, String> {
    match tail.split_whitespace().collect::<Vec<_>>().as_slice() {
        [] => Ok(None),
        [word] => Ok(Some((*word).to_owned())),
        _ => Err(format!("/{verb}: expected at most one word, got {tail:?}")),
    }
}

/// Split a tail into its leading positional text and its `--key value…` flags.
/// A flag runs to the next flag or the end, and its value is whitespace-
/// normalized — the line's own rule (§8.5): the verbatim payloads (a message's
/// content, a prompt's goal) take the whole tail and never come through here.
pub(super) fn split_flags(tail: &str) -> (String, Vec<(String, String)>) {
    let mut positional: Vec<&str> = Vec::new();
    let mut flags: Vec<(String, String)> = Vec::new();
    for token in tail.split_whitespace() {
        match (token.strip_prefix("--"), flags.last_mut()) {
            (Some(key), _) => flags.push((key.to_owned(), String::new())),
            (None, Some((_, value))) => {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(token);
            }
            (None, None) => positional.push(token),
        }
    }
    (positional.join(" "), flags)
}

/// One flag's value: absent, or present with text. A flag typed with nothing
/// after it is a refusal, not an empty string — the operator meant to say
/// something.
pub(super) fn flag(
    flags: &[(String, String)],
    key: &str,
    verb: &str,
) -> Result<Option<String>, String> {
    match flags.iter().find(|(k, _)| k == key) {
        None => Ok(None),
        Some((_, value)) if value.is_empty() => Err(format!("/{verb}: --{key} needs a value")),
        Some((_, value)) => Ok(Some(value.clone())),
    }
}

/// Refuse a flag this verb does not know, naming it and the ones it does — a
/// misspelling must not be silently dropped on the floor.
pub(super) fn only(flags: &[(String, String)], allowed: &[&str], verb: &str) -> Result<(), String> {
    match flags.iter().find(|(k, _)| !allowed.contains(&k.as_str())) {
        None => Ok(()),
        Some((key, _)) => Err(format!(
            "/{verb}: unknown flag --{key}; this verb takes {}",
            allowed
                .iter()
                .map(|a| format!("--{a}"))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Split a tail's first word from the rest — the sub-verb grammars (`prepare
/// dir …`, `prepare ball …`) and nothing else.
pub(super) fn first_word(tail: &str) -> (String, String) {
    match tail.trim_start().split_once(char::is_whitespace) {
        Some((head, rest)) => (head.to_owned(), rest.trim_start().to_owned()),
        None => (tail.trim().to_owned(), String::new()),
    }
}
