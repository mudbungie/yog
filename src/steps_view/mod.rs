//! Per-agent steps inspector view-model (DESIGN §11 Altitude-2 Steps tab;
//! §5.1 #13; §15 Y13). Milestone M2's last piece: browse every byte.
//!
//! A step is `steps/<agent-id>/NNN/` (ARCH §2.3): `meta.json`
//! (`{commit, started_at, ended_at}`), `request.json` (the wire request
//! snapshot), `response.json` (JSONL of §4.4 events), `staging.json` (the
//! transcript entry under construction), `stderr.log` (the model adapter's own
//! stderr — empty on an ordinary run) and `tools/<tool-id>/{input,output}
//! .json`. Yog is a pure reader (§3.5): everything here is a function of
//! those bytes, re-read per tick, deriving nothing it can read.
//!
//! Two tiers, so the cheap list never pays for the heavy drill-in:
//! [`build`] summarizes every step (framing, attempts, tokens, timestamps)
//! for the list — that is this module; [`detail`] parses one selected step's
//! files into jsonview trees on demand — that is the `detail` submodule, split
//! along this same seam to keep both tiers clear of the 300-line cap (§12).
//! The render side is cut the same way: `render` paints the headed step table
//! (its columns — header, hover explanation and cell in one home — are the
//! `columns` submodule), and `drill` paints the selected step's records.
//!
//! Nothing here re-parses a record another module already owns
//! (single source of truth): per-step **framing** and **attempt count** come
//! from the git_tree §4.4 terminal classifier, and **token counts** from the
//! budgets Usage fold — both reused, never duplicated (§15 Y13).

use std::path::Path;

use serde_json::Value;

use crate::budgets::{BudgetSpend, spend_from_bytes};
use crate::git_tree::{AgentState, Framing, framing, segment_count};
use crate::login::auth::{AuthFailure, row_of_model};

mod columns;
mod detail;
mod drill;
mod orphan;
mod render;
pub(crate) mod wire;
mod wound;
pub use detail::{Doc, StepDetail, ToolIo, UNPARSED, detail};
pub(crate) use drill::RECORDS;
pub use orphan::{ORPHANED_MAIL, Orphan};
pub use render::{StepTab, render};
pub use wound::{NO_RESPONSE, Wound, latest_wound};

/// Conv-repo subdir of per-agent step records (ARCH §2.3).
const STEPS_DIR: &str = "steps";
const META_FILE: &str = "meta.json";
const REQUEST_FILE: &str = "request.json";
const RESPONSE_FILE: &str = "response.json";
const STAGING_FILE: &str = "staging.json";
const TOOLS_SUBDIR: &str = "tools";
const INPUT_FILE: &str = "input.json";
const OUTPUT_FILE: &str = "output.json";
/// Zero-padded step-sequence width (`001`, `002`, …) per ARCH §2.3.
const STEP_SEQ_WIDTH: usize = 3;

/// One step's list-row summary. `framing` and `tokens` are reused from the
/// git_tree terminal classifier and the budgets Usage fold; the rest reads
/// `meta.json` (§2.3 `{commit, started_at, ended_at}`), each field absent
/// when `meta.json` is missing or malformed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepSummary {
    /// Zero-padded sequence directory name (`001`).
    pub seq: String,
    /// §4.4 outcome of `response.json` — complete / failed / killed.
    pub framing: Framing,
    /// Completed attempt segments (`end` events, §4.4).
    pub attempts: usize,
    /// Whole-segment token spend for this step (§6).
    pub tokens: BudgetSpend,
    /// Branch-tip sha at step-start (`meta.commit`, §2.10).
    pub commit: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    /// The **Login affordance** (§8.3 detection, §15 M6 Z8): whether this step is
    /// an auth-shaped failure — framing Failed with credential/auth-class error
    /// text — and the provider row its remedy points at
    /// ([`crate::login::auth::AuthFailure`]). When offered, the shell paints Login
    /// one click away beside the step (a prompt-time failure surfaces here as
    /// derived agent state, §13.3). Logic covered; shell paints.
    pub auth_failed: AuthFailure,
    /// The §7.3 **no-response wound** (the `wound` module): this step's driver
    /// produced nothing — no response bytes and no settled `meta.json` — and
    /// nobody is driving the agent. Renders as a failure row
    /// ([`NO_RESPONSE`], ichor) instead of the quiet ash "stopped" its framing
    /// alone reads as, and carries the adapter's own reason from the step's
    /// `stderr.log` when there is one (bl-55d8).
    pub wound: Wound,
}

/// The ordered per-step summaries for one agent's `steps/<agent-id>/` tree,
/// plus the view-level **orphaned-mail state** (bl-ace6, the `orphan`
/// module): a delivered message nobody is answering, which no per-step
/// field can carry because the driver that died never created a step.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StepsView {
    pub steps: Vec<StepSummary>,
    pub orphan: Orphan,
}

