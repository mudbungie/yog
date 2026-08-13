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
    /// A destination that is not there yet answers empty text, the same "new
    /// file" reading every editor's own load already gives; a lineage refuses
    /// (its browse is the §9.3 pane's own gesture, bl-ee0a).
    ReadConfig { file: config::ConfigFile },
    /// **Which branch this agent tracks on** (§16.3, bl-0164): the marks
    /// pane's `Read current`, over the same space [`Action::SetMarks`] re-reads
    /// after it writes. It never refuses and never spawns — the value's one
    /// home is the space's own balls config, so a workspace with no project
    /// (or an unprimed one) answers exactly as any other does, which is what
    /// makes the launched-then-pointed-at-a-project case askable.
    Marks { workspace: PathBuf },
    /// One workspace's effective provider table with the §5.1 #22 credential
    /// presence, rendered (§8.5, bl-0164): the §8.3 login pane's
    /// `↻ providers + credentials` and the §9.5 config rows' capability read
    /// — one derivation, every seat.
    ///
    /// **It names its workspace** (bl-fcd5). Providers and their sign-ins live
    /// inside a wall since the blast-radius ruling, so there is no global
    /// table to answer: the same provider reads *signed in* in one sphere and
    /// not in another, and a query that named none could only answer for
    /// whichever wall happened to be standing.
    Providers { workspace: PathBuf },
}
