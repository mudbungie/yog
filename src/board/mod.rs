//! The V4 board (VISION §5 V4, DESIGN §11): the balls section as columns.
//!
//! **Nothing here is a second status model.** The three live rungs are balls'
//! own — `claimant ⇒ claimed`, else an unresolved *claim*-blocker ⇒ `blocked`,
//! else `ready` — read through [`crate::projects::balls::ladder`], the same
//! function the §3.5 join and `bl list` derive from. The fourth column is
//! balls' *other* published predicate, `Task::closeable`: a **close**-blocker
//! never shows as a status ("it only gates the finish"), so a ball that is
//! claimable but cannot deliver is `ready` to the ladder and yet is not what an
//! operator means by ready. [`Column`] is those two axes crossed, total, over
//! stored blockers alone — no new field, no new index, no cached status.
//!
//! The rest of a row is likewise a projection of facts that already have
//! owners: the gate is the blocking ball (what *mints* it is that ball's own
//! close), the drones are the §3.3 goal stamps resolved to their roots — the
//! very set [`crate::spend`] attributes by, so "whose spend is this" and "which
//! conversation is on it" are one derivation — and the spend column is §3.5's
//! join over the worker's `steps/` fold.
//!
//! **Pure over the published snapshot**, which is what makes it a §8.5 query
//! (`Query::Board`) rather than a widget: the window renders it, `/board` and
//! the `{"op":"board"}` envelope answer it, one implementation.
//!
//! **V4.2's armed-loop facts ride here too** (bl-66fb), and they ride the same
//! rule: [`Board::fleet`] is one entry per **armed** workspace and is empty in
//! every world that has not armed one — which is every world by default. There
//! is no chip that says "unarmed", because a chip announcing the absence of a
//! mechanism is the capability theater VISION §5 refuses; unarmed, this is
//! byte-for-byte today's board (V4's burden check, verbatim: *"unarmed, the
//! board is today's balls section"*). The facts themselves are derived in
//! [`crate::fleet::facts`] — cap from the config entry, count from the rows
//! below, the last tick from the ops tail, the ceiling from §3.5's own gate.

mod rollup;
mod rows;
#[cfg(test)]
mod tests;

pub use rollup::descendants;
pub use rows::{Drone, Gate, stamped_roots};

use crate::app::Snapshot;
use crate::projects::balls::{Ball, Status};
use crate::projects::join::JoinState;
use crate::spend::{Figure, Prices};
use std::collections::{HashMap, HashSet};

/// A board column — the operator's four buckets. `Ready`/`Blocked`/`Claimed`
/// are balls' ladder rungs verbatim; `Gated` is the ladder's `Ready` split by
/// balls' own `closeable` predicate, which is a fact about the same stored
/// blockers and not a rung anyone stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Column {
    Ready,
    Gated,
    Claimed,
    Blocked,
}

impl Column {
    /// Left-to-right board order: what you can take, what waits on a gate, what
    /// is running, what cannot start.
    pub const ALL: [Column; 4] = [
        Column::Ready,
        Column::Gated,
        Column::Claimed,
        Column::Blocked,
    ];

    /// The column's stable word — the header, and the headless spelling. Three
    /// of the four are `Status::word` by construction (asserted in tests), so
    /// the board and `bl list` cannot drift apart in vocabulary either.
    ///
    /// `pub(crate)` for AGENTS rule 2 (a `pub fn` returns owned), which the
    /// borrowed `&'static str` would break; the crate is the only consumer and
    /// `Origin::as_str` keeps the same shape for the same reason.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Column::Ready => "ready",
            Column::Gated => "gated",
            Column::Claimed => "claimed",
            Column::Blocked => "blocked",
        }
    }
}

/// The two axes crossed — total, and the whole column derivation. `gated` is
/// "an unresolved close-blocker remains", i.e. `!Task::closeable`.
///
/// A **claimed** ball that is also gated stays in `claimed`: a drone holds it
/// and is working, which is what the operator needs to see; its gate still
/// renders on the row ([`BoardRow::gates`] is filled for every column), so
/// nothing is hidden by the bucket it lands in.
pub fn column(status: Status, gated: bool) -> Column {
    match (status, gated) {
        (Status::Claimed, _) => Column::Claimed,
        (Status::Blocked, _) => Column::Blocked,
        (Status::Ready, true) => Column::Gated,
        (Status::Ready, false) => Column::Ready,
    }
}

