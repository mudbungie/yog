//! The reply codec's **decode side** (§8.5, REMOTE §9 step 2, bl-7067) — the
//! whole surface read back into the typed [`Reply`], so a seat that did not
//! derive the answer can hold the same datum the window holds.
//!
//! It is the mirror of [`encode`](super::encode) and it is **strict**, exactly
//! as the gesture codec's decode is: an unknown `kind`, a missing field, a
//! mistyped value and an unknown token each refuse with a reason naming the
//! offending token. The forgiving-parse discipline of an `ops.jsonl` read is
//! for bytes yog wrote itself; a reply arriving over the wire is a peer's
//! statement about a world this process cannot see, and a guessed answer is
//! worse than none.
//!
//! **The refusal is the envelope with no `kind`.** `ok` cannot be the
//! discriminant, because [`Reply::Outcome`] spells the captured run's own
//! verdict there — a `bl close` that failed the gate is `ok: false` and is not
//! a refusal. So a body carrying a `kind` is an answer, and a body carrying
//! none must be `{"ok": false, "error": …}`, which is [`refusal`](super::refusal)'s
//! whole shape.
//!
//! **What keeps this exhaustive.** [`encode`](super::encode)'s match is the
//! compile gate — a variant added tomorrow does not build until it is spelled
//! — and the round-trip test over a fixture of every variant is the gate on
//! *this* side: a variant with a fixture and no arm here fails that test, and a
//! variant with no fixture at all leaves its own encode arm unexecuted, which
//! the 100% coverage floor refuses. Neither gate is a `match` on the enum,
//! because a decoder's input is a string; between them nothing lands unspelled.

use serde_json::{Map, Value};

use super::Reply;
use super::board::decode::board;
use super::queue::queue_row_of;
use super::rows::decode::{conv_row, join_row, lineage_row, op_row, provider_row, rows_of, ws_row};
use super::search::hit_of;
use crate::boundary::codec::fields::{bool_of, list_of, opt_str_of, opt_val, str_of, strings_of};
use crate::boundary::codec::prepared_from_value;
use crate::registry::mailbox::{capture_of, invocation_of};

mod inspector;

/// Read one reply body. The outer `Err` is a malformed envelope — bytes this
/// codec cannot read at all — and the inner `Err` is the refusal the envelope
/// faithfully carried, which is the very `Result<Reply, String>` both
/// chokepoints answer with.
pub fn decode(v: &Value) -> Result<Result<Reply, String>, String> {
    let o = v.as_object().ok_or("reply: not a JSON object")?;
    let Some(kind) = o.get("kind") else {
        return refusal_of(o).map(Err);
    };
    let kind = kind.as_str().ok_or("reply: non-string field \"kind\"")?;
    receipt(kind, o)
        .or_else(|| listing(kind, o))
        .or_else(|| inspector::decode(kind, o))
        .unwrap_or_else(|| Err(format!("unknown reply kind {kind:?}")))
        .map(Ok)
}

/// The kind-less envelope: a refusal, and nothing else may wear that shape.
fn refusal_of(o: &Map<String, Value>) -> Result<String, String> {
    if bool_of(o, "ok")? {
        return Err("reply: an answer with no kind".to_owned());
    }
    str_of(o, "error")
}

/// The receipts (§8.5): what one act earned. `None` when the kind is not one.
fn receipt(kind: &str, o: &Map<String, Value>) -> Option<Result<Reply, String>> {
    Some(match kind {
        "outcome" => outcome(o),
        "prepared" => prepared(o),
        "started" => str_of(o, "conversation").map(|conversation| Reply::Started { conversation }),
        "deleted" => Ok(Reply::Deleted),
        "armed" => bool_of(o, "armed").map(|armed| Reply::Armed { armed }),
        "flagged" => Ok(Reply::Flagged),
        "answered" => answered(o),
        "floored" => bool_of(o, "standing").map(|standing| Reply::Floored { standing }),
        "nudged" => Ok(Reply::Nudged),
        "acked" => Ok(Reply::Acked),
        "trail-cleared" => Ok(Reply::TrailCleared),
        "applied" => Ok(Reply::Applied),
        "advertised" => Ok(Reply::Advertised),
        // The routing leg's asking side (bl-024b): the handle always, the
        // capture only once there is one — `opt_val` is what makes "absent"
        // and "answered nothing" two readings rather than one.
        "routed" => routed(o),
        "marks" => str_of(o, "branch").map(|branch| Reply::Marks { branch }),
        "config" => str_of(o, "text").map(|text| Reply::Config { text }),
        _ => return None,
    })
}

fn prepared(o: &Map<String, Value>) -> Result<Reply, String> {
    let body = o.get("prepared").ok_or("prepared: missing prepared")?;
    prepared_from_value(body).map(Reply::Prepared)
}

