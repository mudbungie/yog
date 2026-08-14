//! The boundary's typed answers (§8.5) and their one JSON spelling. A [`Reply`]
//! is what [`dispatch`](super::dispatch::dispatch) and
//! [`answer`](super::answer::answer) return — the datum both frontends consume:
//! the GUI reads the variant in RAM, the headless transport writes
//! [`encode`] to the deposit's reply file. Encode-only: a reply is yog's own
//! statement, never an instruction it parses back — except the `prepare`
//! reply's `prepared` body, which deliberately re-enters as the next
//! [`Prompt`](super::Action::Prompt) gesture and shares its codec spelling.

use crate::actions::verbs::Outcome;
use crate::binding::Workspace;
use crate::board::Board;
use crate::nav::convs::ConvRow;
use crate::opslog::OpRow;
use crate::projects::join::JoinRow;
use crate::search::Found;
use crate::start::Prepared;
use serde_json::{Map, Value, json};
use std::path::PathBuf;

use super::codec::prepared_value;

/// The V4 board row's own encoders — split at the §12 budget, on the seam that
/// board rows are the one reply whose rows carry derived sub-objects (gates,
/// drones, two §3.5 figures).
mod board;
/// The §6 decision queue's row encoder — the other reply whose rows carry a
/// derived list (its firing signals).
mod queue;
mod rows;
use board::{board_row, fleet_facts};
use queue::queue_row;
use rows::{conv_row, join_row, lineage_row, op_row, ws_row};

/// The search reply's own address-flattening — split at the same budget.
mod search;
use search::hit_row;

/// One workspace row (§3.1 classification + the §6 rollups the tab bar shows):
/// the [`Query::Workspaces`](super::Query::Workspaces) answer's element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRow {
    pub workspace: Workspace,
    /// Attention-bearing agents in it (§6).
    pub attention: usize,
    /// Root-and-member agent count.
    pub agents: usize,
    /// Whether anything in it is Live/InFlight right now.
    pub running: bool,
}

/// The typed answer a gesture earns. Exhaustive over the boundary's outcomes;
/// an error path is the `Err(String)` beside it, encoded by [`refusal`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// A short verb's captured run (§8.2) — `ok` iff exit 0.
    Outcome(Outcome),
    /// The `prepare` action's product: the composer's fire-time parameters.
    Prepared(Prepared),
    /// The `prompt` action's product: the minted conversation name (§3.3).
    Started {
        conversation: String,
    },
    /// The §3.6 unmaking completed.
    Deleted,
    /// The VISION §4.9 monitor's arming landed: `armed` says which way.
    Armed {
        armed: bool,
    },
    /// An attention item was raised on a conversation (VISION §4.9).
    Flagged,
    /// A parked invocation was answered (§8.6): which `tool_use` the answer
    /// landed on, the tool it names, the verdict written, and whether the
    /// releasing `lernie advance` was launched. It answers with the *held
    /// invocation* rather than the queue that remains (the `seen` precedent):
    /// the mark lifts only once the re-adjudication runs, so a queue read here
    /// would still show the park it just answered — a receipt that lied.
    Answered {
        tool_use: String,
        tool: String,
        ruling: crate::control::judge::Ruling,
        advanced: bool,
    },
    /// A conversation's capability floor was written (§8.6, VISION §4.9's
    /// fifth rung): whether one **stands** over it now — re-derived from the
    /// trail after the write, never an echo of the direction that was asked
    /// (the [`Marks`](Self::Marks) precedent). The two differ exactly where it
    /// matters: restoring a conversation whose ancestor is still floored leaves
    /// it floored, and a receipt saying otherwise would be a lie.
    Floored {
        standing: bool,
    },
    /// The §4.2 ack line landed — every current alarm is acknowledged.
    Acked,
    /// The trail was truncated; the clear is the fresh trail's first row.
    TrailCleared,
    /// A §9 config file landed: the path that now holds the staged text
    /// (bl-3f46). A lineage write answers with its `lernie config` run
    /// ([`Outcome`](Self::Outcome)) instead — a write and a spawn earn
    /// different receipts, here as everywhere else on the boundary.
    Applied {
        file: String,
    },
    /// The agent's tracking branch (§16.3), **re-read after the write**: what
    /// actually landed, beside the space root it landed in, never an echo of
    /// what was asked. The space is answered too because "which branch" and
    /// "whose branch" are one question — a branch name alone cannot tell the
    /// project's board from an agent's own universe of the same name.
    Marks {
        branch: String,
        space: PathBuf,
    },
    Workspaces(Vec<WsRow>),
    Conversations(Vec<ConvRow>),
    Balls(Vec<JoinRow>),
    /// The V4 board (VISION §5 V4) — the columns, their rows, and each row's
    /// gates, drones and figures.
    Board(Board),
    /// The §6 decision queue (VISION §5 V5.2): what is waiting on the operator.
    /// The answer to **both** `attention` and `seen` — an acknowledgement is
    /// answered by the queue that remains, never by an echo of what it wrote,
    /// so a teleoperator's loop is one gesture per decision rather than a read
    /// after every write (the [`Marks`](Self::Marks) precedent).
    Attention(Vec<crate::boundary::answer::queue::QueueRow>),
    Ops(Vec<OpRow>),
    /// What a command does (§8.5): the whole roster, or one verb's page.
    Help(Vec<crate::boundary::help::HelpRow>),
    /// What matched (§8.5): the ranked hits *and* the sources that could not be
    /// read, because an answer that hid the second half would be a lie about
    /// the first.
    Search(Found),
    /// What the workspace's attempts changed in their project (§5.1 #32), and
    /// the named file's patch when the query asked for one.
    WorkDiff {
        attempts: Vec<crate::workdiff::Attempt>,
        patch: Option<crate::files_view::Preview>,
    },
    /// One §9 destination's current bytes (§8.5, bl-0164) —
    /// [`ReadConfig`](super::Query::ReadConfig)'s answer, the file editors'
    /// Reload spelled.
    Config {
        text: String,
    },
    /// brazen's effective provider table with the §5.1 #22 credential
    /// presence (§8.5, bl-0164) — [`Providers`](super::Query::Providers)'
    /// answer, the §8.3 login pane's own rows.
    Providers(Vec<crate::config_edit::brazen::ProviderRowView>),
    /// The workspace's config lineages with each tip's files (§9.3, bl-dff8) —
    /// [`Lineages`](super::Query::Lineages)' answer, the config pane's two
    /// dropdowns.
    Lineages(Vec<crate::config_edit::branch::Lineage>),
    /// The model ids one provider offers (§9.4, bl-dff8) —
    /// [`Models`](super::Query::Models)' answer, the picker's roster. Never
    /// empty: a provider that offered nothing is a refusal saying so, not a
    /// list a seat would read as "no models exist".
    Models(Vec<String>),
}

