//! The **populating-read roster** (§8.5): every query both frontends ask, in
//! one enum. Its own file at §12's line budget (bl-765d), on the seam §8.5
//! already draws and the help table is already cut along — actions mutate,
//! queries populate. Nothing else moved with it: [`answer`](super::answer) is
//! still the one chokepoint, and [`Gesture`](super::Gesture) still names both
//! halves.

use super::config;

/// One populating read (§8.5): a §2 I1 derivation over the published snapshot,
/// answered by [`answer::answer`] — the same functions the frame's view-models
/// delegate to, so both frontends render one derivation.
///
/// **A read addresses by NAME, never by path** (REMOTE §8, bl-f5f6), exactly as
/// the mutating half does: a `workspace` is its §3.1 directory leaf, resolved
/// **once** at [`answer`](super::answer::answer) ahead of the table, over the
/// one table that says which reads name one ([`Query::workspace`],
/// `src/boundary/address.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// The enumerated workspaces (§3.1) with their §6 attention rollups.
    Workspaces,
    /// The §11 conversation list of one workspace: one row per root agent,
    /// subtree-aggregated, attention > running > recency.
    Conversations { workspace: String },
    /// The §3.5 join rows — every ball⇄workspace binding fact.
    Balls,
    /// **What one workspace holds** (§3.2/§3.5, §11 balls section; REMOTE §9.7,
    /// bl-b4b5): every ball bound to it, with its badge, the project its `bl`
    /// verbs run in, the claimant they stamp `--as`, and its priced figure.
    ///
    /// Distinct from [`Balls`](Query::Balls) by **address**, not by source, the
    /// way [`Board`](Query::Board) is distinct by altitude: that answers the
    /// world's whole binding table, this answers the one workspace a seat is
    /// looking at — which is what every §11 balls surface actually paints, and
    /// what a seat holding a §3.1 name could not select out of the table
    /// without the engine-side join bl-7407 refused.
    ///
    /// **The figure rides the row rather than earning a query.** A ball's spend
    /// is a filter over the same `Snapshot::bills` walk the listing is made
    /// from (§3.5, bl-9dd4), so asking for it separately would be a second read
    /// of one derivation — the two-readings-of-one-tail defect bl-296f closed
    /// at the activity chip.
    WorkspaceBalls { workspace: String },
    /// The V4 board (VISION §5 V4): the same balls, projected into the
    /// operator's four columns with their gates, drones, spend and epic
    /// rollups. Distinct from [`Balls`](Query::Balls) by *altitude*, not by
    /// source — that answers the binding facts, this answers the board built on
    /// them — and both are one derivation over the published snapshot.
    Board,
    /// **The decision queue** (VISION §5 V5.2): every conversation the §6
    /// predicate says is waiting on the operator, in the roster order the ↓ key
    /// walks, each carrying why it fires and what it last said. The attention
    /// strip's own derivation — at the window a count and a jump control, here
    /// a list an agent reads, answers ([`Action::MarkSeen`]) and hands on.
    Attention,
    /// The `ops.jsonl` tail (§4.2), newest last, at most `max` rows.
    Ops { max: usize },
    /// **What this workspace's agents actually changed** (§5.1 #32, VISION
    /// §4.10): the pure git read `target..source` of every attempt the
    /// workspace holds, and — when `file` names one — that file's bounded
    /// patch. The second query whose subject is the world's bytes rather than
    /// the snapshot's derivations: the snapshot says which balls this
    /// workspace claims, and the project repos are read for the rest.
    WorkDiff {
        workspace: String,
        file: Option<crate::workdiff::WorkFile>,
    },
    /// **Every delivery attempt this workspace holds, as science reads it**
    /// (§3.9, VISION §4.10 item 7; bl-40ab): per attempt, the frozen inputs
    /// (goal, pinned documents, governing config commit — model and skills ride
    /// it), the base/source/target OIDs and the delivered commit when one
    /// exists, the terminal response, usage and wall time, the project diff, the
    /// verdicts, and the accepted/rejected/reworked outcome.
    ///
    /// **It composes [`WorkDiff`](Query::WorkDiff), it does not replace it.**
    /// The diff row is one derivation with one home, and a science row carries
    /// that row rather than restating its fields — so the two answers agree by
    /// construction and the §11 fan-group seat can read the churn from either.
    /// What this adds is the agent side of the join: the conversation each
    /// attempt was bound to, and everything only that conversation can be
    /// asked. The third world-bytes read of the pair, and the reason it is a
    /// query rather than a table is VISION §4.5's join discipline — nothing is
    /// stored and nothing is cached, so a row is a statement about the world as
    /// it is now.
    Science { workspace: String },
    /// One text across the whole world (§8.5, [`search`](crate::search)): live
    /// and closed ball title+body, workspace and conversation identity+goal,
    /// and transcript text. The **asynchronous** query — its subject is the
    /// world's bytes rather than the snapshot's derivations, so a window asks
    /// it off-frame and renders whatever answer has landed. It carries no
    /// bound: how much of the world one answer hands back is yog's decision
    /// ([`search::MAX`](crate::search::MAX)), not a knob two seats could spell
    /// differently.
    Search { text: String },
    /// What a command does (§8.5): the whole roster, or one verb's page. The
    /// **higher-order** query — its subject is the interface, not the world —
    /// and therefore the one query with no snapshot to read, which is why every
    /// seat may answer it in place instead of depositing it ([`help`]).
    Help { verb: Option<String> },
    /// One §9 destination's current bytes (§8.5, bl-0164): the file editors'
    /// Reload, spelled — the same [`ConfigFile`](config::ConfigFile) the write
    /// already carries, so a read and its write name the place the same way.
    /// A **file** destination that is not there yet answers empty text, the
    /// same "new file" reading every editor's own load already gives; a
    /// **lineage** answers `git show config/<lineage>:<path>` — the §9.3 pane's
    /// own Load — and a path the lineage does not hold refuses in git's words,
    /// because a config commit's absence and a missing blob are one answer from
    /// git and yog will not invent an empty file over a real one (bl-dff8).
    ReadConfig { file: config::ConfigFile },
    /// **What lineages this workspace has, and what each holds** (§9.3,
    /// bl-dff8): the config pane's own browse — `for-each-ref refs/heads/config/`
    /// paired with each tip's `ls-tree`. The listing a
    /// [`ReadConfig`](Query::ReadConfig) of a lineage reads a path out of, so a
    /// headless operator picks a file it has seen rather than one it guessed.
    /// A third world-bytes query: the workspace's own git, never the snapshot.
    Lineages { workspace: String },
    /// **The model ids one provider offers** (§9.4, §5.1 #26, bl-dff8): the
    /// picker's roster (`bz --list-models`), spelled. Never stored and never
    /// cached by yog — the roster is the provider's fact, so this is a query
    /// and not a field, asked in the named workspace's wall exactly as
    /// [`Providers`](Query::Providers) is: the same provider row is signed in
    /// (and therefore listable) in one sphere and not in another.
    Models { workspace: String, provider: String },
    /// **Which branch this agent tracks on** (§16.3, bl-0164): the marks
    /// pane's `Read current`, over the same space [`Action::SetMarks`] re-reads
    /// after it writes. It never refuses and never spawns — the value's one
    /// home is the space's own balls config, so a workspace with no project
    /// (or an unprimed one) answers exactly as any other does, which is what
    /// makes the launched-then-pointed-at-a-project case askable.
    Marks { workspace: String },
    /// **The conversation itself** (§11 Altitude-2 Transcript, bl-6233): every
    /// committed `messages/` entry of one agent, with the live streaming tail
    /// folded on when a call is in flight. The first of the §11 inspector
    /// family — the five reads that were reachable from no seat but the window,
    /// which is what made a chat unreadable headless (REMOTE §9 step 1).
    ///
    /// **The in-flight tail is folded, not dropped.** The tail is the
    /// snapshot's own [`Stream`](crate::git_tree::Stream), already carried by
    /// the [`Deps`](super::dispatch::Deps) this query is answered from, so
    /// folding it costs no read and keeps one derivation behind both seats: a
    /// headless answer that stopped at the committed half would say something
    /// different from the window about the same moment, which is the exact
    /// parity §8.5 exists to hold.
    Transcript { workspace: String, agent: String },
    /// **The live tail, followed** (REMOTE §3, §10; DESIGN §7.2, bl-73e7) — the
    /// second follow-class read, and the one the operator ruling of 2026-08-22
    /// minted: one conversation's streaming answer, delivered as it is written
    /// rather than as it is asked for.
    ///
    /// Its subject is the same fold [`Transcript`](Query::Transcript) carries
    /// on its tail — `inspector::live_tail` — so the two cannot describe one
    /// moment differently (bl-6233). What differs is the **cadence**: an intake
    /// that can hold a connection answers a frame per growth of the open
    /// `response.json` and terminates the stream when the step commits, and an
    /// intake that cannot answers the tail as of now, which is the general path
    /// with one frame. The pull read stays the fallback, so a seat that loses
    /// the lane keeps the tail at ask cadence rather than losing it.
    Follow { workspace: String, agent: String },
    /// **Every step one conversation has taken** (§11 Altitude-2 Steps): the
    /// cheap per-step summary list — framing, attempts, tokens, timestamps, the
    /// §8.3 login affordance and the §7.3 wound. The agent's liveness is read
    /// off the snapshot rather than asked for, because it is a fact the world
    /// already published and a parameter would let a seat contradict it.
    Steps { workspace: String, agent: String },
    /// **One step's records, drilled in** (§11 Altitude 2): `meta`, `request`,
    /// `staging`, every `response.json` event and every tool call's
    /// input/output, each as a jsonview doc — plus each **capture log** that has
    /// bytes (bl-83d6): the step's own `stderr.log` and the agent's
    /// `driver.log`, as bounded previews rather than docs, because nothing
    /// parsed them. The second tier, asked by
    /// sequence name exactly as the list answers it — a step the tree does not
    /// hold answers absent records rather than refusing, the same forgiving
    /// read the window makes (§10: never a false definite).
    Step {
        workspace: String,
        agent: String,
        seq: String,
    },
    /// **The agent worktree, read-only** (§11 Altitude-2 Files): the bounded
    /// sorted listing, and — when `path` names one of its listed files — that
    /// file's bounded preview. The [`WorkDiff`](Query::WorkDiff) shape, for the
    /// same reason: a listing and one entry's bytes are one question asked at
    /// two depths. `path` is **resolved against the listing yog just built**,
    /// never joined blind, so this read can open nothing the same answer did
    /// not already name.
    ///
    /// **`at` names the tree, and a tree is a selection, not a fold** (REMOTE
    /// §9.7, bl-44e9). VISION V1.2's pin folds four tabs to one commit, and
    /// three of them read something the seat already holds — the transcript is a
    /// prefix of the chat it was answered, the budget is a rollup on the notch,
    /// the spine is the answer itself. This one is the tab whose *subject* is a
    /// different tree, so it is the one the commit has to reach, and it reaches
    /// it the way [`Step`](Query::Step)'s `seq` and
    /// [`WorkDiff`](Query::WorkDiff)'s `file` do: as a parameter naming **which
    /// thing you are asking about**. `None` is the live worktree. Nothing about
    /// the operator's *selection* crosses — a seat that has pinned nothing asks
    /// nothing different, which is what keeps DESIGN §8.5's "views gain no
    /// boundary representation" true of this.
    Files {
        workspace: String,
        agent: String,
        path: Option<String>,
        at: Option<String>,
    },
    /// **The config commit this conversation is frozen on** (§9.3, §5.1 #17;
    /// VISION V1.2's *config-frozen-at*; REMOTE §9.7, bl-13f9): the nearest
    /// `config/*` ancestor of a commit, whether that ancestor is still a
    /// lineage's own tip, and every path its tree holds.
    ///
    /// **`at` names the commit, and a commit is a selection** — the
    /// [`Files`](Query::Files) shape, and for the same reason: this is the
    /// *second* of VISION V1.2's four pinnable tabs whose subject is a tree the
    /// pin names rather than something the seat already holds. `None` is the
    /// agent's own branch tip, resolved off the published snapshot, so a seat
    /// that has pinned nothing asks nothing different and no seat has to know a
    /// tip before it may ask. Nothing about the operator's *selection* crosses.
    ///
    /// It **refuses** rather than answering absent, unlike its siblings: the
    /// derivation is a walk of the workspace's own git and it fails the way
    /// [`Lineages`](Query::Lineages) fails — a defective or unfetched
    /// workspace, or a commit that forks off no config lineage at all. Absence
    /// would read as *this conversation has no policy*, which is never true.
    Governing {
        workspace: String,
        agent: String,
        at: Option<String>,
    },
    /// **The step spine** (VISION V1, §11): one notch per operable commit and
    /// the child cards hanging off them. The notches are answered; the
    /// operator's *pin* is not — a pin is a viewport fold, and §8.5 files folds
    /// under views, exactly as [`Conversations`](Query::Conversations) answers
    /// the all-collapsed list.
    Rail { workspace: String, agent: String },
    /// **The undelivered mail** (§11 Inbox, ARCH §2.11): one agent's deposit
    /// files, each parsed beside its verbatim bytes.
    Inbox { workspace: String, agent: String },
    /// **One conversation as a seat sees it** (REMOTE §9.4, bl-1eb0): who is
    /// selected, what the conversation is called, its own §3.5 liveness, the §6
    /// marks it wears, what is in flight in it and the two §8.2 verb gates.
    ///
    /// The seventh member of the conversation-addressed family, and the one
    /// that made the other six paintable by a face holding no world: the §11
    /// centre pane derived all of this on the frame thread out of the engine's
    /// agent set, which is not a thing a wire can carry. Every field is a fold
    /// the boundary already owned; only the spelling is new.
    Agent { workspace: String, agent: String },
    /// One workspace's effective provider table with the §5.1 #22 credential
    /// presence, rendered (§8.5, bl-0164): the same derivation the §8.3 login
    /// pane's `↻ providers + credentials` paints — one derivation, and since
    /// bl-20cb the window has exactly one seat at it, so this reply and that
    /// pane are the whole set.
    ///
    /// **It names its workspace** (bl-fcd5). Providers and their sign-ins live
    /// inside a wall since the blast-radius ruling, so there is no global
    /// table to answer: the same provider reads *signed in* in one sphere and
    /// not in another, and a query that named none could only answer for
    /// whichever wall happened to be standing.
    Providers { workspace: String },
    /// **This workspace's registered clients** (REMOTE §5, bl-4e08): who
    /// participates in it, which of them holds a live connection right now, and
    /// what each advertises. Three reads joined at the moment they are asked —
    /// the §4.1 registration listing, the wire server's presence RAM, and each
    /// client's own advertised set — so nothing is stored that could go stale
    /// and a flap needs no invalidation.
    ///
    /// **It is a point-in-time observation, deliberately** (REMOTE §5): the
    /// seat sees the flap and the model's cached prefix never does, which is
    /// why presence is answered here rather than declared anywhere durable.
    Clients { workspace: String },
    /// **A tool host's next work** (REMOTE §3, §5; bl-024b) — *the*
    /// follow-class read, and the first with a consumer. The answer stays
    /// pending until this client has an invocation or the engine's hold
    /// expires, which is why it is a read and not a poll: a machine waiting on
    /// a machine is the ask rate REMOTE §10 set as the criterion for minting
    /// one.
    ///
    /// **The identity is the intake's**, exactly as an advertisement's is: a
    /// connection drains its own queue, and a `client` field here would let one
    /// connection take another's work. An intake carrying no client identity —
    /// the deposit inbox, `yog gesture`, the window — refuses in band.
    ///
    /// It names no workspace: a tool set is a fact about a machine (REMOTE
    /// §5.1) and so is the queue of calls to it.
    Invocations,
    /// **What one routed invocation captured** (REMOTE §5): the asking side's
    /// poll, answered `null` while the far machine still runs it. Bounded by
    /// the *asker's* patience rather than the engine's — a vanished client is
    /// this read answering nothing until its caller gives up and says so, which
    /// is the visible in-band refusal §5 asks for and never a hang.
    Capture { invocation: String },
}
