//! The **action roster** (§8.5): the mutating half of the boundary, its own
//! file at §12's cap — the same seam [`super::query`] is cut on, one enum over.
//! Actions mutate the world and are the §4.2 trail's rows; queries populate.
//! Two rosters, and only one of them can ever be wrong about the world.

use crate::start::{Payload, Prepared};

use super::config;

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
    /// `litany stop <ws> <agent> [--stop-children]` (§8.2).
    Stop {
        workspace: String,
        agent: String,
        children: bool,
    },
    /// **Send and interrupt** (§8.2, bl-a33d): stop whatever is running this
    /// conversation, then deposit `content` — and the deposit's own
    /// driver-start is the trigger, litany's standing law (ARCH §2.9: there is
    /// no resume verb, a deposit into a quiescent branch starts a driver). One
    /// gesture, because the operator's act is one; **two ops rows**, because
    /// the interrupt and the deposit are independently observable mutations and
    /// a composite row would hide that a stop fired (§4.2).
    ///
    /// It gates on nothing the seat has to know: a stop landing on a branch with
    /// nothing in flight is declined in band and the deposit still lands — the
    /// same gesture at zero work, not a case of its own. What it *does* rest on
    /// is litany bl-b98d, which the pin carries; [`interrupt`](super::interrupt)
    /// records what that settling is and why nothing here may be built on a
    /// yog-side guess at litany's step state.
    Interrupt {
        workspace: String,
        agent: String,
        content: String,
    },
    /// `litany scan <ws>` — flush inboxes, deposit died epitaphs (§8.2).
    Scan { workspace: String },
    /// **Fire inference on a conversation from the state it is already in**
    /// (§8.2's Nudge row, bl-9bef): `litany advance <ws> <agent>`, detached.
    /// It carries no text, and that absence is the whole gesture — litany
    /// derives what is due from the transcript tail (ARCH §6 *warrant*), so a
    /// first turn whose model call died re-dispatches **in place**. Never
    /// [`Message`](Self::Message) with an empty body: a deposit would put a
    /// second user turn on the wire saying what the first already said.
    Nudge { workspace: String, agent: String },
    /// `litany retarget <ws> <agent>` — the §9.4 **change of lineage**
    /// (bl-2d19, re-scoped by bl-e654): mark this conversation to be re-forked
    /// onto the config lineage's head, which its own executor lands at the next
    /// step boundary. It is not how a config edit reaches a running
    /// conversation — that needs no gesture at all, since control resolves the
    /// followed lineage's tip at every step — it is how a conversation changes
    /// *which* lineage it follows, and the one way out of a divergence holding
    /// it on its fork commit. **No config name on the wire**: yog's picker
    /// writes one lineage, and that lineage is the one lawful destination, so
    /// naming a branch here would be a knob with one value (§9.3).
    Retarget { workspace: String, agent: String },
    /// **The `bl` family** (§8.2, bl-92d3): close, claim, unclaim, create,
    /// update — one variant over
    /// [`verbs::Verb`](crate::actions::verbs::Verb), the fold the monitor's,
    /// the fleet's, the routing leg's and the §3.8 fan's families each took,
    /// and on a seam every layer beneath already draws: `codec::balls`,
    /// `line::balls`, `answer::balls`, `reply::balls`. That type's own doc
    /// carries the five and what each spends.
    Ball(crate::actions::verbs::Verb),
    /// The §8.1 start flow's mutating half: seed → ensure-workspace → the ball
    /// rung's `bl` steps, returning the composer's [`Prepared`] — the ▶ Start /
    /// Create-&-Start / raise gesture. The prompt is the separate, deferred
    /// [`Action::Prompt`], exactly as the GUI defers it to the composer.
    Prepare { workspace: String, payload: Payload },
    /// Fire the detached `litany prompt` (§8.1): mint the conversation name,
    /// pass it via `--name`, spawn detached — the goal verbatim (bl-6920).
    /// `prepared` is the [`Action::Prepare`] reply (or a re-composed equal);
    /// `goal` the edited text. **`seed` is the firing seat's own §3.3
    /// prediction** (bl-1747): a seat that painted a greyed name fires the seed
    /// it painted, and `None` predicted nothing (a deposited line, the §4.3
    /// loop) so the door draws off the stamp. A `Deps` field until acts crossed.
    Prompt {
        prepared: Prepared,
        goal: String,
        seed: Option<u64>,
    },
    /// The §3.8 mutating fan's family (VISION §4.10, bl-8746): spread one
    /// delivery obligation into N isolated candidates, or retire one of them.
    /// One variant over [`fan::Verb`](crate::fan::Verb) rather than two here —
    /// the fold the monitor's, the fleet's and the routing leg's families take,
    /// for the same reason and on a seam every layer beneath already draws:
    /// `boundary::fan` holds both executors, `codec::fan` both spellings,
    /// `line::fan` both readers. That type's own doc says what each end costs.
    Fan(crate::fan::Verb),
    /// The §3.6 unmaking, gated exactly as the dialog gates it: refused unless
    /// the workspace is yog's own, nothing is live, and `typed` re-states its
    /// name — fail-closed at fire time, whichever frontend fires.
    DeleteWorkspace { workspace: String, typed: String },
    /// `litany delete <ws> <agent> [--children]` — the §3.6 class one
    /// conversation deep (bl-f17a). Gated on liveness here, fail-closed;
    /// `typed` re-stating the conversation's name is the one thing that arms
    /// `--children`, and an unarmed fire is the bare verb — litany's own
    /// `HasDescendants` decline rides back for a subtree nobody confirmed.
    DeleteAgent {
        workspace: String,
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
    /// *derived* — read off `refs/litany/held/<agent>` at fire time — never
    /// typed, so the answer lands on exactly what is parked now and cannot
    /// race. **Nothing here ever calls stop** — a decline is the model's own
    /// in-band tool result, which it reads and steps past, and that is a
    /// property of what an answer *means* rather than of what a stop costs: the
    /// cost changed when litany bl-b98d landed ([`Interrupt`](Self::Interrupt)),
    /// and this is unaffected by it.
    AnswerHold {
        workspace: String,
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
        workspace: String,
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
    MarkSeen { workspace: String, agent: String },
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
    SetMarks { workspace: String, branch: String },
    /// One **attempt** (VISION §5 V2, bl-dc0c): `litany dispatch <role> <ws>
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
        workspace: String,
        /// The dispatching parent's agent id (== its branch name).
        parent: String,
        /// The attempt's fire-time overrides: fork point, role (the model),
        /// skills to pin.
        attempt: crate::fork::Attempt,
        /// The goal, verbatim (§3.3, bl-6920).
        goal: String,
    },
    /// **A tool host presents its set** (REMOTE §5, bl-4e08): the three facts
    /// per element — name, description, JSON Schema verbatim — which the engine
    /// writes into that client's registration when they differ from what is
    /// stored ([`registry::tools`](crate::registry::tools)).
    ///
    /// **It names no client, and that is the gesture.** The identity it lands
    /// under is the *intake's* — a connection's certificate common name, read
    /// exactly where scoping reads it (REMOTE §4) — because a client field
    /// would let any connection overwrite any other client's set, which is the
    /// authorization the certificate already decided. An intake carrying no
    /// client identity (the `gestures/` inbox, `yog gesture`, the window)
    /// therefore refuses in band: it is a boundary verb like any other, and the
    /// wire gains nothing it does not (REMOTE §3).
    Advertise {
        tools: Vec<crate::registry::tools::Tool>,
    },
    /// **The routing leg's two acts** (REMOTE §5, §9 step 7; bl-024b): queue
    /// one tool call for the machine that advertised it, and post back what
    /// running it captured. One variant over
    /// [`mailbox::Verb`](crate::registry::mailbox::Verb) rather than two here,
    /// exactly as the monitor's and the fleet's families fold — one subject,
    /// one mailbox, one pair of ends, and that type's own doc says what each
    /// end costs.
    Route(crate::registry::mailbox::Verb),

    /// **Enroll a device** (REMOTE §1.4 as amended, §8.4; bl-f4e3) — one variant
    /// over [`enroll::Request`](crate::registry::enroll::Request), whose own doc
    /// carries the ruling and the §4.2 grade a foot is refused it by.
    Enroll(crate::registry::enroll::Request),
    /// The §9.4 model pick: give `role` this `model` on this provider row, for
    /// `workspace`. §9.2 and §9.3 composed by one gesture — refuse either half
    /// and neither is written — because litany's cross-check makes the role
    /// assignment and the model declaration two halves of one fact.
    PickModel {
        workspace: String,
        role: String,
        provider: String,
        model: String,
    },
}
