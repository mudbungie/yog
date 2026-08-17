//! The git-tree view-model types (§7.1 live view, §3.5 agent-state contract):
//! the error enum and the five inert structures [`GitTree::from_repo`] answers
//! with. Pure data — no git call, no egui dep, no derivation beyond the one
//! constructor that hands off to [`ProbeStack::derive`]; every field is a fact
//! read off refs or disk on the tick that built it (§3.5 stateless re-read).
//! [`super`] holds the wiring that fills them.

use super::{AgentState, ProbeStack, Stream};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum GitTreeError {
    #[error("git invocation failed: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("git {command} in {repo:?} failed: {stderr}")]
    Git {
        command: String,
        repo: PathBuf,
        stderr: String,
    },
    #[error("malformed git log line: {0:?}")]
    LogFormat(String),
    /// The governing-config derivation (§5.1 #17) declined: either no
    /// `config/*` lineage reaches the agent's branch, or two candidate
    /// ancestors are incomparable. Both mean a defective workspace, declined
    /// rather than guessed (mirrors lernie `workspace.rs`).
    #[error("governing config: {0}")]
    Governing(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitTree {
    /// The config lineage (`HEAD` → `config/default`, §2.2),
    /// first-parent, oldest to newest.
    pub commits: Vec<CommitNode>,
    /// Every agent branch (`agents/*`, §2.3), enumerated via
    /// `git for-each-ref refs/heads/agents/`. A flat authoritative set;
    /// the render tree is derived from the ids by [`descent_order`]
    /// (§2.3 hyphenated descent) — never stored (PRINCIPLES "Single
    /// source of truth").
    pub agents: Vec<Agent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    /// Commit subject — config commits are the only trunk commits
    /// (§2.2–§2.3: agents never merge anywhere), so the subject is the
    /// row's label.
    pub subject: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepCommit {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    /// Commit subject. Surfaces what a branch commit is — a dispatch, a
    /// delivery commit, or a work-product-transfer commit (§2.11, §2.6,
    /// §7.1 "delivery/result-message commits surfaced").
    pub subject: String,
}

/// One agent branch (`agents/<agent-id>`, §2.3). Named `Agent` — every
/// row is an agent, not an "unmerged conversation branch"; nothing merges
/// (§2.6), so the merged/unmerged framing is gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// Branch name (`agents/<agent-id>`). Held separately so the renderer
    /// can label rows without re-deriving.
    pub branch_name: String,
    /// The agent id (`agents/` prefix stripped) — the identity everywhere
    /// (steps/, inbox/, worktree dir, descent). The descent tree keys off
    /// this (§2.3).
    pub agent_id: String,
    pub tip_oid: String,
    pub tip_short_oid: String,
    pub tip_timestamp_unix: i64,
    /// The agent's **last action of any kind** (§11 recency, bl-cad5): the
    /// newest of the tip commit timestamp, the newest `messages/` entry mtime
    /// and the latest step's `response.json` mtime (the live streaming tail).
    /// Committing is only *one* way an agent acts, so
    /// [`tip_timestamp_unix`](Agent::tip_timestamp_unix) alone leaves a
    /// streaming or just-messaged conversation looking stale. Gathered here at
    /// snapshot time (§3.5 stateless re-read) so the §11 list's sort and its
    /// age label are one fact read once — never a stat from the render path.
    pub last_action_unix: i64,
    /// How many messages have **ever landed** in this agent's `messages/`
    /// directory (§5.1 #12): the monotonic `NNN` counter's high-water mark,
    /// read off the very readdir
    /// [`last_action_unix`](Agent::last_action_unix) already performs, so the
    /// two facts cost one directory walk (the §5.1 #10 discipline). Not a
    /// count of files present (bl-fde5): compaction deletes entries below the
    /// surviving counter, so a file count goes *down* mid-flight while this
    /// fact never does.
    ///
    /// Its one consumer is §7.2's pending echo: the operator's just-sent
    /// message is superseded when this count passes the baseline the echo
    /// recorded — a landed message advances the counter, a compaction never
    /// lowers it, so the predicate has one reading. Nothing cheaper says it
    /// honestly — [`last_action_unix`](Agent::last_action_unix) moves on a
    /// streaming token and [`tip_oid`](Agent::tip_oid) on any step commit, so
    /// either would retire the echo while the message was still missing.
    pub messages: usize,
    /// When the latest step's **model call began** (§5.1 #28 elapsed, bl-9dfb):
    /// the mtime of that step's `request.json`. lernie writes that file exactly
    /// once, immediately before handing the request to the adapter — the same
    /// instant it later records as `meta.json`'s `started_at` — and never
    /// appends to it, so its mtime *is* the call's start rather than its last
    /// sign of life. `meta.json` itself cannot serve: it lands only after the
    /// call returns, so it is absent for exactly the call the strip is timing.
    /// `None` when no step has written a request yet, or its stamp is
    /// unreadable — the strip then omits elapsed rather than inventing one.
    /// Gathered here at snapshot time (§3.5) like every other disk fact; the
    /// render path never stats.
    pub call_start_unix: Option<i64>,
    /// Commits on this branch past every config lineage, oldest to newest
    /// (each with its subject).
    pub steps: Vec<StepCommit>,
    pub preview: Option<String>,
    /// What the latest step's live `response.json` says (§5.1 #10, #28b): the
    /// answer text so far, the reasoning text so far, and the kind of the last
    /// content delta. **One value off one read** — three fields filled from
    /// one pass could otherwise be filled from three, and then they would be
    /// three mid-write states of one file rather than its answer.
    ///
    /// Re-derived from `<workspace>/steps/<agent-id>/<NNN>/response.json` on
    /// every `from_repo` call (§3.5: stateless re-read on each tick) — and, on
    /// the **focused** conversation, superseded per frame by the §7.2 live-tail
    /// follower's fresher fold of the same file (`app::live`), which is the one
    /// place a rendered `Agent` carries something no derivation put there.
    pub stream: Stream,
    /// Tool calls under this branch's latest step's `tools/` directory
    /// (ARCH §3.3), derived purely from `input.json` / `output.json`
    /// presence. Re-derived on every `from_repo` call (§3.5).
    pub tool_calls: Vec<ToolCall>,
    /// §3.5 agent-state classification, derived from the executor lock and
    /// the latest step's `response.json` terminal segment. Re-derived on
    /// every `from_repo` call (§3.5).
    pub state: AgentState,
    /// A liveness probe could not observe (DESIGN §10): the lock probe
    /// returned `Unknown`, or the writer probe did under a held lock. The
    /// [`state`](Agent::state) is then the best framing-only reading, never a
    /// false definite, and the renderer flags it with an uncertainty ("?")
    /// suffix. Always `false` on Linux, where `/proc` is authoritative.
    pub state_uncertain: bool,
    /// Pending (undelivered) deposits in the agent's inbox
    /// (`<workspace>/inbox/<agent-id>/*.md`, §2.11), oldest-first — the §5.1
    /// #11 derivation gathered at snapshot time (§3.5 stateless re-read) so
    /// its three seats (the `✉n` badge, the Inbox tab, the inbox-composer's
    /// pending queue, bl-929d) read one listing and the render path never
    /// touches disk. The count is this listing's length, never a second
    /// stored fact.
    pub pending: Vec<crate::inboxview::InboxEntry>,
    /// `refs/lernie/conflicted/<agent-id>` oid, or `None` when unmarked —
    /// a work-product transfer was declined (§2.6). Rendered as an
    /// orthogonal mark alongside the state (§3.5, §7.1); the oid is the §6
    /// attention watermark evidence (rule 4).
    pub conflicted_oid: Option<String>,
    /// `refs/lernie/budget-exhausted/<agent-id>` oid, or `None` — the
    /// agent tree hit a spend ceiling (§6). Rendered alongside the state
    /// (§3.5, §7.1); the oid is the §6 watermark evidence (rule 3).
    pub budget_oid: Option<String>,
    /// `refs/lernie/abandoned/<agent-id>` oid, or `None` — the policy
    /// assertion that a stopped branch will not be retried (ARCH §8). Its
    /// presence suppresses the stop-attention signal (§6 rule 2).
    pub abandoned_oid: Option<String>,
    /// `refs/lernie/notify/<agent-id>` oid, or `None` — the branch asked
    /// the UI to raise a notification (ARCH §8). The oid is the §6
    /// watermark evidence (rule 1: unseen = oid ≠ watermark).
    pub notify_oid: Option<String>,
    /// The invocation the capability control **parked** before it executed
    /// (`refs/lernie/held/<agent-id>`, ARCH §3.3, DESIGN §8.6): which
    /// `tool_use`, which tool, and the control's reason. `None` for every
    /// branch nothing is holding, which is nearly all of them.
    ///
    /// The **value**, not an oid, and that is §6 rule 6's shape: a park is not
    /// acknowledgeable — hiding a parked drone behind a watermark would hide a
    /// drone that cannot move — so there is nothing to seen-gate, and what the
    /// operator needs is what the blob says.
    pub held: Option<crate::control::hold::Held>,
    /// The start-flow ball id stamped in this agent's `goal.md` (DESIGN §3.3),
    /// parsed back by [`crate::start::parse_ball_stamp`] — the *derived*
    /// conversation↔ball association, never stored (§3.2, §5.1: a fact whose
    /// one home is the goal content). Only a root the yog start flow composed
    /// carries one; a sub-agent or a hand-typed conversation reads `None`.
    pub goal_ball: Option<String>,
    /// The **lernie-stored name fact** (DESIGN §3.3 as ruled by bl-50f3): the
    /// `name` blob on this agent's own branch, committed beside `goal.md` at
    /// dispatch and read back `git show agents/<id>:name` — the `agents/*`
    /// refs stay the only registry, so this is a query, never a stored index.
    /// The one durable home of the name since lernie 0.0.4; any agent may
    /// carry it — a yog-fired root (`--name` at fire) or a lernie-dispatched
    /// child alike, no special case. `None` for an unnamed agent and for a
    /// pre-0.0.4 branch with no blob.
    pub name: Option<String>,
    /// The **legacy** `You are <x>.` stamp on this agent's `goal.md` first line
    /// (DESIGN §3.3), parsed back by [`crate::start::parse_identity_stamp`].
    /// Demoted by bl-08f2 from the name's home to its fallback: it covers only
    /// pre-0.0.4 roots (no [`name`](Agent::name) blob) until lernie's 30-day
    /// retention ages them out — then the rung is deleted. Not a fact home;
    /// since bl-6920 nothing composes the stamp, so new roots never carry one.
    pub goal_name: Option<String>,
}

impl Agent {
    /// The agent's display identity, or `None` when it has none — the top of
    /// the §3.3 ladder in one fold: the lernie-stored [`name`](Agent::name)
    /// fact, else the legacy [`goal_name`](Agent::goal_name) stamp parse.
    /// Every seat that names an agent (the §11 row title and in-flight strip,
    /// the center header, the §3.6 deletion gate, the mint's occupied set)
    /// reads this one fold, so retiring the legacy rung is one deletion here.
    /// Whether this row exists **only in memory** — §7.2's pending
    /// conversation, the row a fired start paints before its driver has written
    /// a branch. A derived agent comes off `git for-each-ref`, so it always has
    /// a tip; an empty one cannot be derived, which makes this a query rather
    /// than a flag (§5.1's discipline: no fact stored twice). Its one consumer
    /// is the §11 tone — faded while a send is only yog's word for it,
    /// brightening when the derivation makes it a statement.
    pub fn in_memory(&self) -> bool {
        self.tip_oid.is_empty()
    }

    pub fn name_fact(&self) -> Option<String> {
        self.name.clone().or_else(|| self.goal_name.clone())
    }

    /// Whether [`name_fact`](Agent::name_fact)'s answer is the **legacy
    /// display-only rung** (bl-8068): a goal-stamp parse with no lernie-stored
    /// [`name`](Agent::name) blob behind it. Such a name renders as the title,
    /// but lernie resolves message targets by id or *stored* name only — a
    /// peer addressing this name gets `no agent "<x>" in this workspace`. The
    /// seats that show the name hover this fact so an operator never reads an
    /// unaddressable name as an addressable one. `false` for a fact-named
    /// agent and for one with no name claim at all.
    pub fn name_display_only(&self) -> bool {
        self.name.is_none() && self.goal_name.is_some()
    }
}

/// A single tool call surfaced to the renderer. The disk records carry
/// more metadata (timing, exit code, raw stdout) but the view-model only
/// needs identity + state to drive the indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// `tool_use.id` from the wire (e.g. `toolu_01abc…`); also the
    /// `<tool-id>/` directory name under
    /// `<workspace>/steps/<agent-id>/<NNN>/tools/`.
    pub tool_id: String,
    /// What the tool is **called** (`Read`, `Bash`, …) — `input.json`'s `name`
    /// field, read at enumerate time beside the two presence checks that
    /// decide [`state`](ToolCall::state), never from a render path (§11
    /// bl-cad5's rule). `None` when the record carries no parsable name; the
    /// §11 in-flight strip then drops the segment rather than printing the
    /// opaque `tool_id`, which names nothing to an operator.
    pub name: Option<String>,
    /// When this call **started** (§5.1 #28 elapsed, bl-9dfb): the mtime of the
    /// `input.json` whose presence already decides
    /// [`state`](ToolCall::state). The executor lands that record atomically
    /// immediately before it spawns the tool — the same instant it later records
    /// as `output.json`'s `started_at` — and never rewrites it, so its mtime is
    /// the call's start. No commit timestamp could serve instead: step records
    /// are not git-tracked (§2.3), so this file has no commit at all. `None`
    /// only when the stamp is unreadable (the record vanished between the
    /// presence check and the stat); the strip then omits elapsed.
    pub start_unix: Option<i64>,
    pub state: ToolCallState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallState {
    /// `input.json` has landed but `output.json` has not — the tool
    /// executor is still running. Renderer pulses this node.
    InFlight,
    /// Both `input.json` and `output.json` are present on disk. Renders
    /// statically; no repaint scheduling.
    Complete,
}

impl GitTree {
    /// Derive a workspace's tree with a fresh, throwaway [`ProbeStack`] — the
    /// one-shot path (tests, a single read). The live UI instead holds one
    /// [`ProbeStack`] across ticks (§15 Y11) so its TTL cache pays off; both
    /// route through the same [`ProbeStack::derive`] derivation.
    pub fn from_repo(workspace: &Path) -> Result<Self, GitTreeError> {
        ProbeStack::platform().derive(workspace)
    }
}
