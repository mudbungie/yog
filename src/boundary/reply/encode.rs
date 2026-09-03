//! The reply's one JSON spelling (§8.5) — split from the type at §12's
//! per-file budget (bl-6233), on the seam the codec is already cut along: a
//! [`Reply`] is what the boundary *answers*, this is how the headless transport
//! *says* it, and the GUI reads the variant without ever coming here.
//!
//! The match stays exhaustive over [`Reply`], so the compile gate is unchanged
//! — a variant added tomorrow does not build until it is spelled.

use serde_json::{Map, Value, json};

/// The keyed bodies, split at §12's cap — see that module's doc.
mod bodies;

use super::board::{board_row, fleet_facts};
use super::rows::{conv_row, join_row, lineage_row, provider_row, role_row, ws_row};
use super::{Reply, prepared_value};
use crate::registry::mailbox::{capture_value, invocation_value};
use bodies::{delivered, enrolled_reply, governing, outcome_reply};

/// The follow lane's reply kind (bl-73e7), named once for both directions.
pub(super) const FOLLOW: &str = "follow";

/// The sign-in standing's reply kind (bl-c285), named once for both
/// directions — one kind, because a receipt and a frame are one value.
pub(super) const LOGIN: &str = "login";

/// Enrollment's reply kind (bl-f4e3), named once for both directions — and the
/// word REMOTE §1.4's QR envelope contract is written against.
pub(super) const ENROLLED: &str = "enrolled";

/// Encode a reply to its file body. `ok` is the one field every reply carries.
pub fn encode(reply: &Reply) -> Value {
    match reply {
        Reply::Outcome(outcome) => outcome_reply(outcome),
        Reply::Prepared(prepared) => {
            json!({ "ok": true, "kind": "prepared", "prepared": prepared_value(prepared) })
        }
        // The rows ARE `prepared` bodies, in the spelling `prompt` reads back.
        Reply::Fanned(candidates) => rows_reply("fanned", candidates.iter().map(prepared_value)),
        Reply::Retired { discarded } => {
            json!({ "ok": true, "kind": "retired", "discarded": discarded })
        }
        Reply::Delivered(delivery) => delivered(delivery),
        Reply::Started { conversation } => {
            json!({ "ok": true, "kind": "started", "conversation": conversation })
        }
        Reply::Deleted => json!({ "ok": true, "kind": "deleted" }),
        Reply::Armed { armed } => json!({ "ok": true, "kind": "armed", "armed": armed }),
        Reply::Flagged => json!({ "ok": true, "kind": "flagged" }),
        Reply::Answered {
            tool_use,
            tool,
            ruling,
            advanced,
        } => json!({ "ok": true, "kind": "answered", "tool_use": tool_use, "tool": tool,
                     "verdict": ruling.word(), "advanced": advanced }),
        Reply::Floored { standing } => {
            json!({ "ok": true, "kind": "floored", "standing": standing })
        }
        Reply::Nudged => json!({ "ok": true, "kind": "nudged" }),
        Reply::Acked => json!({ "ok": true, "kind": "acked" }),
        Reply::TrailCleared => json!({ "ok": true, "kind": "trail-cleared" }),
        Reply::Applied => json!({ "ok": true, "kind": "applied" }),
        Reply::Advertised { wrote } => json!({ "ok": true, "kind": "advertised", "wrote": wrote }),
        Reply::Enrolled(enrolled) => enrolled_reply(enrolled),
        // The routing leg's asking side (bl-024b): the handle, and the capture
        // once there is one. `capture` is **absent** rather than empty while
        // the far machine still runs it — a reader must not have to tell "not
        // finished" from "finished saying nothing".
        Reply::Routed {
            invocation,
            capture,
        } => {
            let mut map = obj_reply("routed");
            map.insert("invocation".to_owned(), json!(invocation));
            if let Some(capture) = capture {
                map.insert("capture".to_owned(), capture_value(capture));
            }
            Value::Object(map)
        }
        // The follow-class read's rows, in the one invocation spelling.
        Reply::Invocations(rows) => rows_reply("invocations", rows.iter().map(invocation_value)),
        // The branch, and only the branch (REMOTE §8, bl-ccf7): the space it is
        // a branch of is a pure function of the workspace the gesture named, so
        // saying it here would be that name spelled a second time, as a path.
        Reply::Marks { branch } => json!({ "ok": true, "kind": "marks", "branch": branch }),
        // The enumeration, and how current the derivation behind it is
        // (bl-b4b5). Both notes are **absent** rather than null in the ordinary
        // case, which is what makes "fresh" and "the engine declined to say"
        // two readings rather than one.
        Reply::Workspaces(view) => {
            let mut map = rows_map("workspaces", view.rows.iter().map(ws_row));
            for (key, note) in [("stale", &view.stale), ("growth", &view.growth)] {
                if let Some(note) = note {
                    map.insert(key.to_owned(), json!(note));
                }
            }
            Value::Object(map)
        }
        Reply::WorkspaceBalls(rows) => {
            rows_reply("workspace-balls", rows.iter().map(super::balls::bound_ball))
        }
        Reply::Conversations(rows) => rows_reply("conversations", rows.iter().map(conv_row)),
        Reply::Balls(rows) => rows_reply("balls", rows.iter().map(join_row)),
        // The board answers its rows and, when a §4.3 loop is armed over them,
        // that loop's facts beside them. `fleet` is absent — not empty — in an
        // unarmed world: a reader must not have to tell "no loop" from "a loop
        // with nothing in it" (V4's burden check, in the wire shape).
        Reply::Board(board) => {
            let mut map = rows_map("board", board.rows.iter().map(board_row));
            if !board.fleet.is_empty() {
                map.insert(
                    "fleet".to_owned(),
                    json!(board.fleet.iter().map(fleet_facts).collect::<Vec<Value>>()),
                );
            }
            Value::Object(map)
        }
        Reply::Attention(rows) => super::queue::attention(rows),
        Reply::Acknowledged(ack) => super::queue::acknowledged(ack),
        Reply::Ops(rows) => rows_reply("ops", rows.iter().map(super::op_row::op_row)),
        Reply::Help(rows) => rows_reply("help", rows.iter().map(help_row)),
        // The encoders written beside their types rather than here: each
        // family's rows are that module's own vocabulary (see its `wire`). The
        // roster still holds the one line that names each encoder, so there
        // remains exactly one place to learn which reply encodes how.
        Reply::WorkDiff { attempts, patch } => {
            crate::workdiff::wire::reply(attempts, patch.as_ref())
        }
        // The projection over those rows (§3.9, bl-40ab), spelled beside its
        // own type for the same reason — and its `diff` object is the work
        // diff's own row, so an attempt's identity has one spelling anywhere.
        Reply::Science(rows) => crate::science::wire::reply(rows),
        // The §11 inspector family (bl-6233) — the conversation's own reads.
        Reply::Transcript(transcript) => crate::transcript::wire::reply(transcript),
        // One frame of the live tail (bl-73e7). The body is the fold's own
        // spelling, so a follow frame and the tail folded into a transcript
        // are the same value said the same way.
        Reply::Follow(stream) => json!({ "ok": true, "kind": FOLLOW,
                                         "stream": crate::git_tree::stream_wire::stream_value(stream) }),
        Reply::Login(view) => bodies::login_reply(view),
        Reply::Steps(view) => crate::steps_view::wire::steps(view),
        Reply::Step(detail) => crate::steps_view::wire::detail(detail),
        Reply::Files {
            view,
            preview,
            working_dir,
        } => crate::files_view::wire::reply(view, preview.as_ref(), working_dir.as_deref()),
        Reply::Rail(rail) => crate::rail::wire::reply(rail),
        // Flat rather than a row list (bl-13f9): a governing config is one
        // object, so it wears the `config` reply's shape and not `lineages`'.
        // The oid rides both ways — short is what a pane labels the freeze
        // with, full is what a `git show` outside yog takes — exactly as a
        // lineage row's tip does.
        Reply::Governing(gov) => governing(gov),
        Reply::Inbox(entries) => crate::inboxview::wire::reply(entries),
        // The seat's read of its selection (REMOTE §9.4, bl-1eb0).
        Reply::Agent(view) => super::agent::reply(view),
        // Spelled beside its own rows (bl-1015), the `board` and `queue`
        // shape: one file learns how a search answer is said.
        Reply::Search(found) => super::search::reply(found),
        // The §9 config family's answers, one arm since bl-2410: the carrier
        // its questions were folded onto (bl-719a) has a matching set of
        // replies, and the roster names the family once on this side too.
        // The raw bytes and the same bytes read through the file's schema
        // (§9.5, bl-dc3f): one answer, both views, the file the single fact.
        Reply::Config(view) => super::config_view::config(view),
        Reply::Providers(rows) => rows_reply("providers", rows.iter().map(provider_row)),
        Reply::Roles(rows) => rows_reply("roles", rows.iter().map(role_row)),
        Reply::Lineages(rows) => rows_reply("lineages", rows.iter().map(lineage_row)),
        // The one listing whose row is a bare id: a model has no other fact
        // yog knows — brazen publishes an id and a default flag, and which one
        // is default is a `providers.yaml` question, not a roster one (§9.4).
        Reply::Models(ids) => rows_reply("models", ids.iter().map(|id| json!(id))),
        // The tool set rides in its ONE spelling (`registry::tools::encode`),
        // the same bytes the client's own document holds (REMOTE §5, bl-4e08).
        Reply::Clients(rows) => rows_reply("clients", rows.iter().map(client_row)),
    }
}

