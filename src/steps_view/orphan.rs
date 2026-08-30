//! The **orphaned tail** (bl-ace6, widened by bl-abba): the transcript ends
//! on something that is owed a driver, and no driver is there — plus, when
//! the last one left words behind, why.
//!
//! The §7.3 wound beside this covers a driver that died *inside a step*.
//! The gap it left is the driver that died **before creating one**, or
//! **between two of them**: the ordinary 2nd..Nth message of a conversation
//! is driven by a child *litany* launched (ARCH §2.11 — never a yog spawn,
//! so no §8.1 sink), and when that driver errors out — an unpaired-tail
//! decline, a lease fault, a crashed launch, an executor killed mid-tool-
//! window — the steps tree is unchanged, `ops.jsonl` says `exit 0`, and the
//! transcript simply stops. STORIES INV-2 (no swallowed errors) is the
//! invariant that state violates.
//!
//! The **state** is two observations, both already derived, nothing
//! stored (§5.1 #13):
//!
//! - **The newest transcript entry is owed an answer** — one of the two
//!   [`Tail`] shapes below. Which one it is is read through the transcript
//!   module's own classifier (`transcript::classify`), never a second
//!   reading of a filename here, and only the tail file is opened: the
//!   predicate must not cost a resting conversation the whole record.
//! - **Nobody is driving** — the agent's lock is free (§3.5). Both shapes
//!   are laid down *under* a driver's lock and the driver holds that lock
//!   across them, so on a healthy branch this pair exists only for the
//!   relaunch gap — which is why the banner rides the same grace window the
//!   wound does (bl-90bf), and a send whose driver has not been seen yet
//!   never alarms.
//!
//! **Two shapes, one state, and deliberately not two of everything**
//! (bl-abba). A crashed tool window is the same fact about the same agent,
//! read off the same disk, said in the same seat, through the same grace
//! gate — so it is a second [`Tail`] on this state rather than a third
//! banner beside it. The wire pays for that the way the wound already did
//! (bl-fb87): a `(bool, Option<reason>)` pair stops being a bijection at
//! the third arm, so the class is a token and the reason resolves the rest.
//!
//! The **reason** is the tail of `steps/<agent>/driver.log` — where
//! litany binds every launched driver's stderr (lernie 0.0.9, its
//! bl-55f9; the file yog pinned that release for and then never read).
//! It is append-only across launches, so its content is never the
//! trigger — a stale line from a healed crash must not alarm — only the
//! diagnosis, read exactly when the state holds (a healthy conversation
//! pays no syscall for it).
//!
//! Like the wound, this is not a new stored fact and not a new agent
//! state: the badge vocabulary is untouched (the ■ stopped ruling and
//! attention's rest-not-wound rule both stand — bl-d816 tracks whether
//! they ever should not), and everything here is re-derived per reading.

use std::path::Path;

use super::wound::driven;
use crate::git_tree::AgentState;
use crate::transcript::{Block, EntryKind};

/// What the transcript's tail is owed. Both shapes are "a driver was going
/// to answer this and is not there"; they differ only in what the last
/// entry is, which is the whole of what the operator needs told apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tail {
    /// A delivered `NNN-<sender>.md` — mail nobody is answering (bl-ace6).
    /// Committed tool results and model output are `.json`; only mail ends
    /// `.md` (ARCH §2.3 *Origins*).
    Mail,
    /// A model entry whose `tool_use` blocks no `tool_result` answers
    /// (bl-abba). Only the newest entry can be in this shape — a call
    /// answered later has its result committed *after* it — so no pairing
    /// walk is needed and none is done.
    ToolWindow,
}

/// The sentence yog renders for the mail shape — the §7.3 rendered fact.
pub const ORPHANED_MAIL: &str = "a delivered message has no driver";

/// The same for the tool-window shape. It carries the remedy because the
/// remedy is one gesture and the operator's whole problem is that the
/// conversation reads as an idle one nobody needs to touch: an executor
/// that died mid-window leaves an agent that looks finished, and on an
/// unattended box that window has no upper bound. A deposit settles it —
/// the next drive boundary writes an in-band `is_error` `tool_result` per
/// unanswered id before delivery (litany ARCH §6, its bl-4187, consumed in
/// bl-4c1f) and the branch carries on.
pub const ORPHANED_WINDOW: &str =
    "a tool call has no driver — the turn died mid-tool-window; a message revives it";

/// What the banner adds when `driver.log` has nothing to quote — the
/// honest end of the trail, said outright (the wound's own discipline).
const MUTE: &str = "and driver.log has no words — nothing on disk says why";

/// How the banner introduces the captured bytes. It names the **file**:
/// an operator who wants more than the tail must be told where the
/// whole of it lives.
const SPOKE: &str = "the last driver's words (driver.log):";

