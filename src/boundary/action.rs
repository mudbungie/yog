//! The **action roster** (§8.5): the mutating half of the boundary, its own
//! file at §12's cap — the same seam [`super::query`] is cut on, one enum over.
//! Actions mutate the world and are the §4.2 trail's rows; queries populate.
//! Two rosters, and only one of them can ever be wrong about the world.
//!
//! **An enum cannot be split across files, so what is split is the PROSE**
//! (bl-d255). Each variant keeps the sentence saying what it IS and the §8.2
//! argv it resolves to; the rulings behind it — why the gesture is shaped that
//! way, what it is deliberately not, which ball settled it — live in the
//! doc-only sibling its §8.5 family names ([`conversation`], [`world`],
//! [`capability`], [`folds`], [`device`]), on the precedent
//! [`exact`](crate::wire::hello::version::exact) set one module tree over. No
//! ruling is deleted and none is shaved: a roster that is a roster absorbs its
//! next gesture, and the family it belongs to absorbs its reasons.

use crate::start::{Payload, Prepared};

use super::config;

/// Why the §8.6 capability family's two acts are shaped as they are: prose only.
pub(crate) mod capability;
/// Why the §8.2 per-conversation gestures are shaped as they are: prose only.
pub(crate) mod conversation;
/// Why a REMOTE device's acts are shaped as they are: prose only.
pub(crate) mod device;
/// Why six families ride ONE variant over their own `Verb`: prose only.
pub(crate) mod folds;
/// Why the world's make/unmake/rank acts are shaped as they are: prose only.
pub(crate) mod world;