/// Encode a refusal — a gesture that never ran (decode failure, gate refusal,
/// executor error). The error text is the §7.3 story; the ops trail carries
/// whatever the executor logged before refusing.
pub fn refusal(error: &str) -> Value {
    json!({ "ok": false, "error": error })
}

/// One help page as data — the four facts every seat renders (§8.5), and the
/// fifth no seat renders: **who owes this op a control** (`docs/PARITY.md` §2,
/// bl-8758). The classification ships here rather than in a file of its own
/// because `reply/help.json` is the artifact every client already vendors and
/// replays, so a parity gate reads it without a second fetch and without ever
/// reading another client's tree.
fn help_row(row: &crate::boundary::help::HelpRow) -> Value {
    json!({ "verb": row.verb, "usage": row.usage,
            "summary": row.summary, "detail": row.detail,
            "surface": row.surface.word() })
}

/// One registered client as every seat renders it (REMOTE §5): its identity,
/// whether it is connected right now, and what it advertises.
fn client_row(row: &crate::registry::roster::ClientRow) -> Value {
    json!({ "client": row.client, "present": row.present,
            "tools": crate::registry::tools::encode(&row.tools) })
}

/// The envelope every reply opens with, before its own fields — the shape
/// [`rows_map`] builds a listing on and the one a keyed answer builds itself on.
fn obj_reply(kind: &str) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!(kind));
    map
}

fn rows_reply(kind: &str, rows: impl Iterator<Item = Value>) -> Value {
    Value::Object(rows_map(kind, rows))
}

/// The same, still open for a family that carries one more key.
fn rows_map(kind: &str, rows: impl Iterator<Item = Value>) -> Map<String, Value> {
    let mut map = obj_reply(kind);
    map.insert("rows".to_owned(), json!(rows.collect::<Vec<Value>>()));
    map
}
