//! The boundary's typed answers (§8.5). A [`Reply`] is what
//! [`dispatch`](super::dispatch::dispatch) and
//! [`answer`](super::answer::answer) return — the datum both frontends consume:
//! the GUI reads the variant in RAM, the headless transport writes
//! [`encode`] to the deposit's reply file, and a seat that did not derive the
//! answer reads it back with [`decode`] (REMOTE §9 step 2, bl-7067).
//!
//! It was **encode-only** until that step, on the argument that a reply is
//! yog's own statement rather than an instruction it parses back — the one
//! exception being the `prepare` reply's `prepared` body, which deliberately
//! re-enters as the next [`Prompt`](super::Action::Prompt) gesture and shares
//! its codec spelling. The client/server split (REMOTE §9) retires that
//! argument: a thin seat holds no world, so every answer it renders is one it
//! was told, and the exception has become the general path.
//!
//! The spelling itself is [`encode`], split off at §12's budget (bl-6233): the
//! answer and the way it is said are two subjects, and only one of them is what
//! the window reads.

use crate::actions::verbs::Outcome;
use crate::board::Board;
use crate::nav::convs::ConvRow;
use crate::opslog::OpRow;
use crate::projects::join::JoinRow;
use crate::search::Found;
use crate::start::Prepared;

use super::codec::prepared_value;

/// The §11 conversation seat's own spelling, both directions (REMOTE §9.4,
/// bl-1eb0) — cut off the roster at the budget like `search` and `queue`.
mod agent;
/// The §11 balls section's row — its own file at the budget (bl-b4b5), for
/// the reason its own doc gives.
mod balls;
/// The V4 board row's own encoders — split at the §12 budget, on the seam that
/// board rows are the one reply whose rows carry derived sub-objects (gates,
/// drones, two §3.5 figures).
mod board;
/// The whole surface's JSON spelling read back into the type (bl-7067) -- the
/// thin seat's half of the codec, cut on the same seam as the spelling itself.
mod decode;
/// The whole surface's JSON spelling, and the envelope helpers it shares.
mod encode;
/// The §6 decision queue's row encoder — the other reply whose rows carry a
/// derived list (its firing signals).
mod queue;
/// `pub(crate)` for the §5.1 agent-state token table, which the rail's own
/// encoder reads rather than keeping a second copy of (bl-6233).
pub(crate) mod rows;
/// The search reply's own address-flattening — split at the same budget.
mod search;

/// The one listing row the boundary itself owns — its own file at §12's budget
/// (bl-296f), for the reason its own doc gives.
mod ws_row;

pub use decode::decode;
pub use encode::{encode, refusal};
pub use ws_row::{Workspaces, WsRow};

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
    /// The nudge's driver was launched (§8.2, bl-9bef): `lernie advance` is
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
    /// answers with its `lernie config` run ([`Outcome`](Self::Outcome))
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
    /// A tool host's set landed (REMOTE §5, bl-4e08). **It carries nothing**,
    /// on §8.1's own test: the stored set after the write *is* the set the
    /// gesture carried, so a count or an echo here would be one computable fact
    /// said twice — the [`Applied`](Self::Applied) shape. Whether the write
    /// actually touched the file is an optimization, not an answer.
    Advertised,
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
    /// **The conversation** (§11, bl-6233) — [`Transcript`](super::Query::Transcript)'s
    /// answer: the committed `messages/` entries with the in-flight tail folded
    /// on, which is the whole of what the window's chat pane paints.
    Transcript(crate::transcript::Transcript),
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
    },
    /// The config commit a conversation is frozen on (§9.3, §5.1 #17) —
    /// [`Governing`](super::Query::Governing)'s answer, the §11 Config tab's
    /// whole content and VISION V1.2's *config-frozen-at* spelled. The one
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

#[cfg(test)]
mod tests;
