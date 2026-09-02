//! One agent branch as the view-model carries it (§7.1, §2.3) — the fattest of
//! [`super`]'s inert structures, on its own file for §12's pre-split rule.
//!
//! Every field is a fact read off refs or disk on the tick that built it (§3.5
//! stateless re-read); the two `impl` folds beside them derive nothing new,
//! they only state one ladder in one place.

use super::{StepCommit, ToolCall};
use crate::git_tree::{AgentState, Stream};

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
    /// the mtime of that step's `request.json`. litany writes that file exactly
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
    /// The latest turn was **cut off at the output limit** (§4.4, bl-fb87):
    /// at rest, with the latest step's settled tail framing cleanly around a
    /// `finish` whose canonical reason is `length`. Transport completion is not
    /// task completion — the segment kept every wire promise while the turn it
    /// framed ran out of `max_tokens` mid-utterance, so nothing more is coming
    /// and no continuation exists to run.
    ///
    /// Its one consumer is §8.2's Nudge gate
    /// ([`nudge_enabled`](crate::actions::nudge_enabled)): linked litany reads
    /// this shape as `NothingDue` and exits without creating a step, so
    /// offering the control would be a control that fires and does nothing
    /// (QUALITY H4's theater). The *recovery* is Message, which is offered
    /// unconditionally, and the §7.3 step wound says so in words. `false`
    /// under a held lock, where the question is not asked (§3.5).
    pub truncated: bool,
    /// **Why the latest model call failed**, in the provider's own words, or
    /// `None` when it did not (bl-b43b's `refused`, widened by bl-9b88). Read
    /// off the same bytes and in the same pass as
    /// [`truncated`](Agent::truncated), and at rest for its reason exactly — a
    /// driver holding the lease is itself the answer to "what now".
    ///
    /// Two shapes of one fact, and until bl-9b88 only the first was read: the
    /// in-band `error` event brazen speaks on stdout, and — when the adapter
    /// died before reaching that contract, which is what a credential-less
    /// provider row does — the step's own `stderr.log`. The second shape is the
    /// one the live sighting wore: every conversation in a workspace launched a
    /// driver, every model call refused, and every seat painted a conversation
    /// that simply never answers.
    ///
    /// It is the *why* riding beside the state rather than a state of its own:
    /// the badge set is frozen at four (§5.1 #9), so a failed call comes to rest
    /// [`Stopped`](AgentState::Stopped) like every other wound, and what
    /// separates it from an operator's own `/stop` is this sentence. The §11 row
    /// says its [`clause`](crate::git_tree::clause) and paints `Tone::Bad`; the
    /// provider **row** to sign in to is the steps surface's `auth_row` — one
    /// fact, one home, one query deeper.
    pub failure: Option<String>,
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
    /// `refs/litany/conflicted/<agent-id>` oid, or `None` when unmarked —
    /// a work-product transfer was declined (§2.6). Rendered as an
    /// orthogonal mark alongside the state (§3.5, §7.1); the oid is the §6
    /// attention watermark evidence (rule 4).
    pub conflicted_oid: Option<String>,
    /// `refs/litany/budget-exhausted/<agent-id>` oid, or `None` — the
    /// agent tree hit a spend ceiling (§6). Rendered alongside the state
    /// (§3.5, §7.1); the oid is the §6 watermark evidence (rule 3).
    pub budget_oid: Option<String>,
    /// `refs/litany/abandoned/<agent-id>` oid, or `None` — the policy
    /// assertion that a stopped branch will not be retried (ARCH §8). Its
    /// presence suppresses the stop-attention signal (§6 rule 2).
    pub abandoned_oid: Option<String>,
    /// `refs/litany/notify/<agent-id>` oid, or `None` — the branch asked
    /// the UI to raise a notification (ARCH §8). The oid is the §6
    /// watermark evidence (rule 1: unseen = oid ≠ watermark).
    pub notify_oid: Option<String>,
    /// The invocation the capability control **parked** before it executed
    /// (`refs/litany/held/<agent-id>`, ARCH §3.3, DESIGN §8.6): which
    /// `tool_use`, which tool, and the control's reason. `None` for every
    /// branch nothing is holding, which is nearly all of them.
    ///
    /// The **value**, not an oid, and that is §6 rule 6's shape: a park is not
    /// acknowledgeable — hiding a parked drone behind a watermark would hide a
    /// drone that cannot move — so there is nothing to seen-gate, and what the
    /// operator needs is what the blob says.
    pub held: Option<crate::control::hold::Held>,
    /// The newest **flag** raised on this agent (VISION §4.9, §6 rule 7,
    /// bl-6f2f): when, and why in the raiser's own words. `None` for every
    /// conversation nobody has flagged, which is nearly all of them.
    ///
    /// **The one field on this type the workspace does not answer.** Every
    /// other fact here is read off `<workspace>/…` by [`super::super::from_repo`];
    /// a flag lives in yog's own `ops.jsonl`, and this is stamped on by
    /// [`crate::monitor::flag::fold`] at the one place the ops trail and the
    /// derived trees are both final. It rides *here* rather than arriving as a
    /// seventh parameter because §6's predicate and its acknowledgement both
    /// take `&Agent` and nothing else — that is the shape of a signal, and a
    /// signal that did not fit it would have to be threaded through the rank
    /// sort, both rollups, the roster walk and every caller of each.
    pub flagged: Option<crate::monitor::Flag>,
    /// The start-flow ball id stamped in this agent's `goal.md` (DESIGN §3.3),
    /// parsed back by [`crate::start::parse_ball_stamp`] — the *derived*
    /// conversation↔ball association, never stored (§3.2, §5.1: a fact whose
    /// one home is the goal content). Only a root the yog start flow composed
    /// carries one; a sub-agent or a hand-typed conversation reads `None`.
    pub goal_ball: Option<String>,
    /// The **litany-stored name fact** (DESIGN §3.3 as ruled by bl-50f3): the
    /// `name` blob on this agent's own branch, committed beside `goal.md` at
    /// dispatch and read back `git show agents/<id>:name` — the `agents/*`
    /// refs stay the only registry, so this is a query, never a stored index.
    /// The one durable home of the name since lernie 0.0.4; any agent may
    /// carry it — a yog-fired root (`--name` at fire) or a litany-dispatched
    /// child alike, no special case. `None` for an unnamed agent and for a
    /// pre-0.0.4 branch with no blob.
    pub name: Option<String>,
    /// The **legacy** `You are <x>.` stamp on this agent's `goal.md` first line
    /// (DESIGN §3.3), parsed back by [`crate::start::parse_identity_stamp`].
    /// Demoted by bl-08f2 from the name's home to its fallback: it covers only
    /// pre-0.0.4 roots (no [`name`](Agent::name) blob) until litany's 30-day
    /// retention ages them out — then the rung is deleted. Not a fact home;
    /// since bl-6920 nothing composes the stamp, so new roots never carry one.
    pub goal_name: Option<String>,
}

