//! The row **decoders** (§8.5, REMOTE §9 step 2, bl-7067) — one per encoder in
//! [`super`], in the same order, so the two directions of one row's spelling
//! sit a screen apart and cannot drift.
//!
//! Strict, exactly as the gesture codec's decode is: a missing field, a
//! mistyped value and an unknown token each refuse **naming the offender**.
//! A reply read off the wire is a peer's statement about a world this process
//! cannot see — the forgiving-parse discipline of an `ops.jsonl` read applies
//! to bytes yog wrote itself, never to bytes it is told.

use serde_json::Value;

use super::super::WsRow;
use crate::binding::WorkspaceKind;
use crate::boundary::codec::fields::{
    bool_of, i64_of, list_of, opt, opt_str_of, opt_val, pick, str_of, strings_of, u64_of, usize_of,
};
use crate::boundary::codec::{parse_join, parse_origin};
use crate::config_edit::branch::{ConfigBranch, Lineage};
use crate::config_edit::brazen::ProviderRowView;
use crate::git_tree::AgentState;
use crate::monitor::{Check, Verdict};
use crate::nav::convs::Tone;
use crate::nav::convs::{ConvBall, ConvRow, Flight};
use crate::opslog::OpRow;
use crate::projects::join::JoinRow;

/// The §5.1 agent-state table — [`state_token`](super::state_token)'s other
/// half. Two spellings of one vocabulary is the tree's standing shape here
/// (`join_token`/`parse_join`, `Verdict::token`/`Verdict::parse`): the match is
/// the compile gate, the table is the parser, and the round-trip test over
/// every arm is what holds them together.
const STATES: [(&str, AgentState); 4] = [
    ("live", AgentState::Live),
    ("in-flight", AgentState::InFlight),
    ("quiescent", AgentState::Quiescent),
    ("stopped", AgentState::Stopped),
];

pub(crate) const FLIGHTS: [(&str, Flight); 3] = [
    ("inference", Flight::Inference),
    ("tools", Flight::Tools),
    ("subagents", Flight::Subagents),
];

const TONES: [(&str, Tone); 6] = [
    ("plain", Tone::Plain),
    ("weak", Tone::Weak),
    ("good", Tone::Good),
    ("bad", Tone::Bad),
    ("live", Tone::Live),
    ("in-flight", Tone::InFlight),
];

/// The §5.1 state a row's `state` key names.
pub(crate) fn state_of(obj: &serde_json::Map<String, Value>) -> Result<AgentState, String> {
    pick(obj, "state", &STATES)
}

pub(crate) fn ws_row(v: &Value) -> Result<WsRow, String> {
    let o = v.as_object().ok_or("workspace row: not an object")?;
    let workspace = str_of(o, "workspace")?;
    // A named workspace's §3.1 name **is** the row's own, so the kind carries
    // no second copy of it (REMOTE §8, bl-f5f6).
    let kind = match str_of(o, "kind")?.as_str() {
        "named" => WorkspaceKind::Named {
            name: workspace.clone(),
        },
        "foreign" => WorkspaceKind::Foreign,
        "replay" => WorkspaceKind::Replay,
        other => return Err(format!("workspace row: unknown kind {other:?}")),
    };
    Ok(WsRow {
        workspace,
        kind,
        attention: usize_of(o, "attention")?,
        agents: usize_of(o, "agents")?,
        running: bool_of(o, "running")?,
        pinned: opt(o, "pinned", usize_of)?,
        config_tip: opt_val(o, "config_tip", config_tip)?,
    })
}

/// The §2.2 lineage tip, read back on the two oids the encoder writes.
fn config_tip(v: &Value) -> Result<crate::model_pick::ConfigTip, String> {
    let o = v.as_object().ok_or("config tip: not an object")?;
    Ok(crate::model_pick::ConfigTip {
        oid: str_of(o, "oid")?,
        short_oid: str_of(o, "short_oid")?,
    })
}

pub(crate) fn lineage_row(v: &Value) -> Result<Lineage, String> {
    let o = v.as_object().ok_or("lineage row: not an object")?;
    Ok(Lineage {
        branch: ConfigBranch {
            name: str_of(o, "name")?,
            tip_oid: str_of(o, "oid")?,
            tip_short_oid: str_of(o, "short_oid")?,
            tip_timestamp_unix: i64_of(o, "committed")?,
        },
        files: strings_of(o, "files")?,
    })
}

