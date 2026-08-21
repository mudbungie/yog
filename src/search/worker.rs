//! The thread a window's searches run on (§8.5) — never the frame, and since
//! bl-44e9 **over the wire** (REMOTE §1.2, §9.7).
//!
//! The frame renders published values and derives nothing (§7.2); a search walks
//! every transcript in the world, so it is the derivation worker's shape applied
//! to a second question: the frame *asks* through the
//! [`SearchCell`](crate::state::SearchCell) and renders whatever answer has
//! landed, exactly as it renders whatever snapshot has landed. There is no
//! frame-side wait to make the window stutter, and no request channel — the cell
//! is both directions.
//!
//! **What bl-44e9 changed is where the walk happens, and nothing else.** The
//! window read its own snapshot here; it asks the engine now, like every other
//! read the window makes. It stays a thread of its own, and §9.7 weighed exactly
//! that: `Query::Search` walks every transcript in the world and a standing
//! question is re-asked every `ASK_PERIOD`, so riding the
//! [`asker`](crate::wire::asker) would turn a once-per-ask walk into a 2 Hz one
//! **and** put it in front of every other surface's answer — the poster's own
//! ruling (*"an act runs as long as the verb behind it"*) pointed at a read. So
//! the mechanism is unchanged and the read crosses; the win is §1.2 compliance,
//! not one fewer thread.
//!
//! It is a thread of its own rather than a stage of the derivation pass for its
//! original reason too: a long search must not delay a re-derivation — staleness
//! is what the §7.3 wound banner is *for*, and a search is not a wound.
//!
//! **A refusal is an unreadable source.** The engine can say no — an unresolved
//! world, a dead listener — and [`Found::unreadable`] is already *"each
//! unreadable source, named with why"*, which is exactly what that sentence is.
//! So a refused search paints the reason where a mangled transcript would have,
//! with no second state and no false *"no matches"*.
//!
//! **The abandon predicate went with the walk.** In process a superseded search
//! stopped mid-walk; over the wire the engine finishes what it was asked and the
//! answer for a question nobody is asking any more is dropped on publish. The
//! cost is one wasted walk on the engine, and the alternative — a cancel the
//! boundary would have to carry — is a mechanism for an optimisation.
//!
//! The other two seats need none of this. `yog gesture` and the deposit consumer
//! are already off-frame, and a process that asked has nothing else to do, so
//! they run [`run`](super::run) straight through
//! [`answer`](crate::boundary::answer::answer). One engine, three seats.

use super::Found;
use crate::boundary::reply::Reply;
use crate::boundary::{Gesture, Query, codec};
use crate::state::SearchCell;
use crate::wire::client::Seat;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the searcher looks for an ask. A latency knob only: an ask waits,
/// it never rots — the same reading the §8.5 consumer poll gets.
const SEARCH_POLL: Duration = Duration::from_millis(50);

/// One window's searcher: its seat on the wire, and the cell it is asked
/// through. Built by [`Engine::searcher`](crate::engine::Engine::searcher) so
/// the model never spawns its own thread — a test drives [`pass`](Self::pass) by
/// hand, which is the same reason `Engine::asker` hands back a value.
pub struct Searcher {
    seat: Seat,
    asks: SearchCell,
}

impl Searcher {
    pub(crate) fn new(seat: Seat, asks: SearchCell) -> Self {
        Self { seat, asks }
    }

    /// One pass: answer the outstanding ask, if there is one, by asking the
    /// engine. Returns whether an answer was published — `false` both when
    /// nothing was asked and when the run was superseded mid-flight, which are
    /// the same fact from here (the current question is unanswered either way,
    /// and the next pass takes it).
    pub fn pass(&self) -> bool {
        let Some((seq, text)) = self.asks.pending() else {
            return false;
        };
        let question = codec::encode(&Gesture::Ask(Query::Search { text: text.clone() }));
        self.asks.publish(seq, self.landed(&question, text));
        self.asks.seq() == seq
    }

    /// What one ask earned. Three arms and no fourth: the answer, the engine's
    /// refusal carried as an unreadable source, and a reply of another kind —
    /// which is a codec that has drifted from the query it answers, a defect
    /// rather than a state, so nothing is invented for it.
    fn landed(&self, question: &serde_json::Value, needle: String) -> Found {
        match self.seat.answered(question) {
            Ok(Reply::Search(found)) => found,
            Ok(_) => Found::default(),
            Err(said) => Found {
                needle,
                hits: Vec::new(),
                unreadable: vec![said],
            },
        }
    }

    /// Run [`pass`](Self::pass) forever, parked between looks — the
    /// [`Consumer`](crate::boundary::consumer::Consumer) shutdown shape exactly:
    /// a stop flag, an unpark, a join.
    pub fn start(self) -> SearchThread {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                self.pass();
                std::thread::park_timeout(SEARCH_POLL);
            }
        });
        SearchThread {
            stop,
            handle: Some(handle),
        }
    }
}

/// The searcher thread's handle; [`Drop`] signals stop, unparks and joins.
pub struct SearchThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for SearchThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests;