/// The captured run. `ok` is read back from `exit` rather than from the key
/// beside it: [`Outcome::ok`](crate::actions::verbs::Outcome::ok) is that
/// fact's one authority, and a second copy could only disagree with it.
fn outcome(o: &Map<String, Value>) -> Result<Reply, String> {
    let exit = o
        .get("exit")
        .and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
        .ok_or("outcome: missing or out-of-range field \"exit\"")?;
    Ok(Reply::Outcome(crate::actions::verbs::Outcome {
        exit,
        stdout: str_of(o, "stdout")?,
        stderr: str_of(o, "stderr")?,
    }))
}

fn answered(o: &Map<String, Value>) -> Result<Reply, String> {
    let word = str_of(o, "verdict")?;
    Ok(Reply::Answered {
        tool_use: str_of(o, "tool_use")?,
        tool: str_of(o, "tool")?,
        ruling: crate::control::judge::Ruling::of(&word)
            .ok_or_else(|| format!("unknown verdict {word:?}"))?,
        advanced: bool_of(o, "advanced")?,
    })
}

/// The listings (§8.5): what a populating read answered.
fn listing(kind: &str, o: &Map<String, Value>) -> Option<Result<Reply, String>> {
    Some(match kind {
        "workspaces" => workspaces(o),
        "workspace-balls" => rows_of(o, super::balls::bound_ball_of).map(Reply::WorkspaceBalls),
        "conversations" => rows_of(o, conv_row).map(Reply::Conversations),
        "balls" => rows_of(o, join_row).map(Reply::Balls),
        "board" => board(o).map(Reply::Board),
        "attention" => rows_of(o, queue_row_of).map(Reply::Attention),
        "ops" => rows_of(o, op_row).map(Reply::Ops),
        "help" => help(o),
        "search" => search(o),
        "providers" => rows_of(o, provider_row).map(Reply::Providers),
        "lineages" => rows_of(o, lineage_row).map(Reply::Lineages),
        "models" => strings_of(o, "rows").map(Reply::Models),
        "clients" => rows_of(o, client_row).map(Reply::Clients),
        "invocations" => rows_of(o, invocation_of).map(Reply::Invocations),
        _ => return None,
    })
}

/// The altitude-0 chrome, read back (bl-b4b5): the rows, and the two §7.2
/// notes when the engine had one to say.
fn workspaces(o: &Map<String, Value>) -> Result<Reply, String> {
    Ok(Reply::Workspaces(crate::boundary::reply::Workspaces {
        rows: rows_of(o, ws_row)?,
        stale: opt_str_of(o, "stale")?,
        growth: opt_str_of(o, "growth")?,
    }))
}

/// One invocation's standing, read back (bl-024b).
fn routed(o: &Map<String, Value>) -> Result<Reply, String> {
    Ok(Reply::Routed {
        invocation: str_of(o, "invocation")?,
        capture: opt_val(o, "capture", capture_of)?,
    })
}

/// One registered client, read back (REMOTE §5, bl-4e08) — the tools through
/// `registry::tools`, the same decoder the gesture and the document spend.
fn client_row(v: &Value) -> Result<crate::registry::roster::ClientRow, String> {
    let o = v.as_object().ok_or("client row: not an object")?;
    Ok(crate::registry::roster::ClientRow {
        client: str_of(o, "client")?,
        present: bool_of(o, "present")?,
        tools: crate::registry::tools::decode(o.get("tools").ok_or("client row: missing tools")?)?,
    })
}

/// Help resolves each row against **this seat's own roster** rather than
/// rebuilding a page from the wire, because a [`HelpRow`](crate::boundary::help::HelpRow)
/// is four `&'static str`s out of a `const` table and no decoder can mint one.
/// That is not a shortfall: help's subject is *the interface, not the world*
/// (`boundary::help`: "any seat can answer it in place, with no consumer, no
/// deposit and no wait"), so the roster is the one answer a seat already holds.
/// A verb this build does not know refuses, naming it.
fn help(o: &Map<String, Value>) -> Result<Reply, String> {
    let table = crate::boundary::help::table();
    rows_of(o, |v| {
        let verb = str_of(v.as_object().ok_or("help row: not an object")?, "verb")?;
        table
            .iter()
            .find(|row| row.verb == verb)
            .copied()
            .ok_or_else(|| format!("help: unknown verb {verb:?}"))
    })
    .map(Reply::Help)
}

/// What matched, and — never silently — what could not be read. The needle
/// rides with them since bl-7067 (see [`encode`](super::encode)).
fn search(o: &Map<String, Value>) -> Result<Reply, String> {
    Ok(Reply::Search(crate::search::Found {
        needle: str_of(o, "needle")?,
        hits: list_of(o, "rows", hit_of)?,
        unreadable: strings_of(o, "unreadable")?,
    }))
}