pub(crate) fn conv_row(v: &Value) -> Result<ConvRow, String> {
    let o = v.as_object().ok_or("conversation row: not an object")?;
    // The display-only rung's name is read back off `display` (bl-7067): the
    // encoder withholds `name` there on purpose, and `display` is that same
    // string by construction, so this recovers the fact without the wire
    // carrying it twice.
    let display_only = bool_of(o, "display_only")?;
    let name = if display_only {
        Some(str_of(o, "display")?)
    } else {
        opt_str_of(o, "name")?
    };
    Ok(ConvRow {
        root_id: str_of(o, "root_id")?,
        state: state_of(o)?,
        uncertain: bool_of(o, "uncertain")?,
        preview: str_of(o, "preview")?,
        age_secs: i64_of(o, "age_secs")?,
        last_active_unix: i64_of(o, "last_active_unix")?,
        flight: opt(o, "flight", |o, k| pick(o, k, &FLIGHTS))?,
        attention: usize_of(o, "attention")?,
        members: usize_of(o, "members")?,
        depth: usize_of(o, "depth")?,
        direct: usize_of(o, "direct")?,
        stoppable: bool_of(o, "stoppable")?,
        stop_children: bool_of(o, "stop_children")?,
        ball: opt_val(o, "ball", conv_ball)?,
        name,
        name_display_only: display_only,
        verdict: opt_val(o, "alignment", check)?,
        tone: pick(o, "tone", &TONES)?,
        failure: opt_str_of(o, "failure")?,
    })
}

fn conv_ball(v: &Value) -> Result<ConvBall, String> {
    let o = v.as_object().ok_or("conversation ball: not an object")?;
    Ok(ConvBall {
        id: str_of(o, "id")?,
        state: opt(o, "state", |o, k| parse_join(&str_of(o, k)?))?,
        title: opt_str_of(o, "title")?,
        badge: opt_str_of(o, "badge")?,
    })
}

fn check(v: &Value) -> Result<Check, String> {
    let o = v.as_object().ok_or("alignment: not an object")?;
    let word = str_of(o, "verdict")?;
    Ok(Check {
        workspace: str_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        verdict: Verdict::parse(&word).ok_or_else(|| format!("unknown verdict {word:?}"))?,
        sha: str_of(o, "sha")?,
        reason: str_of(o, "reason")?,
        model: str_of(o, "model")?,
        input_tokens: opt(o, "input_tokens", u64_of)?,
        output_tokens: opt(o, "output_tokens", u64_of)?,
    })
}

pub(crate) fn join_row(v: &Value) -> Result<JoinRow, String> {
    let o = v.as_object().ok_or("ball row: not an object")?;
    Ok(JoinRow {
        project: str_of(o, "project")?,
        ball_id: str_of(o, "ball_id")?,
        state: parse_join(&str_of(o, "state")?)?,
        workspace: opt_str_of(o, "workspace")?,
        claimant: opt_str_of(o, "claimant")?,
        title: opt_str_of(o, "title")?,
    })
}

pub(crate) fn op_row(v: &Value) -> Result<OpRow, String> {
    let o = v.as_object().ok_or("ops row: not an object")?;
    Ok(OpRow {
        ts: str_of(o, "ts")?,
        argv: str_of(o, "argv")?,
        cwd: str_of(o, "cwd")?,
        exit: i32::try_from(i64_of(o, "exit")?).map_err(|_| "ops row: exit out of range")?,
        stdout: str_of(o, "stdout")?,
        stderr: str_of(o, "stderr")?,
        origin: parse_origin(&str_of(o, "origin")?)?,
    })
}

/// The provider facts, the §5.1 #22 credential answer beside them, and the two
/// §9.4 tuning capabilities (bl-23bd) — read **strictly**, as the required
/// booleans the encoder always writes, so a peer that dropped them refuses in
/// band instead of quietly presenting a row with both controls hidden.
pub(crate) fn provider_row(v: &Value) -> Result<ProviderRowView, String> {
    let o = v.as_object().ok_or("provider row: not an object")?;
    Ok(ProviderRowView {
        name: str_of(o, "name")?,
        fact: str_of(o, "fact")?,
        blocked: opt_str_of(o, "blocked")?,
        effort: bool_of(o, "effort")?,
        priority: bool_of(o, "priority")?,
    })
}

/// A listing's `rows` array, each element read by `read`.
pub(crate) fn rows_of<T>(
    obj: &serde_json::Map<String, Value>,
    read: impl Fn(&Value) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    list_of(obj, "rows", read)
}
