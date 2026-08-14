//! The control boundary (VISION §4.8, DESIGN §8.5): the one typed surface
//! every operator gesture crosses.
//!
//! Three families, decided per gesture by the §4.8 ruling. **Actions** mutate
//! the world and are the `ops.jsonl` trail's rows (§4.2); **queries** populate
//! — most are a §2 I1 derivation over the published [`Snapshot`](crate::app::Snapshot),
//! and the §9 config family's reads (bl-0164) are the same on-demand read
//! their write already is, over [`dispatch::Deps`]'s world — returning the
//! same typed data both frontends render; **views** (focus, scroll, tab
//! selection, drafts — §5.3's closed RAM whitelist, and the §4.1 presentation
//! durables beside it) never cross the boundary and gain no representation
//! here.
//!
//! The carrier is a datum, not a convention: the GUI's click-glue constructs
//! [`Action`]/[`Query`] variants and [`dispatch`](dispatch::dispatch) /
//! [`answer`](answer::answer) are the chokepoints both frontends share. The
//! headless serialization is the [`codec`] JSON envelope, deposited as a
//! create-only file into the yog-watched `gestures/` inbox ([`deposit`]),
//! consumed off-frame ([`consume`], [`consumer`]) and answered as a [`reply`]
//! file; `yog gesture` ([`sugar`]) is deposit-and-wait sugar over exactly that.
//! One surface, two serializations, never two implementations (VISION §8).
//!
//! **A new gesture without a headless spelling fails to compile**: adding a
//! variant here leaves [`codec::encode`], [`codec::decode`] and the dispatch
//! match non-exhaustive until the spelling exists.

use crate::start::{Payload, Prepared};

/// The windowless face's leading word (§8.5): `yog headless`. Named here, once,
/// because two spellings — the arm that dispatches it and the help that
/// advertises it — would be two facts.
pub const HEADLESS_SUBCMD: &str = "headless";
use std::path::PathBuf;

pub mod answer;
/// The §3.5 spend ceiling's one seat — the spawn gate.
pub mod ceiling;
pub mod codec;
pub mod config;
pub mod consume;
pub mod consumer;
/// The VISION §4.11 capability family's one executor — the hold answer's row
/// and its releasing `advance` — plus the confinement-required birth gate.
pub mod control;
pub mod deposit;
pub mod dispatch;
/// The VISION §4.3 armed loop's one executor — arming, which is a config write.
pub mod fleet;
pub mod help;
pub mod line;
/// The VISION §4.9 monitor's two executors — arming and flagging.
pub mod monitor;
pub mod reply;
pub mod sugar;
/// `pub(crate)` so the board's own corpus shares this one `Agent`/`Snapshot`
/// fixture rather than standing up a second of the same shape.
#[cfg(test)]
pub(crate) mod tests;

