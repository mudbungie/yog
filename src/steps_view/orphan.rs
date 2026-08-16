//! The **orphaned-mail state** (bl-ace6): a delivered message nobody is
//! answering — and, when the last driver left words behind, why.
//!
//! The §7.3 wound beside this covers a driver that died *inside a step*.
//! The gap it left is the driver that died **before creating one**: the
//! ordinary 2nd..Nth message of a conversation is driven by a child
//! *lernie* launched (ARCH §2.11 — never a yog spawn, so no §8.1 sink),
//! and when that driver errors out at the boundary — an unpaired-tail
//! decline, a lease fault, a crashed launch — the deposit has already
//! landed, `ops.jsonl` says `exit 0`, the steps tree is unchanged, and
//! the transcript simply stops. STORIES INV-2 (no swallowed errors) is
//! the invariant that state violates.
//!
//! The **state** is two observations, both already derived, nothing
//! stored (§5.1 #13):
//!
//! - **The newest transcript entry is delivered mail** — a
//!   `messages/NNN-<sender>.md`. Committed tool results and model output
//!   are `.json`; only mail ends `.md` (ARCH §2.3 *Origins*).
//! - **Nobody is driving** — the agent's lock is free (§3.5). Delivery
//!   only ever happens under a driver's lock and the driver holds that
//!   lock through the model call, so on a healthy branch this pair
//!   exists only for the relaunch gap — which is why the banner rides
//!   the same grace window the wound does (bl-90bf), and a send whose
//!   driver simply has not been seen yet never alarms.
//!
//! The **reason** is the tail of `steps/<agent>/driver.log` — where
//! lernie binds every launched driver's stderr (lernie 0.0.9, its
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

/// The sentence yog renders for this class — the §7.3 rendered fact.
pub const ORPHANED_MAIL: &str = "a delivered message has no driver";

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

/// The orphaned-mail state **and its reason** — the [`super::Wound`]
/// shape, for the wound's own reason: an orphan with nothing to say is
/// a real, distinct answer, and `Some("")` would be one fact with two
/// encodings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Orphan {
    /// Not orphaned: no mail on the tail, or a driver is at work.
    #[default]
    None,
    /// Mail with no driver, and no words anywhere saying why.
    Mute,
    /// Mail with no driver, and the last driver's own words — the tail
    /// of `driver.log`, verbatim.
    Spoke(String),
}

impl Orphan {
    /// Is the mail orphaned? The §11 Altitude-1 banner's gate.
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
            Orphan::Mute => format!("{ALARM} {ORPHANED_MAIL} — {MUTE}"),
            Orphan::Spoke(words) => format!("{ALARM} {ORPHANED_MAIL} — {SPOKE} {words}"),
        }
    }
}

/// Derive the state for one agent. The `driver.log` read is gated on
/// the predicate, and both bounds on how much of it is read are the
/// wound's own borrows: [`crate::opslog::detached::captured`] for how
/// much of a capture file yog ever reads,
/// [`crate::opslog::rows::stderr_tail`] for how much a surface says.
pub(super) fn read(workspace: &Path, agent_id: &str, state: AgentState) -> Orphan {
    if driven(state) || !mail_on_tail(workspace, agent_id) {
        return Orphan::None;
    }
    // Bound rather than chained, like `summarize` in `super`: tarpaulin's
    // llvm engine mis-attributes a multi-line method chain's tail as
    // uncovered.
    let dir = workspace.join(super::STEPS_DIR).join(agent_id);
    let captured = crate::opslog::detached::captured(&dir.join(super::records::DRIVER_LOG_FILE));
    let words = crate::opslog::rows::stderr_tail(captured.trim());
    if words.is_empty() {
        Orphan::Mute
    } else {
        Orphan::Spoke(words)
    }
}

/// Whether the newest `messages/` entry is delivered mail (`.md`) —
/// the transcript's own naming rule (ARCH §2.3: order lives in the
/// zero-padded filename, mail is the one `.md` origin). An absent or
/// empty directory has no tail and is never an orphan.
fn mail_on_tail(workspace: &Path, agent_id: &str) -> bool {
    let dir = crate::files_view::agent_worktree(workspace, agent_id).join("messages");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return false;
    };
    let newest = entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .max();
    newest.is_some_and(|name| name.ends_with(".md"))
}
