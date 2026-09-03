//! **The attention lane** (REMOTE §14.1, bl-09aa; gate bl-5f41):
//! [`Query::Attention`](super::Query::Attention) answered as a *sequence* by an
//! intake that can hold — the first frame the answer as of connect, a further
//! frame whenever the answer under this asker's scope **changes**.
//!
//! **It is the follow lane's third read, and its case is the criterion
//! inverted** (REMOTE §10). The transcript tail and the invocation mailbox hold
//! a connection because their subject moves faster than an operator looks; this
//! one holds because its asker *cannot re-ask* — a pocketed phone performs no
//! read at all, and the platform ends a backgrounded app's sockets (REMOTE
//! §14). The wire needs no push path because a standing ask already is one.
//!
//! **No new clock, no new watch, no subscription noun.** Change is discovered
//! at the derivation worker's own republish and nowhere else: every input to
//! the answer — the derived trees, and `ui.json`'s `seen` watermarks, which the
//! §7.2 worker adopts on its own watch (`derive::route`'s `adopt_ui`) — is in
//! the published [`Snapshot`], so a look whose cell holds the very pointer it last
//! computed from is finished before it starts. A quiet world costs one pointer
//! comparison per tick, and the §7.2 full sweep republishes every 15 s whether
//! anything moved or not, which is the outer bound on discovery.
//!
//! **Frames replace; they never append** — the opposite of the follow lane's
//! ruling (REMOTE §5.5) on that ruling's own argument. The append flip exists
//! because a transcript answer grows with the conversation and re-sending it is
//! quadratic; an attention answer is a handful of rows that grows with nothing,
//! so a delta encoding here would buy a fold contract to save bytes that never
//! multiplied. A seat paints the last frame it holds; a seat that drops the
//! lane re-asks and is whole on its first frame.
//!
//! **The age is a clock reading, not a change** ([`settled`]). Every row's
//! `age_secs` is `now - last_action`, so an answer compared with its ages in it
//! differs at every republish and the lane would degrade into a 15 s poll —
//! frames that wake a radio to say nothing, on the one seat whose battery is
//! the reason this lane exists. So the comparison is over the answer with its
//! ages zeroed, and the frame carries this moment's.
//!
//! **The hold is the follow lane's, unamended** (REMOTE §14.1): the same two
//! constants, so "the follow lane's bounded-hold discipline applies" is one
//! fact in the code rather than two numbers that can drift. The hold ends and
//! the lane re-asks — a stream that ended and a dial that failed are one case
//! (REMOTE §10). Thirty seconds is also short enough to sit inside any read
//! bound the connection itself is under, so a seat is given its terminator in
//! time to ask again rather than losing the connection under it.
//!
//! **Severability is the follow lane's too.** Nothing runs for a seat that
//! never holds the lane: the deposit inbox and every intake that cannot hold
//! answer `Query::Attention` through [`answer`](super::answer::answer) in one
//! frame, byte for byte what it answers today. A foot never reaches the lane
//! (REMOTE §4.2) — the read is not one of the three gestures its grade admits,
//! so the refusal is worded where every other one is.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::app::Snapshot;
use crate::state::SnapshotCell;
use crate::ui_state::{Clock, UiState};

use super::answer::queue::{QueueRow, queue};
use super::follow::{HOLD_TICK, HOLD_WAITS};
use super::reply::Reply;

/// One asker's standing attention read.
pub(crate) struct Attend {
    cell: SnapshotCell,
    /// The workspaces this client is registered in, **spent at connect**
    /// (REMOTE §4): the identity is per request and a held read is one request,
    /// so what every later look narrows to is the authorization this ask
    /// already carried.
    scope: BTreeSet<String>,
    ui_path: PathBuf,
    clock: Arc<dyn Clock>,
    /// The derivation the last computed answer came off, by pointer — the
    /// worker's republish, read as the one fact it is.
    seen: Option<Arc<Snapshot>>,
    /// The last answer written, with its ages zeroed ([`settled`]). `None`
    /// before the first frame, which is what makes that frame unconditional:
    /// the answer as of connect goes out whether it is empty or not.
    last: Option<Vec<QueueRow>>,
    waits: u32,
    quiet: u32,
    tick: Duration,
}

/// The answer with its clock taken out — the comparison key, never a frame.
fn settled(rows: &[QueueRow]) -> Vec<QueueRow> {
    rows.iter()
        .map(|row| QueueRow {
            age_secs: 0,
            ..row.clone()
        })
        .collect()
}

impl Attend {
    /// Hold the attention lane for a client scoped to `scope`, on the
    /// production bound.
    pub(crate) fn new(
        cell: SnapshotCell,
        scope: BTreeSet<String>,
        ui_path: PathBuf,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self::holding(cell, scope, ui_path, clock, HOLD_WAITS, HOLD_TICK)
    }

    /// The same, on a stated hold — a test names a short one rather than
    /// sleeping for real ([`Follow::holding`](super::follow::Follow::holding)'s
    /// own shape).
    pub(crate) fn holding(
        cell: SnapshotCell,
        scope: BTreeSet<String>,
        ui_path: PathBuf,
        clock: Arc<dyn Clock>,
        waits: u32,
        tick: Duration,
    ) -> Self {
        Self {
            cell,
            scope,
            ui_path,
            clock,
            seen: None,
            last: None,
            waits,
            quiet: 0,
            tick,
        }
    }

    /// **One look at the world, taken now** — the frame owed to this asker, or
    /// `None` for a look that owes none. Public to the crate for the follow
    /// lane's reason: this is the mechanism and [`next`](Iterator::next) is
    /// only the patience around it, so a test drives it with no clock and no
    /// sleep at all.
    pub(crate) fn look(&mut self) -> Option<Vec<QueueRow>> {
        let snap = crate::state::latest_snapshot(&self.cell);
        if self
            .seen
            .as_ref()
            .is_some_and(|held| Arc::ptr_eq(held, &snap))
        {
            return None;
        }
        self.seen = Some(Arc::clone(&snap));
        let ui = UiState::open(self.ui_path.clone());
        let rows = queue(&snap.scoped(&self.scope), &ui, self.clock.unix());
        let key = settled(&rows);
        if self.last.as_ref() == Some(&key) {
            return None;
        }
        self.last = Some(key);
        Some(rows)
    }
}

impl Iterator for Attend {
    type Item = Reply;

    /// The next frame, or the end of the hold. Parks between looks, which is
    /// the whole of what makes this a held read.
    ///
    /// There is no third answer here — no `Over` (REMOTE §14.1). A conversation's
    /// tail ends because its step commits; "what needs you" has no end, so the
    /// only reason this lane stops is the bound, and the seat's re-ask is what
    /// starts the next one.
    fn next(&mut self) -> Option<Reply> {
        loop {
            if let Some(rows) = self.look() {
                self.quiet = 0;
                return Some(Reply::Attention(rows));
            }
            self.quiet += 1;
            if self.quiet >= self.waits {
                return None;
            }
            std::thread::sleep(self.tick);
        }
    }
}

#[cfg(test)]
mod tests;
