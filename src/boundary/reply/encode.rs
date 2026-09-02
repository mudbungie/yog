//! The reply's one JSON spelling (§8.5) — split from the type at §12's
//! per-file budget (bl-6233), on the seam the codec is already cut along: a
//! [`Reply`] is what the boundary *answers*, this is how the headless transport
//! *says* it, and the GUI reads the variant without ever coming here.
//!
//! The match stays exhaustive over [`Reply`], so the compile gate is unchanged
//! — a variant added tomorrow does not build until it is spelled.

use serde_json::{Map, Value, json};

use super::board::{board_row, fleet_facts};
use super::queue::queue_row;
use super::rows::{conv_row, join_row, lineage_row, op_row, ws_row};
use super::{Reply, prepared_value};
use crate::registry::mailbox::{capture_value, invocation_value};

/// The follow lane's reply kind (bl-73e7), named once for both directions.
pub(super) const FOLLOW: &str = "follow";

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
        Reply::Advertised => json!({ "ok": true, "kind": "advertised" }),
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
        Reply::Attention(rows) => rows_reply("attention", rows.iter().map(queue_row)),
        Reply::Ops(rows) => rows_reply("ops", rows.iter().map(op_row)),
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
        Reply::Config { text } => json!({ "ok": true, "kind": "config", "text": text }),
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

/// One provider row as the operator reads it (§8.3, §9.5): its name, the
/// credential fact in words, why `bz --login` cannot serve it — `null` exactly
/// when it can — and the two **tuning capabilities** a controls surface shows
/// its `/effort` and `/priority` controls under (bl-23bd).
///
/// The two are always present and always booleans, never absent-is-false on the
/// wire: a capability the seat cannot read is a control it cannot decide about,
/// and this is the row whose whole job is to decide it. Absence is brazen's
/// dialect to speak, and it is spoken one layer down where the column is read.
fn provider_row(row: &crate::config_edit::brazen::ProviderRowView) -> Value {
    json!({ "name": row.name, "fact": row.fact, "blocked": row.blocked,
            "effort": row.effort, "priority": row.priority })
}

/// A captured run as its receipt (§7.3): the verb's own exit and streams,
/// with `ok` derived from the exit rather than stored beside it.
///
/// A body rather than a row for the reason [`delivered`] and [`governing`] are
/// already bodies here — an arm that builds a four-key object is a body, and
/// the roster stops reading as one once any of them is.
fn outcome_reply(outcome: &crate::actions::verbs::Outcome) -> Value {
    json!({
        "ok": outcome.ok(), "kind": "outcome", "exit": outcome.exit,
        "stdout": outcome.stdout, "stderr": outcome.stderr,
    })
}

/// One role's assignment as the workspace's config declares it (§9.4, §5.1
/// #27; bl-2410): the model binding, and the two tuning knobs beside it.
///
/// `effort` is the file's own word or `null` — a *reported* value, not the
/// gesture's closed vocabulary, so a level yog did not write is visible rather
/// than flattened into *not set*. `priority` is always a boolean, because
/// `false` and absent are one fact upstream and a reader must not be made to
/// tell them apart.
fn role_row(row: &crate::model_pick::RoleModel) -> Value {
    json!({ "role": row.role, "provider": row.provider, "model": row.model,
            "effort": row.effort, "priority": row.priority })
}

/// One help page as data — the same four facts every seat renders (§8.5).
fn help_row(row: &crate::boundary::help::HelpRow) -> Value {
    json!({ "verb": row.verb, "usage": row.usage,
            "summary": row.summary, "detail": row.detail })
}

/// One registered client as every seat renders it (REMOTE §5): its identity,
/// whether it is connected right now, and what it advertises.
fn client_row(row: &crate::registry::roster::ClientRow) -> Value {
    json!({ "client": row.client, "present": row.present,
            "tools": crate::registry::tools::encode(&row.tools) })
}

/// The envelope every reply opens with, before its own fields — the shape
/// [`rows_map`] builds a listing on and the one a keyed answer builds itself on.
/// The delivery's four identities (V3.2, bl-c2bd). The two options are
/// **absent** rather than null, upstream's own meaning kept: an unmade source
/// ref and a delivery that landed nothing are absences, not empty strings.
fn delivered(delivery: &crate::fan::Delivery) -> Value {
    let mut map = obj_reply("delivered");
    map.insert("target".to_owned(), json!(delivery.target));
    map.insert("base".to_owned(), json!(delivery.base));
    if let Some(source) = &delivery.source {
        map.insert("source".to_owned(), json!(source));
    }
    if let Some(commit) = &delivery.commit {
        map.insert("commit".to_owned(), json!(commit));
    }
    Value::Object(map)
}

/// The QR envelope's payload (REMOTE §1.4 as amended, bl-f4e3). Every field is
/// present always — there is no absent case, because a device handed five of
/// the six facts cannot dial, cannot verify, or cannot say who it is — and the
/// three PEMs ride **verbatim**, newlines and all: the envelope measures 1567
/// bytes of compact JSON against a byte-mode QR's 2953, so nothing is
/// re-encoded to buy room it does not need.
fn enrolled_reply(enrolled: &crate::registry::enroll::Enrolled) -> Value {
    json!({
        "ok": true, "kind": ENROLLED, "grade": enrolled.grade.word(),
        "name": enrolled.name, "address": enrolled.address,
        "ca": enrolled.ca, "cert": enrolled.cert, "key": enrolled.key,
    })
}

/// The which-config-governs answer (bl-13f9; follow-the-tip, bl-e654). The oid
/// is the **resolved** commit's and rides both ways — short is what a pane
/// labels with, full is what a `git show` outside yog takes — exactly as a
/// lineage row's tip does. `follows` and `diverged_lineages` are the two faces
/// of one enum: a name and `0`, or `null` and the count that held it.
fn governing(gov: &crate::config_edit::branch::GoverningConfig) -> Value {
    json!({
        "ok": true, "kind": "governing",
        "oid": gov.oid, "short_oid": gov.short_oid,
        "follows": gov.followed_lineage(),
        "diverged_lineages": gov.diverged_lineages(),
        "files": gov.files,
    })
}

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
