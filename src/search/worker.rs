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
//! **It fans out** (REMOTE §8.2, bl-670c): a search asks *every* channel this
//! window holds and unions the answers, local channel first and then the entries
//! in leaf order. §8.2 promises exactly that and no more — *"search fans out and
//! unions … no global ordering, dedupe or clock is promised across engines"* —
//! so each host's block is ranked and bounded by that host, and the union is the
//! blocks in channel order. Re-ranking across engines would need one clock over
//! several worlds, and re-cutting the union to [`MAX`](crate::search::MAX) would
//! silently delete a whole host's answer to keep a bound that is already kept
//! once per engine.
//!
//! **The union is published as it arrives, not when it is complete.** Each
//! channel's answer supersedes the last, so the local hits paint at the moment
//! they land and an entry that is slow — or dead, and answering only after its
//! connect gives up — costs the operator nothing but its own block. That is the
//! searcher's half of the isolation the [`asker`](crate::wire::asker) gets from
//! a thread per channel; here one thread is enough, because a fan-out has an
//! order and can publish inside it.
//!
//! The other two seats need none of this. `yog gesture` and the deposit consumer
//! are already off-frame, and a process that asked has nothing else to do, so
//! they run [`run`](super::run) straight through
//! [`answer`](crate::boundary::answer::answer). One engine, three seats.

use super::Found;
use crate::boundary::reply::Reply;
use crate::boundary::{Gesture, Query, codec};
use crate::state::SearchCell;
use crate::wire::dial::Dial;
use crate::wire::link::Landed;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the searcher looks for an ask. A latency knob only: an ask waits,
/// it never rots — the same reading the §8.5 consumer poll gets.
const SEARCH_POLL: Duration = Duration::from_millis(50);

/// One window's searcher: its seats on every channel the window holds, and the
/// cell it is asked through. Built by
/// [`Engine::searcher`](crate::engine::Engine::searcher) so the model never
/// spawns its own thread — a test drives [`pass`](Self::pass) by hand, which is
/// the same reason `Engine::asker` hands back a value.
pub struct Searcher {
    dial: Dial,
    asks: SearchCell,
}

impl Searcher {
    pub(crate) fn new(dial: Dial, asks: SearchCell) -> Self {
        Self { dial, asks }
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
        // The answer knows its own question from the ask rather than from a
        // reply (bl-648a), which is what lets a channel that refused still be
        // an answer to the search the operator typed.
        let mut union = Found {
            needle: text.trim().to_owned(),
            ..Found::default()
        };
        self.dial.fanned(&question, &mut |landed| {
            merge(&mut union, landed);
            self.asks.publish(seq, union.clone());
            self.asks.seq() == seq
        });
        self.asks.seq() == seq
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

/// **One channel's answer folded into the union.** Three arms and no fourth,
/// exactly as before the fan-out: the answer, the channel's refusal carried as
/// an unreadable source (already named with the channel that gave it), and a
/// reply of another kind — a codec that has drifted from the query it answers,
/// which is a defect rather than a state, so nothing is invented for it.
fn merge(union: &mut Found, landed: Landed) {
    match landed {
        Ok(Reply::Search(found)) => {
            union.hits.extend(found.hits);
            union.unreadable.extend(found.unreadable);
        }
        Ok(_) => {}
        Err(said) => union.unreadable.push(said),
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
