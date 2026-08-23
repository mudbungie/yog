//! **The follow lane, stood in for** (REMOTE §3, bl-73e7): one look at the
//! world per frame where a real window holds a socket open.
//!
//! Split off [`super`] at §12's per-file budget. The seam is real rather than a
//! line budget: the reads and the acts here answer a question and are done,
//! while this one carries a **held** read across frames — the `Follow` it mints
//! is state the other answerers have none of, and it is exactly the state a
//! connection is.

use super::world::World;
use crate::boundary::{Gesture, codec};

impl World {
    /// **Answer the follow lane**, the way the [`Lane`](crate::wire::lane::Lane)
    /// thread does — one look at the world per frame instead of a held socket
    /// (bl-73e7). The read itself is the engine's own `boundary::follow`, so
    /// what a beat paints came off the same incremental fold a real connection
    /// would have carried; only the socket is stood in for, exactly as it is
    /// for the asker and the poster.
    ///
    /// **It runs no derivation and asks the standing set for nothing**, which
    /// is the whole claim a paint beat about this can make: bytes appended to
    /// an open step file reach the glass with nothing else having run.
    pub(in crate::shell::acceptance) fn follows(&mut self) {
        use crate::boundary::follow::{Follow, Frame};
        let Some(question) = self.tail.standing() else {
            self.followed = None;
            return;
        };
        let key = question.to_string();
        if self.followed.as_ref().map(|(at, _)| at.as_str()) != Some(key.as_str()) {
            let Ok(Gesture::Ask(crate::boundary::Query::Follow { workspace, agent })) =
                codec::decode(&question)
            else {
                return;
            };
            let Ok(ws) = self.model.derivation().ws_path(&workspace) else {
                return;
            };
            self.followed = Some((
                key.clone(),
                Follow::new(self.model.snapshot_cell(), ws, agent),
            ));
        }
        let Some((_, follow)) = self.followed.as_mut() else {
            return;
        };
        match follow.poll() {
            Frame::Ready(stream) => self.tail.publish(&key, Some(stream)),
            Frame::Over => {
                self.tail.publish(&key, None);
                self.followed = None;
            }
            Frame::Waiting => {}
        }
    }
}
