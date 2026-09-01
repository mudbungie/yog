//! The row encoders (§8.5) — one function per listed thing, split from the
//! [reply roster](super) at §12's per-file budget on the same seam
//! [`super::board`] already took: [`super`] holds the [`Reply`](super::Reply)
//! enum and which encoder each variant spends, this holds what one row of each
//! listing looks like on the wire.

use serde_json::{Map, Value, json};

use super::WsRow;
use crate::binding::WorkspaceKind;
use crate::git_tree::AgentState;
use crate::monitor::Check;
use crate::nav::convs::Tone;
use crate::nav::convs::{ConvRow, Flight};
use crate::opslog::OpRow;
use crate::projects::join::JoinRow;

use super::super::codec::join_token;
use super::super::codec::origin_token;

/// The decoders, beside the tokens they undo (bl-7067).
pub(crate) mod decode;

/// One workspace row: its **name** and its §3.1 classification (REMOTE §8,
/// bl-f5f6 — the path it used to carry beside them is neither meaningful nor
/// safe on a thin client), then the §6 rollups.
pub(super) fn ws_row(row: &WsRow) -> Value {
    let mut map = Map::new();
    map.insert("workspace".to_owned(), Value::String(row.workspace.clone()));
    let kind = match &row.kind {
        WorkspaceKind::Named { .. } => "named",
        WorkspaceKind::Foreign => "foreign",
        WorkspaceKind::Replay => "replay",
    };
    map.insert("kind".to_owned(), Value::String(kind.to_owned()));
    map.insert("attention".to_owned(), json!(row.attention));
    map.insert("agents".to_owned(), json!(row.agents));
    map.insert("running".to_owned(), json!(row.running));
    // The §4.1 pin rank, **absent** rather than null for an unpinned workspace
    // (bl-296f) — a reader must never have to tell "rank 0" from "not pinned",
    // and rank 0 is the first hoisted tab.
    if let Some(rank) = row.pinned {
        map.insert("pinned".to_owned(), json!(rank));
    }
    // The §2.2 lineage tip (bl-b4b5), both oids: short is what the §9.4 picker
    // labels the freeze with, full is what a `git show` outside yog takes — the
    // lineage row's own shape one noun over. Absent for a workspace with no
    // lineage derived yet.
    if let Some(tip) = &row.config_tip {
        map.insert(
            "config_tip".to_owned(),
            json!({ "oid": tip.oid, "short_oid": tip.short_oid }),
        );
    }
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
    // uses it as a `message` target, and litany resolves by exact id else
    // unique *stored* name. A legacy-rung title is goal-stamp prose no stored
    // fact backs, so it is withheld here rather than handed over as a target
    // litany will refuse — `display` above still carries the §3.3 ladder's
    // answer, and `root_id` is the address that always works.
    if let Some(name) = row.name.as_ref().filter(|_| !row.name_display_only) {
        map.insert("name".to_owned(), json!(name));
    }
    // The display-only rung as its own fact (bl-7067). Withholding `name`
    // said "you cannot message this" but not "there is a name" — so the row
    // could not be read back as the row that was answered. The name itself is
    // not re-added: `display` above already IS it whenever this is true
    // (`display_name`'s first rung is the name, verbatim), and a second copy
    // of one string is the thing this codec is not allowed to grow.
    map.insert("display_only".to_owned(), json!(row.name_display_only));
    map.insert("state".to_owned(), json!(state_token(row.state)));
    map.insert("uncertain".to_owned(), json!(row.uncertain));
    map.insert("preview".to_owned(), json!(row.preview));
    map.insert("age_secs".to_owned(), json!(row.age_secs));
    // The same recency fact undistanced (REMOTE §9.9, bl-b7d9): an age alone
    // orders a roster and cannot stamp one. Epoch seconds, the unit every
    // other time on this wire speaks (`committed` above); it rides *beside*
    // `age_secs` because the age is the only carrier of the engine's own clock
    // at answer time, and the pair is one value encoded twice at one instant.
    map.insert("last_active_unix".to_owned(), json!(row.last_active_unix));
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
    // The row's own §8.2 gates (REMOTE §9.4, bl-1eb0). Neither is derivable
    // from anything else here: `state` is the badge aggregated over the whole
    // subtree, so a quiet root with a working child reads Live and has no
    // driver to kill; and the cascade's membership is the Stop menu's looser
    // prefix test, not the strict §5.1 #8 descent `direct` counts.
    map.insert("stoppable".to_owned(), json!(row.stoppable));
    map.insert("stop_children".to_owned(), json!(row.stop_children));
    // How far the row hangs under its conversation root (§11's indent) and how
    // solidly it paints (§11, bl-915e) — both on the wire since bl-7067,
    // because a seat that reads rows and cannot indent them, or cannot tell
    // yog's own pending word from the derivation's, paints a different list
    // than the window does of the same instant.
    map.insert("depth".to_owned(), json!(row.depth));
    map.insert("tone".to_owned(), json!(tone_token(row.tone)));
    // The standing alignment verdict (VISION §4.9 rung V6), absent — not
    // null — for a conversation no armed monitor has ruled on.
    if let Some(check) = &row.verdict {
        map.insert("alignment".to_owned(), check_value(check));
    }
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

/// One §3.5 binding fact. **Both addresses are names** since bl-b4b5 (REMOTE
/// §8.1): the project's §5.1 #1 wire name and the workspace's §3.1 leaf, which
/// is what the row carries now rather than two absolute paths under the
/// engine's home — the last payload residual on that list after
/// `Prepared::binding`.
pub(super) fn join_row(row: &JoinRow) -> Value {
    let mut map = Map::new();
    map.insert("project".to_owned(), json!(row.project));
    map.insert("ball_id".to_owned(), json!(row.ball_id));
    map.insert("state".to_owned(), json!(join_token(row.state)));
    if let Some(ws) = &row.workspace {
        map.insert("workspace".to_owned(), json!(ws));
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

/// The §11 row tone (bl-915e), in the words the two seats share.
pub(super) fn tone_token(tone: Tone) -> &'static str {
    match tone {
        Tone::Plain => "plain",
        Tone::Weak => "weak",
        Tone::Good => "good",
        Tone::Bad => "bad",
        Tone::Live => "live",
        Tone::InFlight => "in-flight",
    }
}

/// One standing alignment check (VISION §4.9): the ruling, the tip it read,
/// the sentence behind it, and the model that said so. The token counts are
/// absent — not zero — when the adapter reported none.
fn check_value(check: &Check) -> Value {
    let mut map = Map::new();
    map.insert("workspace".to_owned(), json!(check.workspace));
    map.insert("agent".to_owned(), json!(check.agent));
    map.insert("verdict".to_owned(), json!(check.verdict.token()));
    map.insert("sha".to_owned(), json!(check.sha));
    map.insert("reason".to_owned(), json!(check.reason));
    map.insert("model".to_owned(), json!(check.model));
    for (key, count) in [
        ("input_tokens", check.input_tokens),
        ("output_tokens", check.output_tokens),
    ] {
        if let Some(count) = count {
            map.insert(key.to_owned(), json!(count));
        }
    }
    Value::Object(map)
}

pub(super) fn flight_token(flight: Flight) -> &'static str {
    match flight {
        Flight::Inference => "inference",
        Flight::Tools => "tools",
        Flight::Subagents => "subagents",
    }
}
