//! **What a gesture earns**, as a type (§8.5) — the boundary's one answer
//! enum, cut off `reply.rs` at §12's pre-split band on the seam `start/model`
//! already uses: the inert shape a caller reads, beside the modules that say
//! it. Nothing here derives, encodes or decodes anything; each variant is one
//! outcome and its doc is why that outcome is its own row.

use crate::actions::verbs::Outcome;
use crate::board::Board;
use crate::nav::convs::ConvRow;
use crate::opslog::OpRow;
use crate::projects::join::JoinRow;
use crate::search::Found;
use crate::start::Prepared;

use super::ws_row::Workspaces;

/// The typed answer a gesture earns. Exhaustive over the boundary's outcomes;
/// an error path is the `Err(String)` beside it, encoded by [`refusal`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// A short verb's captured run (§8.2) — `ok` iff exit 0.
    Outcome(Outcome),
    /// The `prepare` action's product: the composer's fire-time parameters.
    Prepared(Prepared),
    /// The `fan` action's product (§3.8): one `prepare` reply per candidate,
    /// rebound to its own attempt worktree and ready for the ordinary `prompt`.
    Fanned(Vec<Prepared>),
    /// The `retire` action's product: `discarded` says whether the retention
    /// policy also took the source ref — what the policy *did*, never what was
    /// asked, since an undeclared retention keeps it.
    Retired {
        discarded: bool,
    },
    /// The `deliver` action's product (VISION V3.2): the identities one
    /// candidate's delivery acted on — a receipt, never a stored winner; the
    /// standing fact is the tagged squash the target's history now carries.
    Delivered(crate::fan::Delivery),
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
    /// releasing `litany advance` was launched. It answers with the *held
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
    /// The nudge's driver was launched (§8.2, bl-9bef): `litany advance` is
    /// running detached against the conversation. It answers with nothing else
    /// because there *is* nothing else yet — what the model does with the turn
    /// arrives on the transcript and the §4.2 trail, at its own pace, and a
    /// receipt that guessed at it here would be a receipt that lied.
    Nudged,
    /// The §4.2 ack line landed — every current alarm is acknowledged.
    Acked,
    /// The trail was truncated; the clear is the fresh trail's first row.
    TrailCleared,
    /// The §9 config file the gesture named landed (bl-3f46). A lineage write
    /// answers with its `litany config` run ([`Outcome`](Self::Outcome))
    /// instead — a write and a spawn earn different receipts, here as
    /// everywhere else on the boundary.
    ///
    /// **It carries nothing** (REMOTE §8, bl-ccf7). It used to carry the
    /// absolute path that now held the text, which was the last of §8's
    /// path-typed residuals to have a computable answer: a
    /// [`ConfigFile`](super::config::ConfigFile) determines its own location
    /// exactly — the wall's `config.toml`, `models.yaml`,
    /// `workflows/<name>.yaml`, `cadence.yaml` — so the field was a second
    /// representation of the destination the gesture had just named, spelled
    /// as an operator's home root a client on another machine could neither
    /// use nor unsee. The receipt says *that* it landed, which is the whole of
    /// what a write can add to the address it was given — the
    /// [`Nudged`](Self::Nudged) / [`Acked`](Self::Acked) /
    /// [`TrailCleared`](Self::TrailCleared) shape.
    Applied,
    /// The agent's tracking branch (§16.3), **re-read after the write**: what
    /// actually landed, never an echo of what was asked.
    ///
    /// **The space it landed in is not answered** (REMOTE §8, bl-ccf7). It was,
    /// on the argument that "which branch" and "whose branch" are one question
    /// — but since the §16.3 per-agent ruling a workspace's marks space is
    /// always its own (`<wall>/marks`), so
    /// [`marks::read`](crate::world::marks::read) is a pure function of the
    /// workspace the gesture already named and the field could answer nothing
    /// else. One computable fact, spelled twice, and the second spelling was an
    /// absolute path.
    Marks {
        branch: String,
    },
    /// A tool host's set landed (REMOTE §5, bl-4e08), and **whether this
    /// engine WROTE it** (REMOTE §5.1, bl-66d4, PROTOCOL 8). It carried nothing
    /// until then, on §8.1's own test — the stored set after the write *is* the
    /// set the gesture carried, so an echo would be one computable fact said
    /// twice, the [`Applied`](Self::Applied) shape — and that test still rules
    /// out an echo. What it does not rule out is this: `wrote` is not the set,
    /// it is what happened to the DOCUMENT, and no other party can compute it.
    ///
    /// It is false on the ordinary re-presentation, which every reconnect and
    /// every §5.3 hand-off makes. A `true` on any later re-assertion is the
    /// advertising box learning that something blanked or replaced its set
    /// while it was absent — the one event bl-4e08's traffic ruling made
    /// undetectable from that end, reported here rather than in a trail that
    /// only this side reads.
    Advertised {
        wrote: bool,
    },
    /// **One device enrolled** (REMOTE §8.4, bl-f4e3): the QR envelope's whole
    /// payload, and the only moment the minted key exists off the device it is
    /// for — [`enroll::Enrolled`](crate::registry::enroll::Enrolled)'s fields.
    Enrolled(crate::registry::enroll::Enrolled),
    /// **One routed invocation, and its capture if it has one yet** (REMOTE §5,
    /// bl-024b) — the answer to all three gestures of the routing leg's asking
    /// side, because they are one subject at three moments: `invoke` has just
    /// queued it (`capture` absent), `complete` has just answered it, and
    /// `capture` is the poll in between. One variant rather than three, on
    /// [`Marks`](Self::Marks)' own terms: what is answered is the slot **as it
    /// stands after the call**, never an echo of what was asked.
    Routed {
        invocation: String,
        capture: Option<crate::registry::mailbox::Capture>,
    },
    /// What this tool host has been asked to run (REMOTE §3) — the
    /// follow-class read's answer. Empty is the ordinary answer of a hold that
    /// expired with no work, not a failure: the host asks again.
    Invocations(Vec<crate::registry::mailbox::Invocation>),
    /// The altitude-0 chrome (REMOTE §9.7, bl-b4b5): the enumerated workspaces
    /// with their §6 rollups and §4.1 pin ranks, and the §7.2 currency of the
    /// derivation they came off.
    Workspaces(Workspaces),
    Conversations(Vec<ConvRow>),
    Balls(Vec<JoinRow>),
    /// **One workspace's bound balls with their §3.5 figures** (§3.2, §11
    /// balls section) — [`WorkspaceBalls`](super::Query::WorkspaceBalls)'
    /// answer. The §11 strip, the roster rows, the ▶ Continue menu's object and
    /// the settings band's per-ball figures are all selections out of this one
    /// listing (`nav::balls`), which is what makes them one ask.
    WorkspaceBalls(Vec<crate::nav::BoundBall>),
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
    /// **The attempt science projection** (§3.9, VISION §4.10 item 7) —
    /// [`Science`](super::Query::Science)'s answer: one row per delivery
    /// attempt, each carrying the [`WorkDiff`](Self::WorkDiff) row it composes
    /// plus the agent-side join — frozen inputs, usage, wall time, verdicts and
    /// the derived outcome. Nothing in it is stored anywhere.
    Science(Vec<crate::science::Attempt>),
    /// **The conversation** (§11, bl-6233) — [`Transcript`](super::Query::Transcript)'s
    /// answer: the committed `messages/` entries with the in-flight tail folded
    /// on, which is the whole of what the window's chat pane paints.
    Transcript(crate::transcript::Transcript),
    /// **One frame of the live tail** (REMOTE §3, §5.5; DESIGN §7.2; bl-73e7,
    /// bl-3655) — [`Follow`](super::Query::Follow)'s answer, and an **append**
    /// since bl-3655: what landed since the read's previous frame, folded onto
    /// what a seat holds ([`Stream::absorb`](crate::git_tree::Stream::absorb)).
    /// REMOTE §5.5 states that rule; this does not restate it. Empty is the
    /// honest answer for a conversation with nothing in flight.
    Follow(crate::git_tree::Stream),
    /// Every step the conversation has taken (§11) —
    /// [`Steps`](super::Query::Steps)' answer, the Steps tab's list.
    Steps(crate::steps_view::StepsView),
    /// One step's records drilled in (§11) — [`Step`](super::Query::Step)'s
    /// answer, the parsed records and the capture logs that had bytes. A step
    /// the tree does not hold answers absent records rather than refusing:
    /// "nothing was written there" is a reading, not an error.
    Step(crate::steps_view::StepDetail),
    /// The agent worktree's listing and, when the query named a listed file,
    /// its bounded preview (§11) — [`Files`](super::Query::Files)' answer. The
    /// [`WorkDiff`](Self::WorkDiff) shape, because it is the same question:
    /// what is here, and what does this one say.
    Files {
        view: crate::files_view::FilesView,
        preview: Option<crate::files_view::Preview>,
        /// **Where this conversation actually works, when that is not the
        /// directory `view` listed** (bl-1015) — litany's own
        /// `refs/litany/cwd/<agent-id>` mark, which §3.3 calls *"the one
        /// channel"* for the work target: a path or ball rung seeds it at
        /// creation, so every tool step of every turn runs there and the
        /// agent worktree this listing walks holds none of the work.
        ///
        /// `None` is the case where the promise holds — a bare start, whose
        /// tools run in the listed worktree — so a reader never has to tell
        /// "bound elsewhere" from "bound here", exactly as
        /// [`FilesView::AbsentWorktree`](crate::files_view::FilesView) spares
        /// it telling a torn-down worktree from an empty one.
        ///
        /// It is a **statement, not a listing**: yog does not walk it. The
        /// listing is still the agent worktree, and this says outright that
        /// the work products are somewhere this read does not reach and where
        /// that is — which is QUALITY H2 (*"a fact yog cannot derive is
        /// answered as absent and never as a zero"*) applied to a listing.
        /// Without it a conversation that built its whole deliverable in the
        /// bound directory answered a normal listing of goal and soul, and
        /// nothing anywhere said the work had gone elsewhere.
        working_dir: Option<std::path::PathBuf>,
    },
    /// The config commit a conversation resolves its policy from (§9.3, §5.1
    /// #17) — [`Governing`](super::Query::Governing)'s answer, the §11 Config
    /// tab's whole content and VISION V1.2's *config-frozen-at* spelled, with
    /// the lineage it follows or the divergence holding it. The one
    /// member of the family that answers a **derivation over the workspace's
    /// git** rather than a listing, so it refuses where the others answer
    /// absent: a conversation always has a policy, and "none" would be a lie.
    Governing(crate::config_edit::branch::GoverningConfig),
    /// The step spine (VISION V1) — [`Rail`](super::Query::Rail)'s answer: the
    /// notches and the child cards hanging off them, unpinned. The pin is the
    /// viewport's, and §8.5 files a viewport's folds under views.
    Rail(crate::rail::Rail),
    /// One agent's undelivered deposits (§11, ARCH §2.11) —
    /// [`Inbox`](super::Query::Inbox)' answer.
    Inbox(Vec<crate::inboxview::InboxEntry>),
    /// One conversation as a seat sees it (REMOTE §9.4, bl-1eb0) —
    /// [`Agent`](super::Query::Agent)'s answer: the §11 centre pane's identity
    /// line, mark row, live badge and §8.2 gates, none of which had a spelling
    /// any face but the window could read.
    Agent(crate::boundary::answer::agent::AgentView),
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
    /// This workspace's role assignments (§9.4, §5.1 #27) —
    /// [`Read::Roles`](crate::boundary::config::Read::Roles)' answer: the whole
    /// `roles:` block of the commit its lineage stands at, one row per role, in
    /// file order. The type is the grammar's own
    /// [`RoleModel`](crate::model_pick::RoleModel), which the §9.4 gestures
    /// already write and the fork composer already reads — one vocabulary for
    /// one entry (bl-2410).
    Roles(Vec<crate::model_pick::RoleModel>),
    /// The workspace's config lineages with each tip's files (§9.3, bl-dff8) —
    /// [`Lineages`](super::Query::Lineages)' answer, the config pane's two
    /// dropdowns.
    Lineages(Vec<crate::config_edit::branch::Lineage>),
    /// The model ids one provider offers (§9.4, bl-dff8) —
    /// [`Models`](super::Query::Models)' answer, the picker's roster. Never
    /// empty: a provider that offered nothing is a refusal saying so, not a
    /// list a seat would read as "no models exist".
    Models(Vec<String>),
    /// The workspace's registered clients with their presence and advertised
    /// sets (REMOTE §5, bl-4e08) — [`Clients`](super::Query::Clients)' answer,
    /// and the payload the navigator's clients section paints.
    Clients(Vec<crate::registry::roster::ClientRow>),
}