impl Agent {
    /// The agent's display identity, or `None` when it has none — the top of
    /// the §3.3 ladder in one fold: the litany-stored [`name`](Agent::name)
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

    /// Was the latest model call **refused at the provider rung** (bl-b43b)?
    /// The auth-shaped reading of [`failure`](Agent::failure)'s sentence — a
    /// query, never a second stored flag, so the fact and the reading of it
    /// cannot disagree. §6's `AttentionKind::Refused` is the word it earns and
    /// the remedy (sign a provider in) is what makes it worth telling apart
    /// from every other failed call.
    pub fn refused(&self) -> bool {
        self.failure
            .as_deref()
            .is_some_and(crate::login::auth::looks_auth)
    }

    pub fn name_fact(&self) -> Option<String> {
        self.name.clone().or_else(|| self.goal_name.clone())
    }

    /// Whether [`name_fact`](Agent::name_fact)'s answer is the **legacy
    /// display-only rung** (bl-8068): a goal-stamp parse with no litany-stored
    /// [`name`](Agent::name) blob behind it. Such a name renders as the title,
    /// but litany resolves message targets by id or *stored* name only — a
    /// peer addressing this name gets `no agent "<x>" in this workspace`. The
    /// seats that show the name hover this fact so an operator never reads an
    /// unaddressable name as an addressable one. `false` for a fact-named
    /// agent and for one with no name claim at all.
    pub fn name_display_only(&self) -> bool {
        self.name.is_none() && self.goal_name.is_some()
    }
}
