//! **The lane's hand-off, with no wire in it** (REMOTE §3; bl-73e7) — the two
//! channels the frame and the lane meet over, split from
//! [`lane`](super) at §12's per-file budget on the seam the module doc
//! already draws: *"two channels, and deliberately no lock … nothing here is
//! shared mutable state, it is a hand-off in each direction."*
//!
//! Everything in this file is that hand-off and nothing else — no socket, no
//! seat, no thread. What crosses is the **whole** fold per frame, never a
//! delta, so a frame the lane misses costs nothing and a seat never reassembles
//! anything; and **the question is its own key**, as [`link`](crate::wire::link)'s
//! is, so a subject that moved cannot land the previous conversation's tail on
//! the new one.

use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

use serde_json::Value;

use crate::git_tree::Stream;

/// The frame's end of the lane: the conversation it wants followed, and the
/// newest fold that has arrived for it.
pub struct Tail {
    subject: Sender<Option<Value>>,
    frames: Receiver<(String, Option<Stream>)>,
    /// Declared during this frame's render, read at the next settle.
    standing: Option<Value>,
    /// What the lane is on.
    asked: Option<Value>,
    landed: Option<Stream>,
}

/// The lane's end: what to follow, and where a frame goes.
pub struct TailEnd {
    subject: Receiver<Option<Value>>,
    frames: Sender<(String, Option<Stream>)>,
    standing: Option<Value>,
    /// Whether the frame end has gone away. A lane that kept re-asking for a
    /// window nobody is painting would be the one leak a detached thread can
    /// make, so the disconnect is latched rather than re-derived.
    hung_up: bool,
}

/// A fresh pair, minted together for [`link::pair`](crate::wire::link::pair)'s
/// reason: neither end is useful alone.
pub fn pair() -> (Tail, TailEnd) {
    let (s_tx, s_rx) = channel();
    let (f_tx, f_rx) = channel();
    (
        Tail {
            subject: s_tx,
            frames: f_rx,
            standing: None,
            asked: None,
            landed: None,
        },
        TailEnd {
            subject: s_rx,
            frames: f_tx,
            standing: None,
            hung_up: false,
        },
    )
}

/// **A lane nobody answers.** The model holds one from the moment it boots, so
/// a seat's read of the tail is the same call whether or not this box got a
/// lane up — and a window with none simply paints the pull fold.
impl Default for Tail {
    fn default() -> Self {
        pair().0
    }
}

impl Tail {
    /// Declare `question` the followed subject and read whatever fold has
    /// landed for it. Called during render, so it does one clone and nothing
    /// else — never blocks and never dials.
    pub fn ask(&mut self, question: &Value) -> Option<Stream> {
        self.standing = Some(question.clone());
        self.landed.clone()
    }

    /// One frame's whole duty: take what landed, tell the lane what is followed
    /// if that changed, and start the next frame's declaration empty. A subject
    /// nobody declared this frame is a subject nobody is watching, which is the
    /// whole of stopping — there is no unfollow to forget.
    pub fn settle(&mut self) {
        let key = self.asked.as_ref().map(Value::to_string);
        for (answered, frame) in self.frames.try_iter() {
            if Some(&answered) == key.as_ref() {
                self.landed = frame;
            }
        }
        if self.standing != self.asked {
            let _ = self.subject.send(self.standing.clone());
            self.asked = self.standing.clone();
            self.landed = None;
        }
        self.standing = None;
    }
}

impl TailEnd {
    /// The subject as of now — the newest declaration the frame sent, or the one
    /// before it when the frame has declared nothing new. `None` once the frame
    /// end is gone, which is what ends a detached lane.
    pub fn standing(&mut self) -> Option<Value> {
        loop {
            match self.subject.try_recv() {
                Ok(newest) => self.standing = newest,
                Err(TryRecvError::Empty) => return self.standing.clone(),
                Err(TryRecvError::Disconnected) => {
                    self.hung_up = true;
                    return None;
                }
            }
        }
    }

    /// Publish one frame against the question it answers. `None` is *this
    /// stream is over* — the seat drops the tail and falls back to the pull
    /// fold, which is also what makes the step boundary a swap rather than a
    /// duplication.
    pub fn publish(&self, question: &str, frame: Option<Stream>) {
        let _ = self.frames.send((question.to_owned(), frame));
    }
}