/// One ball as the board renders it. Every field is derived on read; the row is
/// a value, never a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRow {
    /// The project's §5.1 #1 wire name and the workspace's §3.1 leaf — both
    /// paths until bl-b4b5, and narrowed with the [`JoinRow`] they are copied
    /// off (REMOTE §8.1): the board is a reply, and a reply says names.
    pub project: String,
    pub id: String,
    pub title: String,
    pub priority: i64,
    pub column: Column,
    /// The §3.5 row state beside the column — they answer different questions:
    /// the column is the ladder, the state is the *binding* (bound here,
    /// claimed elsewhere, project missing).
    pub state: JoinState,
    pub workspace: Option<String>,
    pub claimant: Option<String>,
    pub parent: Option<String>,
    /// The unresolved close-blockers, each naming the ball whose close mints
    /// it. Empty for every ungated row, whatever its column.
    pub gates: Vec<Gate>,
    /// The conversations working this ball (§3.3 goal stamps resolved to their
    /// roots). Named, not re-rendered: the seat resolves each to the very
    /// `ConvRow` the conversation list already paints — one object everywhere.
    pub drones: Vec<Drone>,
    /// The §3.5 figure for this ball; `None` when it is bound to no workspace,
    /// which is the honest answer — an unclaimed ball has spent nothing yet.
    pub spend: Option<Figure>,
    /// The epic rollup: this ball plus its live descendants, folded across
    /// every workspace they are claimed in. `None` when the ball has no
    /// children — a leaf's rollup is its own figure and a second copy of it
    /// would be noise.
    pub rollup: Option<Figure>,
}

/// The board: every live ball of every visible project, in column order, plus
/// the facts of any §4.3 loop armed over them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Board {
    pub rows: Vec<BoardRow>,
    /// One entry per armed workspace (VISION §5 V4 item 2) — cap, count, tick,
    /// lease and where the ceiling will bind. **Empty is the ordinary world**:
    /// nothing is armed until the operator arms it, and unarmed the board must
    /// render exactly what it rendered before this existed.
    pub fleet: Vec<crate::fleet::Facts>,
}

impl Board {
    /// One column's rows, in the board's own order.
    pub fn in_column(&self, column: Column) -> Vec<BoardRow> {
        self.rows
            .iter()
            .filter(|r| r.column == column)
            .cloned()
            .collect()
    }

    /// How many rows a column holds — the header's count, derived, never kept.
    pub fn count(&self, column: Column) -> usize {
        self.rows.iter().filter(|r| r.column == column).count()
    }
}

/// Build the board off a published snapshot (§7.2) and the §3.5 price table.
///
/// Ordering is `bl list`'s leading key and then a total tiebreak: priority
/// ascending, then id, then project — deterministic across instances (I9),
/// which a `HashMap` iteration is not. Columns are a grouping the seat applies
/// over that one order, never a second sort.
///
/// The board **is** the §3.5 join, read one row at a time: every join row that
/// names a live ball becomes a board row, and the rows that name none —
/// delivered, unassigned-workspace, orphaned-project — fall away here rather
/// than through a fabricated status. That is why no row needs a default state:
/// the binding half comes from the join and the ladder half from the ball, and
/// a row only exists when both are present.
/// `ui` supplies the two §4.1 durables a board needs — the price table every
/// figure is joined through, and the ceiling the loop's next spawn will meet —
/// and `now_unix` is the caller's wall clock, so the derivation stays clock-free
/// and deterministic under test (`answer`'s own discipline).
pub fn build(snap: &Snapshot, ui: &crate::ui_state::UiState, now_unix: i64) -> Board {
    let prices = ui.prices();
    let rows = rows_of(snap, &prices);
    let fleet = crate::fleet::facts::of(snap, &prices, ui.ceiling(), &rows, now_unix);
    Board { rows, fleet }
}

/// The rows alone — the board's original derivation, unchanged.
fn rows_of(snap: &Snapshot, prices: &Prices) -> Vec<BoardRow> {
    // Keyed by the project's wire **name**, because that is what a join row
    // says since bl-b4b5 — resolved once for the whole index rather than per
    // row, which is the same rule `answer` follows for a query's address.
    let index: HashMap<String, (HashSet<&str>, HashMap<&str, &Ball>)> = snap
        .balls_by_project
        .iter()
        .map(|(project, balls)| {
            let by_id: HashMap<&str, &Ball> = balls.iter().map(|b| (b.id.as_str(), b)).collect();
            (
                snap.project_name(project),
                (by_id.keys().copied().collect(), by_id),
            )
        })
        .collect();
    let mut rows = Vec::new();
    for join in &snap.join_rows {
        let Some((live, by_id)) = index.get(&join.project) else {
            continue;
        };
        let Some(ball) = by_id.get(join.ball_id.as_str()) else {
            continue;
        };
        rows.push(rows::row(snap, prices, join, ball, live, by_id));
    }
    rows.sort_by(|a, b| {
        (a.priority, a.id.as_str())
            .cmp(&(b.priority, b.id.as_str()))
            .then_with(|| a.project.cmp(&b.project))
    });
    rows
}