/// The §11 banner's leading mark — never the only carrier (§11 glyph
/// doctrine); the sentence beside it states the fact in words.
const ALARM: &str = "⚠";

impl Tail {
    /// The class, in words — one home per shape, so the banner and every
    /// assertion against it read the same string.
    fn says(self) -> &'static str {
        match self {
            Tail::Mail => ORPHANED_MAIL,
            Tail::ToolWindow => ORPHANED_WINDOW,
        }
    }
}

/// The orphaned-tail state **and its reason** — the [`super::Wound`]
/// shape, for the wound's own reason: an orphan with nothing to say is
/// a real, distinct answer, and `Some("")` would be one fact with two
/// encodings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Orphan {
    /// Not orphaned: nothing owed on the tail, or a driver is at work.
    #[default]
    None,
    /// A tail owed an answer, and no words anywhere saying why.
    Mute(Tail),
    /// A tail owed an answer, and the last driver's own words — the tail
    /// of `driver.log`, verbatim.
    Spoke(Tail, String),
}

impl Orphan {
    /// Is the tail orphaned? The §11 Altitude-1 banner's gate.
    pub fn orphaned(&self) -> bool {
        !matches!(self, Orphan::None)
    }

    /// The whole rendered fact, in words — one home, so the banner
    /// cannot drift from the derivation and the sentence is assertable
    /// without a frame. `Orphan::None` has no sentence: the caller
    /// gates on [`orphaned`](Self::orphaned).
    pub fn banner(&self) -> String {
        match self {
            Orphan::None => String::new(),
            Orphan::Mute(tail) => format!("{ALARM} {} — {MUTE}", tail.says()),
            Orphan::Spoke(tail, words) => format!("{ALARM} {} — {SPOKE} {words}", tail.says()),
        }
    }
}

/// Derive the state for one agent. The `driver.log` read is gated on
/// the predicate, and both bounds on how much of it is read are the
/// wound's own borrows: [`crate::opslog::detached::captured`] for how
/// much of a capture file yog ever reads,
/// [`crate::opslog::rows::stderr_tail`] for how much a surface says.
pub(super) fn read(workspace: &Path, agent_id: &str, state: AgentState) -> Orphan {
    if driven(state) {
        return Orphan::None;
    }
    let Some(tail) = tail(workspace, agent_id) else {
        return Orphan::None;
    };
    // **A parked call is not a crashed one.** The capability control answers
    // `hold` by parking the invocation before it executes (§8.6) and litany
    // records which one in `refs/litany/held/<agent-id>` — a branch that is
    // waiting on the operator wears exactly the tool-window shape, and it is
    // waiting on purpose. Read live and only here, the way the answer gesture
    // reads it (`control::hold`): an unreadable mark is an absent one, which
    // is that module's own discipline and errs toward saying something rather
    // than toward silence.
    if tail == Tail::ToolWindow && crate::control::hold::read(workspace, agent_id).is_some() {
        return Orphan::None;
    }
    // Bound rather than chained, like `summarize` in `super`: tarpaulin's
    // llvm engine mis-attributes a multi-line method chain's tail as
    // uncovered.
    let dir = workspace.join(super::STEPS_DIR).join(agent_id);
    let captured = crate::opslog::detached::captured(&dir.join(super::records::DRIVER_LOG_FILE));
    let words = crate::opslog::rows::stderr_tail(captured.trim());
    if words.is_empty() {
        Orphan::Mute(tail)
    } else {
        Orphan::Spoke(tail, words)
    }
}

/// Which [`Tail`] the newest `messages/` entry is, if either. Filename order
/// is entry order (ARCH §2.3: the zero-padded counter), so the newest name is
/// a `max` over the listing; only that one file is opened, and the transcript
/// module's own classifier says what it is. An absent or empty directory has
/// no tail and is never an orphan.
fn tail(workspace: &Path, agent_id: &str) -> Option<Tail> {
    let dir = crate::files_view::agent_worktree(workspace, agent_id).join("messages");
    let newest = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .max()?;
    let raw = std::fs::read(dir.join(&newest)).unwrap_or_default();
    match crate::transcript::classify(&newest, &raw) {
        EntryKind::Delivered { .. } => Some(Tail::Mail),
        EntryKind::Model { blocks, .. } => calls(&blocks).then_some(Tail::ToolWindow),
        _ => None,
    }
}

/// Does this entry call a tool at all? A model turn that ended on text is
/// a conversation resting, which is the ordinary shape and no orphan.
fn calls(blocks: &[Block]) -> bool {
    blocks.iter().any(|b| matches!(b, Block::ToolUse { .. }))
}
