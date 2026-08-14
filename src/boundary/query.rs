//! The **populating-read roster** (§8.5): every query both frontends ask, in
//! one enum. Its own file at §12's line budget (bl-765d), on the seam §8.5
//! already draws and the help table is already cut along — actions mutate,
//! queries populate. Nothing else moved with it: [`answer`](super::answer) is
//! still the one chokepoint, and [`Gesture`](super::Gesture) still names both
//! halves.

use super::config;
use std::path::PathBuf;

/// One populating read (§8.5): a §2 I1 derivation over the published snapshot,
/// answered by [`answer::answer`] — the same functions the frame's view-models
/// delegate to, so both frontends render one derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// The enumerated workspaces (§3.1) with their §6 attention rollups.
    Workspaces,
    /// The §11 conversation list of one workspace: one row per root agent,
    /// subtree-aggregated, attention > running > recency.
    Conversations { workspace: PathBuf },
    /// The §3.5 join rows — every ball⇄workspace binding fact.
    Balls,
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
        workspace: PathBuf,
        file: Option<crate::workdiff::WorkFile>,
    },
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
    Lineages { workspace: PathBuf },
    /// **The model ids one provider offers** (§9.4, §5.1 #26, bl-dff8): the
    /// picker's roster (`bz --list-models`), spelled. Never stored and never
    /// cached by yog — the roster is the provider's fact, so this is a query
    /// and not a field, asked in the named workspace's wall exactly as
    /// [`Providers`](Query::Providers) is: the same provider row is signed in
    /// (and therefore listable) in one sphere and not in another.
    Models {
        workspace: PathBuf,
        provider: String,
    },
    /// **Which branch this agent tracks on** (§16.3, bl-0164): the marks
    /// pane's `Read current`, over the same space [`Action::SetMarks`] re-reads
    /// after it writes. It never refuses and never spawns — the value's one
    /// home is the space's own balls config, so a workspace with no project
    /// (or an unprimed one) answers exactly as any other does, which is what
    /// makes the launched-then-pointed-at-a-project case askable.
    Marks { workspace: PathBuf },
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
    Transcript { workspace: PathBuf, agent: String },
    /// **Every step one conversation has taken** (§11 Altitude-2 Steps): the
    /// cheap per-step summary list — framing, attempts, tokens, timestamps, the
    /// §8.3 login affordance and the §7.3 wound. The agent's liveness is read
    /// off the snapshot rather than asked for, because it is a fact the world
    /// already published and a parameter would let a seat contradict it.
    Steps { workspace: PathBuf, agent: String },
    /// **One step's records, drilled in** (§11 Altitude 2): `meta`, `request`,
    /// `staging`, every `response.json` event and every tool call's
    /// input/output, each as a jsonview doc. The second tier, asked by
    /// sequence name exactly as the list answers it — a step the tree does not
    /// hold answers absent records rather than refusing, the same forgiving
    /// read the window makes (§10: never a false definite).
    Step {
        workspace: PathBuf,
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
    Files {
        workspace: PathBuf,
        agent: String,
        path: Option<String>,
    },
    /// **The step spine** (VISION V1, §11): one notch per operable commit and
    /// the child cards hanging off them. The notches are answered; the
    /// operator's *pin* is not — a pin is a viewport fold, and §8.5 files folds
    /// under views, exactly as [`Conversations`](Query::Conversations) answers
    /// the all-collapsed list.
    Rail { workspace: PathBuf, agent: String },
    /// **The undelivered mail** (§11 Inbox, ARCH §2.11): one agent's deposit
    /// files, each parsed beside its verbatim bytes.
    Inbox { workspace: PathBuf, agent: String },
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
    Providers { workspace: PathBuf },
}
