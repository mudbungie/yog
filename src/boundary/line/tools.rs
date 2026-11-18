//! **The routing leg's line grammar** (REMOTE §3, §5; bl-024b) — the two acts
//! that carry an invocation to a tool host and its capture back, typed.
//!
//! They are here rather than in [`parse`](super::parse) for the reason every
//! other family is in its own file: a family's grammar lives beside itself, and
//! the verb table stays a table. What is peculiar to these two is that **both
//! end in a document**, so both take the JSON verbatim to the end of the line —
//! the `/advertise` shape, and for its reason: an input object is the model's
//! own arguments and a capture is a program's own output, and a line grammar
//! that normalized either would be rewriting a payload it does not own.
//!
//! A line seat is an odd place to type either of these — the callers are a
//! driver and a tool host, not a person — but REMOTE §3's ban is on a
//! capability that exists on the wire and nowhere else, so every face gains
//! them or none does. Typing one is also the only way to *watch* the leg by
//! hand, which is worth the two arms.

use serde_json::Value;

use super::args;
use super::parse::act;
use crate::boundary::{Action, Gesture, Query};
use crate::registry::mailbox::stream::piece_of;
use crate::registry::mailbox::{Call, Completion, Post, Verb, capture_of};

/// The family's two lines, by the verb that named one.
pub(super) fn route(verb: &str, tail: &str) -> Result<Gesture, String> {
    match verb {
        "invoke" => invoke(tail, verb),
        // The caller's own table matched one of the two words, so the second
        // arm IS the second word — a fallible re-check whose error arm cannot
        // be reached would be an untestable branch.
        _ => complete(tail, verb),
    }
}

/// `/invoke <client> <tool> [--cwd <dir>] <json input>` — queue one call for
/// a machine, optionally naming the subject's working directory (REMOTE §5's
/// worktree lane, bl-77be). The flag sits between the words and the document
/// because the document runs to the end of the line; a directory containing
/// whitespace cannot ride this face, and the JSON serializations carry it
/// whole.
fn invoke(tail: &str, verb: &str) -> Result<Gesture, String> {
    let (client, rest) = args::first_word(tail);
    let (tool, mut input) = args::first_word(&rest);
    if client.is_empty() || tool.is_empty() {
        return Err(format!(
            "/{verb}: usage: /{verb} <client> <tool> [--cwd <dir>] <json>"
        ));
    }
    let mut cwd = None;
    let (flag, rest) = args::first_word(&input);
    if flag == "--cwd" {
        let (dir, tail) = args::first_word(&rest);
        if dir.is_empty() {
            return Err(format!("/{verb}: --cwd names no directory"));
        }
        cwd = Some(dir);
        input = tail;
    }
    Ok(act(Action::Route(Verb::Invoke(Call {
        client,
        tool,
        input: document(&input, verb)?,
        cwd,
    }))))
}

/// `/complete <invocation> [--partial] <json>` — post what one machine
/// captured, or with the flag a piece of it while the run goes on (REMOTE
/// §5.7). The flag sits before the document for `--cwd`'s reason.
fn complete(tail: &str, verb: &str) -> Result<Gesture, String> {
    let (invocation, document) = args::first_word(tail);
    if invocation.is_empty() {
        return Err(format!(
            "/{verb}: usage: /{verb} <invocation> [--partial] <json>"
        ));
    }
    let (flag, rest) = args::first_word(&document);
    let post = if flag == "--partial" {
        Post::Partial(read(&rest, verb, piece_of)?)
    } else {
        Post::Capture(read(&document, verb, capture_of)?)
    };
    Ok(act(Action::Route(Verb::Complete(Completion {
        invocation,
        post,
    }))))
}

/// The tail as one JSON document, or the refusal naming what it could not read.
fn document(tail: &str, verb: &str) -> Result<Value, String> {
    let text = args::required(tail, verb, "the JSON document")?;
    serde_json::from_str(&text).map_err(|e| format!("/{verb}: {e}"))
}

/// The same, read as a capture or a piece of one — strict on every field,
/// exactly as the envelope is, because it is what a model reads as a tool's
/// answer.
fn read<T>(tail: &str, verb: &str, of: fn(&Value) -> Result<T, String>) -> Result<T, String> {
    of(&document(tail, verb)?).map_err(|e| format!("/{verb}: {e}"))
}

/// `/capture <invocation> [<stdout bytes> <stderr bytes>]` — the asker's poll,
/// and with the two counts REMOTE §5.7's `since`: the text from there on.
pub(super) fn capture(tail: &str, verb: &str) -> Result<Gesture, String> {
    let usage = || format!("/{verb}: usage: /{verb} <invocation> [<stdout bytes> <stderr bytes>]");
    let words: Vec<&str> = tail.split_whitespace().collect();
    let count = |word: &&str| word.parse::<u64>().map_err(|_| usage());
    let (invocation, since) = match words.as_slice() {
        [invocation] => (invocation, None),
        [invocation, stdout, stderr] => (
            invocation,
            Some(crate::registry::mailbox::Offsets {
                stdout: count(stdout)?,
                stderr: count(stderr)?,
            }),
        ),
        _ => return Err(usage()),
    };
    Ok(Gesture::Ask(Query::Capture {
        invocation: (*invocation).to_owned(),
        since,
    }))
}
