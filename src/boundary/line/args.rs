//! The line's argument grammar (§8.5): the small, total helpers every verb arm
//! shares — the context reads that refuse by name, the positional words, and
//! the `--flag value…` split.
//!
//! Every refusal here says *which* verb refused and *what* was missing, because
//! a typed control obeys §11's discoverability rule exactly as a clicked one
//! does: the operator must learn what the gesture needed without reading the
//! source.

use super::Context;
use crate::start::BallSpec;
use std::path::PathBuf;

/// The focused workspace, or the refusal naming it.
pub(super) fn workspace(ctx: &Context, verb: &str) -> Result<PathBuf, String> {
    ctx.workspace
        .clone()
        .ok_or_else(|| format!("/{verb}: no workspace in context — focus one, or use the envelope"))
}

/// The selected conversation's agent, or the refusal naming it.
pub(super) fn agent(ctx: &Context, verb: &str) -> Result<String, String> {
    ctx.agent
        .clone()
        .ok_or_else(|| format!("/{verb}: no conversation is selected — there is nothing to act on"))
}

/// The focused project, or the refusal naming it.
pub(super) fn project(ctx: &Context, verb: &str) -> Result<PathBuf, String> {
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
