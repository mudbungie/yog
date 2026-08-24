//! Commit-node and agent construction. Bridges raw git output (see
//! [`super::cmd`]) into the view-model's [`CommitNode`] and [`Agent`]
//! shapes.
//!
//! The trunk is the config lineage (§2.2); agents are the `agents/*` refs
//! (§2.3), never merged anywhere (§2.6). Per-agent disk reads (the goal's two
//! stamps and its preview, streaming text, tool calls, the pending-deposit
//! listing — §5.1 #11 via [`crate::inboxview::list_inbox`], the one reader)
//! come from the workspace root (`agents/<agent-id>/goal.md`,
//! `steps/<agent-id>/…`, `inbox/<agent-id>/`, §2.2/§2.11); the §3.5 state and
//! the four ref-derived marks (conflicted, budget-exhausted, abandoned, notify)
//! are classified here.

use super::cmd::{LogEntry, for_each_ref_agents, ref_name, walk_branch_steps};
use super::detect::payload_headline;
use super::marks::Marks;
use super::probe::{LockProbe, WriterProbe};
use super::state::classify;
use super::streaming::{RESPONSE_FILE, latest_step_dir, stream_from_disk};
use super::tools::tool_calls_from_disk;
use super::{AGENTS_DIR, Agent, CommitNode, GitTreeError, MESSAGES_DIR, STEPS_DIR};
use std::path::Path;

pub(super) fn build_node(entry: LogEntry) -> CommitNode {
    let LogEntry {
        oid,
        timestamp,
        subject,
    } = entry;
    let short_oid = oid.get(..8).unwrap_or(&oid).to_string();
    CommitNode {
        oid,
        short_oid,
        timestamp_unix: timestamp,
        subject,
    }
}

pub(super) fn enumerate_agents(
    workspace: &Path,
    git_dir: &Path,
    lock: &dyn LockProbe,
    writer: &dyn WriterProbe,
) -> Result<Vec<Agent>, GitTreeError> {
    let out = for_each_ref_agents(git_dir)?;
    let text = String::from_utf8_lossy(&out);
    let marks = Marks::from_repo(git_dir)?;
    let mut agents = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, ' ');
        let branch_name = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?
            .to_string();
        let tip_oid = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?
            .to_string();
        let ts_str = parts
            .next()
            .ok_or_else(|| GitTreeError::LogFormat(line.to_string()))?;
        let tip_ts: i64 = ts_str
            .parse()
            .map_err(|_| GitTreeError::LogFormat(line.to_string()))?;
        // Agent refs are `agents/<id>` (§2.3); the id is the identity
        // everywhere else (steps/, inbox/, worktree dir, descent).
        let agent_id = branch_name
            .strip_prefix("agents/")
            .unwrap_or(&branch_name)
            .to_string();
        let tip_short_oid = tip_oid.get(..8).unwrap_or(&tip_oid).to_string();
        let steps = walk_branch_steps(git_dir, &branch_name)?;
        let liveness = classify(workspace, &agent_id, lock, writer);
        let goal = goal_from_disk(workspace, &agent_id);
        // One readdir of `messages/` for both of its facts (§5.1 #12): when the
        // transcript last moved, and how many entries it holds.
        let messages = messages_from_disk(workspace, &agent_id);
        let last_action = last_action_from_disk(workspace, &agent_id, tip_ts, messages.newest_unix);
        // One read of the live `response.json` for both of its facts (§5.1
        // #10, #28b) — what the model has said, and what it is doing.
        let stream = stream_from_disk(workspace, &agent_id);
        agents.push(Agent {
            name: ref_name(git_dir, &branch_name)?,
            preview: goal.preview,
            stream,
            tool_calls: tool_calls_from_disk(workspace, &agent_id),
            state: liveness.state,
            state_uncertain: liveness.uncertain,
            truncated: liveness.truncated,
            pending: crate::inboxview::list_inbox(workspace, &agent_id),
            conflicted_oid: marks.conflicted_oid(&agent_id),
            budget_oid: marks.budget_oid(&agent_id),
            abandoned_oid: marks.abandoned_oid(&agent_id),
            notify_oid: marks.notify_oid(&agent_id),
            held: marks.held(&agent_id),
            goal_ball: goal.ball,
            goal_name: goal.name,
            call_start_unix: call_start_from_disk(workspace, &agent_id),
            branch_name,
            agent_id,
            tip_oid,
            tip_short_oid,
            tip_timestamp_unix: tip_ts,
            last_action_unix: last_action,
            messages: messages.count,
            steps,
        });
    }
    Ok(agents)
}

/// The agent's last action of any kind (§11 recency, bl-cad5): the newest of
/// the **tip commit timestamp** (a committed step), the newest **`messages/`
/// entry mtime** (a delivery or a result landing as a file) and the **latest
/// step's `response.json` mtime** (the live streaming tail, rewritten as tokens
/// arrive). Committing is only one of the three ways an agent acts, so the tip
/// alone leaves a streaming or just-messaged conversation reading stale.
///
/// The tail is read unconditionally rather than only while in flight: a closed
/// tail's mtime is when that stream *finished*, which is an action too — so the
/// in-flight case is the general path with a still-growing file, not a branch.
fn last_action_from_disk(workspace: &Path, agent_id: &str, tip_ts: i64, newest_msg: i64) -> i64 {
    tip_ts
        .max(newest_msg)
        .max(live_tail_mtime(workspace, agent_id))
}