/// Encode a reply to its file body. `ok` is the one field every reply carries.
pub fn encode(reply: &Reply) -> Value {
    match reply {
        Reply::Outcome(outcome) => json!({
            "ok": outcome.ok(), "kind": "outcome", "exit": outcome.exit,
            "stdout": outcome.stdout, "stderr": outcome.stderr,
        }),
        Reply::Prepared(prepared) => {
            json!({ "ok": true, "kind": "prepared", "prepared": prepared_value(prepared) })
        }
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
        Reply::Acked => json!({ "ok": true, "kind": "acked" }),
        Reply::TrailCleared => json!({ "ok": true, "kind": "trail-cleared" }),
        Reply::Applied { file } => json!({ "ok": true, "kind": "applied", "file": file }),
        // Both halves of the one answer: the branch, and the space it is a
        // branch of — the gesture's own two words, in the shape it sets them.
        Reply::Marks { branch, space } => {
            json!({ "ok": true, "kind": "marks", "branch": branch,
                    "space": space.display().to_string() })
        }
        Reply::Workspaces(rows) => rows_reply("workspaces", rows.iter().map(ws_row)),
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
        // The one encoder written beside its type rather than here: the rows'
        // shape is `workdiff`'s own vocabulary (see that module's `wire`).
        Reply::WorkDiff { attempts, patch } => {
            crate::workdiff::wire::reply(attempts, patch.as_ref())
        }
        Reply::Search(found) => json!({
            "ok": true, "kind": "search",
            "rows": found.hits.iter().map(hit_row).collect::<Vec<Value>>(),
            "unreadable": found.unreadable,
        }),
        Reply::Config { text } => json!({ "ok": true, "kind": "config", "text": text }),
        Reply::Providers(rows) => rows_reply("providers", rows.iter().map(provider_row)),
        Reply::Lineages(rows) => rows_reply("lineages", rows.iter().map(lineage_row)),
        // The one listing whose row is a bare id: a model has no other fact
        // yog knows — brazen publishes an id and a default flag, and which one
        // is default is a `providers.yaml` question, not a roster one (§9.4).
        Reply::Models(ids) => rows_reply("models", ids.iter().map(|id| json!(id))),
    }
}

/// One provider row as the operator reads it (§8.3, §9.5): its name, the
/// credential fact in words, and why `bz --login` cannot serve it — `null`
/// exactly when it can.
fn provider_row(row: &crate::config_edit::brazen::ProviderRowView) -> Value {
    json!({ "name": row.name, "fact": row.fact, "blocked": row.blocked })
}

/// Whether a dispatch outcome was clean — the draft-clearing predicate the
/// composer reads (§5.3: RAM until *sent*): a captured run must have exited 0;
/// any other reply is its action's success by construction; a refusal is not.
pub fn cleared(result: &Result<Reply, String>) -> bool {
    match result {
        Ok(Reply::Outcome(outcome)) => outcome.ok(),
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Encode a refusal — a gesture that never ran (decode failure, gate refusal,
/// executor error). The error text is the §7.3 story; the ops trail carries
/// whatever the executor logged before refusing.
pub fn refusal(error: &str) -> Value {
    json!({ "ok": false, "error": error })
}

fn rows_reply(kind: &str, rows: impl Iterator<Item = Value>) -> Value {
    Value::Object(rows_map(kind, rows))
}

/// The same, still open for a family that carries one more key.
fn rows_map(kind: &str, rows: impl Iterator<Item = Value>) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!(kind));
    map.insert("rows".to_owned(), json!(rows.collect::<Vec<Value>>()));
    map
}

/// One help page as data — the same four facts every seat renders (§8.5).
fn help_row(row: &crate::boundary::help::HelpRow) -> Value {
    json!({ "verb": row.verb, "usage": row.usage,
            "summary": row.summary, "detail": row.detail })
}

#[cfg(test)]
mod tests;
