//! **The frame's half of the read path** (REMOTE §1.2, §3; bl-ae05): what the
//! window asks the wire for, and what has come back.
//!
//! The window is a client of its own engine over loopback mTLS, so every read
//! it paints is an answer that travelled a socket — and a socket is exactly the
//! thing a frame may never wait on. This is the shape the §8.5 searcher already
//! proved: **the frame declares a standing question and renders whatever has
//! landed**, exactly as it renders whatever snapshot has landed. There is no
//! frame-side wait, so there is no stutter to measure.
//!
//! **Two channels, and deliberately no lock.** The crate confines `Mutex` to
//! [`state`](crate::state) (AGENTS.md rule 7) and that file is at its own
//! ceiling; more to the point, nothing here is *shared* mutable state — it is a
//! hand-off in each direction, which is what a channel is. The frame owns its
//! own map of what landed and never blocks to read it.
//!
//! **The question is its own key.** A standing question is identified by its
//! encoded envelope's text, so there is no second naming to keep in step and
//! two callers asking the same thing are one ask. The set is re-declared every
//! frame and sent only when it *changes* — one compare per frame — so a
//! question nobody asked last frame stops being asked and its answer is
//! dropped. Nothing has to say "stop".
//!
//! **Cadence is human** ([`asker`](super::asker)). The window is not a machine
//! polling a machine: it re-asks its standing set on a period an operator
//! reads at, which is why REMOTE §10's connection-per-gesture ruling still
//! stands — the criterion there is *"when a seat's ask rate exceeds human
//! cadence"*, and this does not.

use crate::boundary::reply::Reply;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{Receiver, Sender, channel};

/// What one question earned: the typed reply, or the reason there is none.
///
/// **One `Err`, not two.** A refusal the engine sent and a channel that failed
/// are the same fact to a frame — the answer cannot be painted, and here is the
/// sentence — so a reader carries no case for which layer said so.
pub type Landed = Result<Reply, String>;

/// The window's end: the questions it is standing on and the answers it holds.
pub struct Link {
    questions: Sender<Vec<Value>>,
    answers: Receiver<(String, Landed)>,
    standing: BTreeMap<String, Value>,
    asked: BTreeSet<String>,
    landed: BTreeMap<String, Landed>,
}

/// The asker's end: the standing set to ask, and where answers go.
pub struct LinkEnd {
    questions: Receiver<Vec<Value>>,
    answers: Sender<(String, Landed)>,
    standing: Vec<Value>,
}

/// A fresh pair. The two ends are minted together because neither is useful
/// alone — which is also why there is no way to attach one later.
pub fn pair() -> (Link, LinkEnd) {
    let (q_tx, q_rx) = channel();
    let (a_tx, a_rx) = channel();
    (
        Link {
            questions: q_tx,
            answers: a_rx,
            standing: BTreeMap::new(),
            asked: BTreeSet::new(),
            landed: BTreeMap::new(),
        },
        LinkEnd {
            questions: q_rx,
            answers: a_tx,
            standing: Vec::new(),
        },
    )
}

/// **A link nobody answers.** The model holds one from the moment it boots, and
/// the engine hands it a live one when there is a wire to answer over — so a
/// frame's read path is the same call whether or not this box got a listener
/// up, and the surface that got no answer says so rather than branching.
impl Default for Link {
    fn default() -> Self {
        pair().0
    }
}

impl Link {
    /// Declare `question` standing and read whatever has landed for it. Called
    /// during render, so it does exactly one map insert and one map read.
    pub fn ask(&mut self, question: &Value) -> Option<Landed> {
        let key = question.to_string();
        let landed = self.landed.get(&key).cloned();
        self.standing.insert(key, question.clone());
        landed
    }

    /// One frame's whole duty: take what landed, tell the asker what is
    /// standing if that changed, and start the next frame's declaration empty.
    /// Answers for questions no longer standing are dropped here, which is the
    /// whole of forgetting.
    pub fn settle(&mut self) {
        for (key, answer) in self.answers.try_iter() {
            self.landed.insert(key, answer);
        }
        let wanted: BTreeSet<String> = self.standing.keys().cloned().collect();
        if wanted != self.asked {
            let _ = self
                .questions
                .send(self.standing.values().cloned().collect());
            self.asked = wanted;
            self.landed.retain(|key, _| self.asked.contains(key));
        }
        self.standing.clear();
    }
}

impl LinkEnd {
    /// The standing set as of now — the newest declaration the frame sent, or
    /// the one before it when the frame has declared nothing new.
    pub fn standing(&mut self) -> Vec<Value> {
        if let Some(newest) = self.questions.try_iter().last() {
            self.standing = newest;
        }
        self.standing.clone()
    }

    /// Publish one answer. A send that fails is a window that has gone away,
    /// and the asker's own loop is what ends then.
    pub fn publish(&self, question: &Value, landed: Landed) -> bool {
        self.answers.send((question.to_string(), landed)).is_ok()
    }
}

#[cfg(test)]
mod tests;
