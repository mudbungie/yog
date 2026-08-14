//! The row encoders (§8.5) — one function per listed thing, split from the
//! [reply roster](super) at §12's per-file budget on the same seam
//! [`super::board`] already took: [`super`] holds the [`Reply`](super::Reply)
//! enum and which encoder each variant spends, this holds what one row of each
//! listing looks like on the wire.

use serde_json::{Map, Value, json};

use super::WsRow;
use crate::binding::WorkspaceKind;
use crate::git_tree::AgentState;
use crate::nav::convs::{ConvRow, Flight};
use crate::opslog::OpRow;
use crate::projects::join::JoinRow;

use super::super::codec::join_token;
use super::super::codec::origin_token;

pub(super) fn ws_row(row: &WsRow) -> Value {
    let mut map = Map::new();
    map.insert(
        "workspace".to_owned(),
        Value::String(row.workspace.path.to_string_lossy().into_owned()),
    );
    let (kind, name) = match &row.workspace.kind {
        WorkspaceKind::Named { name } => ("named", Some(name.clone())),
        WorkspaceKind::Foreign => ("foreign", None),
        WorkspaceKind::Replay => ("replay", None),
    };
    map.insert("kind".to_owned(), Value::String(kind.to_owned()));
    if let Some(name) = name {
        map.insert("name".to_owned(), Value::String(name));
    }
    map.insert("attention".to_owned(), json!(row.attention));
    map.insert("agents".to_owned(), json!(row.agents));
    map.insert("running".to_owned(), json!(row.running));
    Value::Object(map)
}

/// One lineage as the §9.3 browse answers it (bl-dff8): the branch's bare name
/// — the word `/config branch <lineage> …` takes — its tip, and the paths that
/// tip holds. The tip is answered **both** short and full: the short oid is what
/// the pane labels a lineage with, the full one is what a `git show` outside yog
/// takes.
pub(super) fn lineage_row(row: &crate::config_edit::branch::Lineage) -> Value {
    json!({
        "name": row.branch.name,
        "oid": row.branch.tip_oid,
        "short_oid": row.branch.tip_short_oid,
        "committed": row.branch.tip_timestamp_unix,
        "files": row.files,
    })
}

pub(super) fn conv_row(row: &ConvRow) -> Value {
    let mut map = Map::new();
    map.insert("root_id".to_owned(), json!(row.root_id));
    map.insert("display".to_owned(), json!(row.display_name()));
    // `name` is the **addressable** name (bl-8068): a peer reading this row
    // uses it as a `message` target, and lernie resolves by exact id else
    // unique *stored* name. A legacy-rung title is goal-stamp prose no stored
    // fact backs, so it is withheld here rather than handed over as a target
    // lernie will refuse — `display` above still carries the §3.3 ladder's
    // answer, and `root_id` is the address that always works.
    if let Some(name) = row.name.as_ref().filter(|_| !row.name_display_only) {
        map.insert("name".to_owned(), json!(name));
    }
    map.insert("state".to_owned(), json!(state_token(row.state)));
    map.insert("uncertain".to_owned(), json!(row.uncertain));
    map.insert("preview".to_owned(), json!(row.preview));
    map.insert("age_secs".to_owned(), json!(row.age_secs));
    if let Some(flight) = row.flight {
        map.insert("flight".to_owned(), json!(flight_token(flight)));
    }
    map.insert("attention".to_owned(), json!(row.attention));
    map.insert("members".to_owned(), json!(row.members));
    // The subagent field's first number (bl-fa82's spec decision): the machine
    // surface gains the strict §5.1 #8 child count so a reader need not
    // re-derive the descent grammar. The *fold* state has no home here — the
    // answer stays root rows, and expansion is a viewport's (§13.1).
    map.insert("direct".to_owned(), json!(row.direct));
    if let Some(ball) = &row.ball {
        let mut b = Map::new();
        b.insert("id".to_owned(), json!(ball.id));
        if let Some(state) = ball.state {
            b.insert("state".to_owned(), json!(join_token(state)));
        }
        if let Some(title) = &ball.title {
            b.insert("title".to_owned(), json!(title));
        }
        if let Some(badge) = &ball.badge {
            b.insert("badge".to_owned(), json!(badge));
        }
        map.insert("ball".to_owned(), Value::Object(b));
    }
    Value::Object(map)
}

pub(super) fn join_row(row: &JoinRow) -> Value {
    let mut map = Map::new();
    map.insert(
        "project".to_owned(),
        json!(row.project.to_string_lossy().into_owned()),
    );
    map.insert("ball_id".to_owned(), json!(row.ball_id));
    map.insert("state".to_owned(), json!(join_token(row.state)));
    if let Some(ws) = &row.workspace {
        map.insert(
            "workspace".to_owned(),
            json!(ws.to_string_lossy().into_owned()),
        );
    }
    if let Some(claimant) = &row.claimant {
        map.insert("claimant".to_owned(), json!(claimant));
    }
    if let Some(title) = &row.title {
        map.insert("title".to_owned(), json!(title));
    }
    Value::Object(map)
}

pub(super) fn op_row(row: &OpRow) -> Value {
    json!({
        "ts": row.ts, "argv": row.argv, "cwd": row.cwd, "exit": row.exit,
        "stdout": row.stdout, "stderr": row.stderr,
        "origin": origin_token(row.origin),
    })
}

/// The §5.1 agent-state tokens.
pub(crate) fn state_token(state: AgentState) -> &'static str {
    match state {
        AgentState::Live => "live",
        AgentState::InFlight => "in-flight",
        AgentState::Quiescent => "quiescent",
        AgentState::Stopped => "stopped",
    }
}

pub(super) fn flight_token(flight: Flight) -> &'static str {
    match flight {
        Flight::Inference => "inference",
        Flight::Tools => "tools",
        Flight::Subagents => "subagents",
    }
}