/// Summarize every step of `agent_id` in `workspace`, in sequence order. A
/// missing `steps/<agent-id>/` tree yields an empty view.
///
/// `state` is the agent's already-derived §3.5 liveness — the second half of
/// the `wound` rule. A driver at work is still filling its newest step, so
/// that one step's unanswered shape is a call in flight, not a wound (§10:
/// never a false definite). Every other field is a pure read of the step's own
/// bytes and ignores `state`.
pub fn build(workspace: &Path, agent_id: &str, state: AgentState) -> StepsView {
    let mut steps: Vec<StepSummary> = step_seqs(workspace, agent_id)
        .into_iter()
        .map(|seq| summarize(workspace, agent_id, &seq))
        .collect();
    if wound::driven(state)
        && let Some(newest) = steps.last_mut()
    {
        newest.wound = Wound::None;
    }
    route_auth(workspace, agent_id, &mut steps);
    StepsView {
        steps,
        orphan: orphan::read(workspace, agent_id, state),
    }
}

/// Upgrade every auth-shaped step from `Unrouted` to the provider row it failed
/// on (bl-8e34), so the Login affordance beside it names what to log in to.
///
/// The whole derivation is here rather than in [`summarize`] because it is the
/// one **git** read in this module: the agent's governing config commit, asked
/// once for the roles it declares (`fork::roles_at` — §9.4's own grammar
/// reader, so the picker and this can never disagree about a file). It is paid
/// only when a step is actually failing, and never per step: a healthy agent's
/// view costs exactly what it did before. Each failing step then contributes
/// its own model id (its `request.json`, read for that step alone), and the
/// join is [`row_of_model`].
fn route_auth(workspace: &Path, agent_id: &str, steps: &mut [StepSummary]) {
    if !steps.iter().any(|step| step.auth_failed.offered()) {
        return;
    }
    let roles = crate::fork::roles_at(workspace, &format!("agents/{agent_id}"));
    for step in steps.iter_mut().filter(|s| s.auth_failed.offered()) {
        if let Some(row) = step_model(workspace, agent_id, &step.seq)
            .and_then(|model| row_of_model(&model, &roles))
        {
            step.auth_failed = AuthFailure::Row(row);
        }
    }
}

/// The model id a step's `request.json` was dispatched with — the only place a
/// step records which model it asked for (ARCH §4.2: the id rides the canonical
/// request verbatim). Absent or malformed bytes are `None`, like every other
/// record read here.
fn step_model(workspace: &Path, agent_id: &str, seq: &str) -> Option<String> {
    // Bound rather than chained, like `summarize` above: tarpaulin's llvm engine
    // mis-attributes a multi-line method chain's tail as uncovered.
    let step = workspace.join(STEPS_DIR).join(agent_id).join(seq);
    let bytes = std::fs::read(step.join(REQUEST_FILE)).ok()?;
    let request: Value = serde_json::from_slice(&bytes).ok()?;
    request.get("model")?.as_str().map(str::to_string)
}

/// The zero-padded `NNN` step dirs under `steps/<agent-id>/`, numeric order.
/// Non-step entries (stray files, odd names) are skipped; an absent tree is
/// empty. Lexicographic sort over fixed-width digits is numeric order.
fn step_seqs(workspace: &Path, agent_id: &str) -> Vec<String> {
    let dir = workspace.join(STEPS_DIR).join(agent_id);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut seqs: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let is_seq = name.len() == STEP_SEQ_WIDTH
                && name.bytes().all(|b| b.is_ascii_digit())
                && entry.path().is_dir();
            is_seq.then_some(name)
        })
        .collect();
    seqs.sort();
    seqs
}

fn summarize(workspace: &Path, agent_id: &str, seq: &str) -> StepSummary {
    let step = workspace.join(STEPS_DIR).join(agent_id).join(seq);
    let response = std::fs::read(step.join(RESPONSE_FILE)).unwrap_or_default();
    let meta_bytes = std::fs::read(step.join(META_FILE)).ok();
    let meta = meta_bytes
        .as_ref()
        .and_then(|bytes| serde_json::from_slice::<Value>(bytes).ok());
    StepSummary {
        seq: seq.to_string(),
        framing: framing(&response),
        attempts: segment_count(&response),
        tokens: spend_from_bytes(&response),
        commit: meta_field(meta.as_ref(), "commit"),
        started_at: meta_field(meta.as_ref(), "started_at"),
        ended_at: meta_field(meta.as_ref(), "ended_at"),
        auth_failed: crate::login::auth::classify(&response),
        wound: wound::read(&step, &response, meta_bytes.is_some()),
    }
}

/// A string field of the parsed `meta.json`, or `None` when meta is
/// absent/malformed or the field is missing / non-string.
fn meta_field(meta: Option<&Value>, key: &str) -> Option<String> {
    meta?.get(key)?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests;