/// One mutating operator gesture (§8.5): every variant carries its whole
/// parameter set, so the two frontends construct byte-identical intents. The
/// §8.2 verb table is the argv each resolves to; the start family carries the
/// §8.1 composite's two real gestures (prepare, then the deferred prompt).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `lernie message <ws> <agent> <content>` — the resume gesture (§8.2).
    Message {
        workspace: PathBuf,
        agent: String,
        content: String,
    },
    /// `lernie stop <ws> <agent> [--stop-children]` (§8.2).
    Stop {
        workspace: PathBuf,
        agent: String,
        children: bool,
    },
    /// `lernie scan <ws>` — flush inboxes, deposit died epitaphs (§8.2).
    Scan { workspace: PathBuf },
    /// `lernie retarget <ws> <agent>` — the §9.4 exit from the config freeze
    /// (bl-2d19): mark this conversation to be re-forked onto the config
    /// lineage's head, which its own executor lands at the next step boundary.
    /// **No config name on the wire**: yog's picker writes one lineage and the
    /// drift it offers this against is measured against that lineage's tip, so
    /// naming a branch here would be a knob with one lawful value (§9.3).
    Retarget { workspace: PathBuf, agent: String },
    /// `bl close <id> --as <name>` (§8.2) — `name` is the ball's bound
    /// workspace name (§3.2), never the operator `$USER`.
    Close {
        project: PathBuf,
        id: String,
        name: String,
    },
    /// `bl claim <id> --as <name>` — assign a ready ball (§8.2/§3.2).
    Assign {
        project: PathBuf,
        id: String,
        name: String,
    },
    /// `bl unclaim <id> --as <name>` — release (§8.2/§3.2).
    Release {
        project: PathBuf,
        id: String,
        name: String,
    },
    /// Re-home a bound ball: `bl unclaim --as <from>` then `claim --as <to>` (§8.2).
    Move {
        project: PathBuf,
        id: String,
        from: String,
        to: String,
    },
    /// `bl create <title> --as <name> [--body B]` (§8.2).
    Create {
        project: PathBuf,
        title: String,
        name: String,
        body: Option<String>,
    },
    /// `bl update <id> --as <name> [--title T][--body B][-m N]` (§8.2).
    Update {
        project: PathBuf,
        id: String,
        name: String,
        title: Option<String>,
        body: Option<String>,
        note: Option<String>,
    },
    /// The §8.1 start flow's mutating half: seed → ensure-workspace → the ball
    /// rung's `bl` steps, returning the composer's [`Prepared`] — the ▶ Start /
    /// Create-&-Start / raise gesture. The prompt is the separate, deferred
    /// [`Action::Prompt`], exactly as the GUI defers it to the composer.
    Prepare {
        workspace: PathBuf,
        payload: Payload,
    },
    /// Fire the detached `lernie prompt` (§8.1): mint the conversation name,
    /// pass it via `--name`, spawn detached — the goal verbatim (bl-6920).
    /// `prepared` is the
    /// [`Action::Prepare`] reply (or a re-composed equal); `goal` the edited text.
    Prompt { prepared: Prepared, goal: String },
    /// The §3.6 unmaking, gated exactly as the dialog gates it: refused unless
    /// the workspace is yog's own, nothing is live, and `typed` re-states its
    /// name — fail-closed at fire time, whichever frontend fires.
    DeleteWorkspace { workspace: PathBuf, typed: String },
    /// `lernie delete <ws> <agent> [--children]` — the §3.6 class one
    /// conversation deep (bl-f17a). Gated on liveness here, fail-closed;
    /// `typed` re-stating the conversation's name is the one thing that arms
    /// `--children`, and an unarmed fire is the bare verb — lernie's own
    /// `HasDescendants` decline rides back for a subtree nobody confirmed.
    DeleteAgent {
        workspace: PathBuf,
        agent: String,
        typed: String,
    },
    /// The alignment monitor's family (VISION §4.9, rung V6): arm a workspace
    /// on a cheap model, disarm it, or raise an attention item on one
    /// conversation. One variant over [`monitor::Verb`](crate::monitor::Verb)
    /// rather than three here — same subject, same config file, same trail.
    /// Arming is the operator's explicit action and it *is* the mechanism:
    /// unarmed, no call is made, no row is written and nothing renders.
    Monitor(crate::monitor::Verb),
    /// The armed loop's family (VISION §4.3, rung V4 item 2): arm one
    /// workspace's fleet loop on a project and a cap, or disarm it. One variant
    /// over [`fleet::Verb`](crate::fleet::Verb) rather than two here, exactly as
    /// the monitor's family folds — same subject, same config file, same trail.
    ///
    /// **Arming is the explicit user action and it *is* the mechanism** (I7,
    /// §4.3): unarmed nothing spawns, nothing is reaped, nothing renders; an
    /// armed loop's spawns are that action, continuing. Severability is
    /// deleting the `cadence.yaml` entry, never editing a code path.
    Fleet(crate::fleet::Verb),
    /// **Answer the invocation parked at one conversation's capability
    /// boundary** (VISION §4.11 items 5–6, §8.6). The held `tool_use` id is
    /// *derived* — read off `refs/lernie/held/<agent>` at fire time — never
    /// typed, so the answer lands on exactly what is parked now and cannot
    /// race. **Nothing here ever calls stop** (lernie bl-b98d).
    AnswerHold {
        workspace: PathBuf,
        agent: String,
        /// `pass` releases it, `refuse` declines it in band, `hold` pins the
        /// park across a later policy edit — the control's own vocabulary,
        /// one word list ([`crate::control::judge::Ruling`]) in both
        /// directions.
        ruling: crate::control::judge::Ruling,
    },
    /// **Raise or lower one conversation's capability floor** (VISION §4.9's
    /// fifth rung, §4.11 item 7, §8.6): under a raised floor every effect class
    /// above `read` adjudicates to a hold, so a drone keeps reading, keeps its
    /// branch and keeps its history, and everything it reaches for waits on an
    /// operator instead of executing. Lowering is the symmetric restore — the
    /// fold is latest-row-wins, so the two directions are one gesture, never an
    /// order anyone has to get right.
    ///
    /// **A verdict is an input to this; it is never a substitute for it.** The
    /// monitor rules whether work serves the goal, the capability boundary
    /// rules what an agent may ever do. That is why this sits beside
    /// [`AnswerHold`](Self::AnswerHold) rather than inside
    /// [`Monitor`](Self::Monitor): §4.9's ladder spends existing verbs from
    /// the families that own them — notice is `message`, stop is `stop` — and
    /// this rung belongs to the capability family.
    Floor {
        workspace: PathBuf,
        /// The conversation the floor is written for. It stands over that
        /// conversation **and its whole descent** — the fold matches by
        /// hyphenated prefix ([`crate::control::judge::Answers::floored`]) —
        /// so flooring a parent floors a subtree without enumerating one,
        /// children not yet born included.
        agent: String,
        /// `true` revokes tool auto-approval; `false` restores it.
        raised: bool,
    },
    /// Acknowledge every alarm on the trail (§4.2/§7.3, bl-c417): append the
    /// ack line every failure-derived alarm reads past.
    Ack,
    /// **Answer one item of the §6 decision queue** (VISION §5 V5.2, bl-f6fe):
    /// record this conversation's present evidence as seen — the very
    /// watermarks the window writes by focusing it, from one evidence
    /// definition, so the two frontends converge over one disk (I0). The
    /// windowed seat keeps its focus-tick entry (focus is a view and gains no
    /// spelling); this is the entry a seat with no focus needs.
    MarkSeen { workspace: PathBuf, agent: String },
    /// Start a fresh trail (§4.2 as amended): truncate `ops.jsonl`, logging
    /// the clear as the new trail's first row.
    ClearTrail,
    /// One §9 config apply, carrying the **full staged text** (bl-3f46): the
    /// destination decides the pipeline it goes through, so the four config
    /// editors are one gesture rather than four ([`config::ConfigFile`]).
    ApplyConfig {
        file: config::ConfigFile,
        text: String,
    },
    /// **Amend an agent's own tracking branch** (§16.3, the
    /// per-agent ruling): point `workspace`'s balls space at `branch`. The
    /// launched-then-told-to-work-on-a-project case, and the same verb a launch
    /// spends — clause 2 and clause 4 are one gesture differing only in when it
    /// fires. It writes balls' own layer-2 config key in that space and stores
    /// nothing of yog's own shape; the reply is the branch **re-read** after
    /// the write.
    SetMarks { workspace: PathBuf, branch: String },
    /// One **attempt** (VISION §5 V2, bl-dc0c): `lernie dispatch <role> <ws>
    /// <parent> --goal <goal> --from <ref> [--pin …]` — the ordinary fork,
    /// with the pinned notch's commit (or a `config/<name>` head) as its ref.
    ///
    /// **A cohort is N of these, not a variant of its own.** V2's ×N fires
    /// this gesture N times with per-attempt overrides; membership is derived
    /// from the notch the children hang on and the ref each forked off
    /// ([`crate::rail::cohort`]), so nothing here — and nothing on disk —
    /// records a fan. That is why the boundary grows one attempt-shaped verb
    /// instead of a fan verb: N=1 and N>1 are the same gesture, counted.
    Fork {
        workspace: PathBuf,
        /// The dispatching parent's agent id (== its branch name).
        parent: String,
        /// The attempt's fire-time overrides: fork point, role (the model),
        /// skills to pin.
        attempt: crate::fork::Attempt,
        /// The goal, verbatim (§3.3, bl-6920).
        goal: String,
    },
    /// The §9.4 model pick: give `role` this `model` on this provider row, for
    /// `workspace`. §9.2 and §9.3 composed by one gesture — refuse either half
    /// and neither is written — because lernie's cross-check makes the role
    /// assignment and the model declaration two halves of one fact.
    PickModel {
        workspace: PathBuf,
        role: String,
        provider: String,
        model: String,
    },
}

/// Anything that crosses the boundary: an action or a query. Views do not —
/// they are §5.3's whitelist and have no spelling here, by design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gesture {
    Act(Action),
    Ask(Query),
}

/// The §8.2 after-verb ball-refresh target, one table over the action roster —
/// split out at §12's cap (bl-dc0c), because it is a *query on* the enum rather
/// than part of it.
mod project;

/// The populating-read roster, its own file at §12's cap (bl-765d). The seam is
/// the §8.5 taxonomy the help table is already cut along: actions mutate,
/// queries populate — two rosters, and only one of them can ever be wrong about
/// the world.
mod query;
pub use query::Query;