/// What one readdir of `<workspace>/agents/<agent-id>/messages/` yields (§5.1
/// #12): how many messages have ever landed and the newest mtime among the
/// entries. Two facts from one walk — the same fold `stream_from_disk` makes
/// for the live response — because reading the directory twice would cost a
/// second syscall per agent per tick and could catch two different states of
/// it.
struct Messages {
    /// The `NNN` counter's high-water mark — the highest counter present, not
    /// a count of files (bl-fde5). The two were the same until compaction:
    /// lernie's compactor deletes entries *below* the surviving counter (§5.1
    /// #12), so a file count shrinks mid-flight while the messages-ever-landed
    /// fact this field states never goes down — which is what the §7.2 echo's
    /// passed-the-baseline predicate needs to stay a reading and not a race.
    count: usize,
    newest_unix: i64,
}

/// When the latest step's **model call began** (`Agent::call_start_unix`, §5.1
/// #28, bl-9dfb): the mtime of that step's `request.json`, the write lernie
/// makes once and immediately before it hands the request to the adapter — the
/// instant it itself calls the step's `started_at`. `None` when the agent has no
/// step tree, or its latest step has not written a request.
///
/// The **branch tip** was the other candidate and it does not hold: lernie takes
/// no pre-call commit from step 2 on (§2.10), so the tip is whatever the
/// *previous* step's tool window committed — and a branch resumed by `lernie
/// advance` after a stop calls the model against a tip hours old, which would
/// read as an hours-long call five seconds in.
fn call_start_from_disk(workspace: &Path, agent_id: &str) -> Option<i64> {
    let steps = workspace.join(STEPS_DIR).join(agent_id);
    mtime_unix(&latest_step_dir(&steps)?.join("request.json"))
}

/// The [`Messages`] fold over `<workspace>/agents/<agent-id>/messages/` (§5.1
/// #12): a zero high-water and a zero mtime when the directory is absent or
/// empty. The mtime is over every entry — a stray subdirectory's mtime is
/// still a write into the transcript directory — while the count reads only
/// names carrying the `NNN` counter, through the one parse of that shape
/// ([`crate::transcript::seq_of`]): a stray entry is a write, but it is not a
/// message that landed.
fn messages_from_disk(workspace: &Path, agent_id: &str) -> Messages {
    let dir = workspace.join(AGENTS_DIR).join(agent_id).join(MESSAGES_DIR);
    let mut fold = Messages {
        count: 0,
        newest_unix: 0,
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return fold;
    };
    for entry in entries.flatten() {
        fold.newest_unix = fold.newest_unix.max(mtime_unix(&entry.path()).unwrap_or(0));
        if let Some(seq) = crate::transcript::seq_of(&entry.file_name().to_string_lossy()) {
            fold.count = fold.count.max(seq);
        }
    }
    fold
}

/// Mtime of the latest step's `response.json` — the very file
/// [`stream_from_disk`] folds (§3.5) — or zero when no step has opened
/// one.
fn live_tail_mtime(workspace: &Path, agent_id: &str) -> i64 {
    let steps = workspace.join(STEPS_DIR).join(agent_id);
    latest_step_dir(&steps)
        .and_then(|dir| mtime_unix(&dir.join(RESPONSE_FILE)))
        .unwrap_or(0)
}

/// A path's modification time as whole unix seconds, or `None` when there is no
/// stamp to read (absent path, or one from before the epoch). The recency fold
/// spends a `None` as zero — an unknown action time is no action, and it is
/// taking a max — while the two elapsed starts (§5.1 #28) keep it as absence,
/// which is what makes the strip omit rather than invent. `pub(super)` so the
/// tool-record reader ([`super::tools`]) stamps `input.json` through this one
/// definition.
pub(super) fn mtime_unix(path: &Path) -> Option<i64> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|at| at.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|since| i64::try_from(since.as_secs()).ok())
}

/// What a yog-composed `goal.md` says (§3.3), all three facts of it: the
/// start-flow ball id, the **legacy** `You are <x>.` identity line (the name's
/// home before lernie 0.0.4; kept only for pre-0.0.4 roots until retention ages
/// them out), and the operator's payload headline — the §3.3 ladder's second
/// rung. The two stamps are read back with yog's own inverses
/// ([`crate::start::parse_ball_stamp`] / [`crate::start::parse_identity_stamp`]),
/// one parse per compose.
struct Goal {
    ball: Option<String>,
    name: Option<String>,
    preview: Option<String>,
}

/// All three facts from `<workspace>/agents/<id>/goal.md` (the agent worktree,
/// §7.1) in **one** read — a missing file (a removed worktree, a non-yog agent)
/// drops every one of them, and a goal without a given stamp yields `None` for
/// that stamp alone.
///
/// The preview's source is this file and **not** `steps/<id>/001/request.json`
/// (bl-368d): the request record is the assembled model context, which since
/// the §3.7 instruction freeze opens with a pinned-instruction frame and wraps
/// a deposit in its `---` envelope, so its head is never what the operator
/// said. `goal.md` is the payload's one home, so the fact has one home too.
fn goal_from_disk(workspace: &Path, agent_id: &str) -> Goal {
    let path = workspace.join(AGENTS_DIR).join(agent_id).join("goal.md");
    let text = std::fs::read_to_string(&path).ok();
    Goal {
        ball: text.as_deref().and_then(crate::start::parse_ball_stamp),
        name: text.as_deref().and_then(crate::start::parse_identity_stamp),
        preview: text.as_deref().map(payload_headline),
    }
}
