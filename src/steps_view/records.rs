//! **What records a step surface has** — the drill-in picker's row set (§11
//! Altitude 2), the words each seat carries, and the two capture-log file names
//! the §7.3 banners quote.
//!
//! The row set is **derived, not declared** (bl-83d6). It used to be a fixed
//! table of the five JSON records litany contracts to write, which is why the
//! whole of a long `stderr.log` was unreadable in-window: the only view of it
//! was the wound banner's tail, three lines of the file's last 4 KiB, with the
//! rest reachable only by leaving the window. The two capture logs are now
//! seats of their own, and their presence is read off the disk rather than
//! asserted here:
//!
//! - **The five JSON records are the step's contract.** litany writes them, so
//!   each keeps its seat unconditionally and an absent one paints "(absent)" —
//!   the *fact* that a record the step should hold is missing is worth a seat.
//! - **A log is evidence, and it only exists when something went wrong.**
//!   `stderr.log` is *"empty on an ordinary run"* (litany ARCH §2.3), so a
//!   permanent seat for it would be a dead row on every healthy step in the
//!   tree. It is offered exactly when the file has bytes — the presence rule is
//!   [`super::detail`]'s single read of it, never a second stat here.
//!
//! Both logs render as a [`crate::files_view::Preview`] rather than a
//! [`super::Doc`]: nothing parsed them, so the three-way parsed/absent/
//! unparseable vocabulary has nothing to say about them, while the bounded-file
//! one (64 KiB, the cap said outright, a NUL-bearing file declared binary) is
//! exactly the reading a log wants and is already what the Files tab shows.

use super::{StepDetail, StepTab};

/// One picker seat: which record, the on-disk file name that names it, and what
/// that name means to a reader who has never met the spec — the `Workspaces:`
/// label's idiom (bl-2d87, bl-3ffc), one home for all three.
pub(crate) type Record = (StepTab, &'static str, &'static str);

/// The adapter subprocess's captured stderr, per step (litany ARCH §2.3). One
/// home for the name: the §7.3 wound quotes this file's tail in its banner and
/// the picker seats the whole of it, and two spellings of one file name drift.
pub(crate) const STDERR_FILE: &str = "stderr.log";

/// Where litany binds a launched driver's stderr (ARCH §2.11; litany bl-55f9):
/// beside the step dirs, one per agent, append-only across launches. The
/// orphaned-mail banner quotes its tail; the picker seats the whole of it.
pub(crate) const DRIVER_LOG_FILE: &str = "driver.log";

/// The five records a step leaves behind — the picker's word, and what that
/// word means for someone reading it cold. `pub(crate)`: the §11
/// discoverability invariant (bl-68ac) makes the shell's own step-tab control
/// carry the same explanation, and two spellings of one fact drift — this is
/// its one home, the same argument the column table makes for a heading.
pub(crate) const RECORDS: [Record; 5] = [
    (
        StepTab::Meta,
        "meta",
        "The step's own note about itself: the commit it started from and the \
         times it began and ended.",
    ),
    (
        StepTab::Request,
        "request",
        "Exactly what was sent to the model to open this step — the prompt, the \
         history and the settings, as they went over the wire.",
    ),
    (
        StepTab::Staging,
        "staging",
        "The conversation entry being assembled out of this step's reply, caught \
         mid-write before it became part of the transcript.",
    ),
    (
        StepTab::Response,
        "response",
        "The model's reply as it streamed back, one event per line — text, tool \
         calls, usage and the end of each attempt.",
    ),
    (
        StepTab::Tools,
        "tools",
        "Every tool this step called, each with the arguments it was handed and \
         what it gave back.",
    ),
];

/// The step's own adapter stderr — the wound's reason, in full instead of three
/// lines of it.
const STDERR_SEAT: Record = (
    StepTab::Stderr,
    STDERR_FILE,
    "What the model adapter itself said while this step ran. Empty on an \
     ordinary run, so words here are the reason the call produced nothing — \
     shown whole, not just the tail the banner quotes.",
);

/// The agent's driver log — the one seat here that is not the step's own file,
/// and it says so in its own words.
const DRIVER_SEAT: Record = (
    StepTab::Driver,
    DRIVER_LOG_FILE,
    "What the programs driving this whole conversation said, oldest first — one \
     file for the agent, not for this step. A driver that died before it could \
     write a step says why here, and nowhere else.",
);

/// The seats the picker offers for one drilled-in step: the five contract
/// records, then each capture log the step actually has bytes in.
///
/// Order is fixed and the logs come last, so a step that grows evidence never
/// moves the seat an operator was aiming at — and a log seat appearing IS the
/// signal that something was written.
///
/// **`None` is the general path with nothing in it, not a special case**: a seat
/// whose drill-in has not landed yet (a step just picked, one round trip out —
/// REMOTE §9.7) holds no logs, so it offers no log seats. The five contract
/// records need no answer to be nameable, which is why the strip is there from
/// the first frame.
pub(crate) fn seats(detail: Option<&StepDetail>) -> Vec<Record> {
    let mut seats = RECORDS.to_vec();
    for (seat, log) in [
        (STDERR_SEAT, detail.and_then(|d| d.stderr.as_ref())),
        (DRIVER_SEAT, detail.and_then(|d| d.driver.as_ref())),
    ] {
        if log.is_some() {
            seats.push(seat);
        }
    }
    seats
}