/// One mutating operator gesture (§8.5): every variant carries its whole
/// parameter set, so the two frontends construct byte-identical intents. The
/// §8.2 verb table is the argv each resolves to; the start family carries the
/// §8.1 composite's two real gestures (prepare, then the deferred prompt).
///
/// **A gesture addresses by NAME, never by path** (REMOTE §8, bl-f5f6): a
/// `workspace` is its §3.1 directory leaf, a `project` its derived
/// [`naming`](crate::naming) name. Across machines a path is meaningless and a
/// disclosure besides, so the world is reached by resolving the name **once**,
/// at [`dispatch`](super::dispatch::dispatch), ahead of the table — the tables
/// that say which name a variant carries are [`Action::workspace`] and
/// [`Action::project`] (`src/boundary/address.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `litany message <ws> <agent> <content>` — the resume gesture (§8.2).
    Message {
        workspace: String,
        agent: String,
        content: String,
    },
    /// `litany stop <ws> <agent>` (§8.2) — the SIGTERM cascade over the whole
    /// subtree ([`conversation`]).
    Stop {
        workspace: String,
        agent: String,
        children: bool,
    },
    /// **Send and interrupt** (§8.2, bl-a33d): stop this conversation, then
    /// deposit `content` — one gesture, two §4.2 rows ([`conversation`]).
    Interrupt {
        workspace: String,
        agent: String,
        content: String,
    },
    /// `litany scan <ws>` — flush inboxes, deposit died epitaphs (§8.2).
    Scan { workspace: String },
    /// **Fire inference from the state the conversation is already in** (§8.2's
    /// Nudge row, bl-9bef): `litany advance`, detached ([`conversation`]).
    Nudge { workspace: String, agent: String },
    /// `litany retarget <ws> <agent>` — the §9.4 **change of lineage** (bl-2d19,
    /// re-scoped by bl-e654) ([`conversation`]).
    Retarget { workspace: String, agent: String },
    /// **The `bl` family** (§8.2, bl-92d3) over
    /// [`verbs::Verb`](crate::actions::verbs::Verb) ([`folds`]).
    Ball(crate::actions::verbs::Verb),
    /// The §8.1 start flow's mutating half, returning the composer's
    /// [`Prepared`] — the ▶ Start / Create-&-Start / raise gesture ([`world`]).
    Prepare { workspace: String, payload: Payload },
    /// Fire the detached `litany prompt` (§8.1), the goal verbatim (bl-6920,
    /// [`world`]).
    Prompt {
        prepared: Prepared,
        goal: String,
        seed: Option<u64>,
    },
    /// The §3.8 mutating fan's family (VISION §4.10, bl-8746) over
    /// [`fan::Verb`](crate::fan::Verb) ([`folds`]).
    Fan(crate::fan::Verb),
    /// The §3.6 unmaking, gated exactly as the dialog gates it ([`world`]).
    DeleteWorkspace { workspace: String, typed: String },
    /// `litany delete <ws> <agent> [--children]` — the §3.6 class one
    /// conversation deep (bl-f17a) ([`world`]).
    DeleteAgent {
        workspace: String,
        agent: String,
        typed: String,
    },
    /// The alignment monitor's family (VISION §4.9, rung V6) over
    /// [`monitor::Verb`](crate::monitor::Verb) ([`folds`]).
    Monitor(crate::monitor::Verb),
    /// The armed loop's family (VISION §4.3, rung V4 item 2) over
    /// [`fleet::Verb`](crate::fleet::Verb) ([`folds`]).
    Fleet(crate::fleet::Verb),
    /// **Answer the invocation parked at one conversation's capability
    /// boundary** (VISION §4.11 items 5–6, §8.6) ([`capability`]).
    AnswerHold {
        workspace: String,
        agent: String,
        /// The verdict **and the scope it stands over** (bl-94a5,
        /// [`crate::control::judge::Answer`]).
        answer: crate::control::judge::Answer,
    },
    /// **Raise or lower one conversation's capability floor** (VISION §4.9's
    /// fifth rung, §4.11 item 7, §8.6) ([`capability`]).
    Floor {
        workspace: String,
        /// The conversation written for, **and its whole descent**.
        agent: String,
        /// `true` revokes tool auto-approval; `false` restores it.
        raised: bool,
    },
    /// Acknowledge every alarm on the trail (§4.2/§7.3, bl-c417): append the
    /// ack line every failure-derived alarm reads past.
    Ack,
    /// **Answer one item of the §6 decision queue** (VISION §5 V5.2, bl-f6fe):
    /// this conversation's present evidence, recorded as seen
    /// ([`conversation`]).
    MarkSeen { workspace: String, agent: String },
    /// **Pin a workspace, or unpin it** (§4.1 `pinned`, bl-b986) — an explicit
    /// **set**, never a toggle ([`world`]).
    Pin { workspace: String, pinned: bool },
    /// Start a fresh trail (§4.2 as amended): truncate `ops.jsonl`, logging
    /// the clear as the new trail's first row.
    ClearTrail,
    /// **The §9 config write family** (bl-dd88) over
    /// [`config::Write`](super::config::Write) ([`folds`]).
    Config(config::Write),
    /// One **attempt** (VISION §5 V2, bl-dc0c): `litany dispatch <role> <ws>
    /// <parent> --goal <goal> --from <ref> [--pin …]` ([`world`]).
    Fork {
        workspace: String,
        /// The dispatching parent's agent id (== its branch name).
        parent: String,
        /// Fire-time overrides: fork point, role (the model), skills to pin.
        attempt: crate::fork::Attempt,
        /// The goal, verbatim (§3.3, bl-6920).
        goal: String,
    },
    /// **A tool host presents its set** (REMOTE §5, bl-4e08): name, description
    /// and JSON Schema verbatim per element ([`device`]).
    Advertise {
        tools: Vec<crate::registry::tools::Tool>,
    },
    /// **The routing leg's two acts** (REMOTE §5, §9 step 7; bl-024b) over
    /// [`mailbox::Verb`](crate::registry::mailbox::Verb) ([`folds`]).
    Route(crate::registry::mailbox::Verb),
    /// **Enroll a device** (REMOTE §1.4 as amended, §8.4; bl-f4e3) over
    /// [`enroll::Request`](crate::registry::enroll::Request) ([`device`]).
    Enroll(crate::registry::enroll::Request),
    /// **The sign-in, as an act** (REMOTE §8.3; DESIGN §8.3 as amended by
    /// bl-61bf): `bz --login` on the ENGINE, inside the named workspace's wall
    /// ([`device`]).
    Login { workspace: String, provider: String },
}
