//! The follower itself: the poll, the offset, and the thread that drives it.
//!
//! Split from [`super`] at §12's per-file budget, on the seam between *what the
//! live tail is* (the value, the fold onto the painted snapshot, the model's
//! side of the hand-off) and *how the bytes are gathered*. Everything in this
//! file is one open file and an integer.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use super::LiveTail;
use crate::git_tree::{Stream, fold_stream, latest_response_path};
use crate::state::TailCell;
use crate::watch::Repaint;

/// How often the follower looks for appended bytes. A latency knob, not a
/// correctness one — the derivation still folds the same file on its own
/// cadence, so a missed pass costs freshness and never truth. Half the §11
/// pulse period, so a repaint the pulse was going to make anyway carries the
/// newest bytes rather than the previous pass's.
const FOLLOW_POLL: Duration = Duration::from_millis(16);

/// The response file being followed: where it is, how far in this follower has
/// read, and the trailing bytes that were not a whole line yet.
struct Open {
    path: PathBuf,
    offset: u64,
    partial: Vec<u8>,
}

/// The follower. Built by [`AppModel::follower`](crate::AppModel::follower) so
/// the model never spawns its own thread — a test drives [`pass`](Self::pass)
/// by hand, the same reason `boot` hands back a `Deriver`.
pub struct Follower {
    cell: TailCell,
    /// The subject this follower is currently open on, and its accumulated
    /// fold. `None` before the frame has asked for anything, and again
    /// whenever the ask names a conversation with no step file yet.
    at: Option<(LiveTail, Open)>,
}

impl Follower {
    pub(crate) fn new(cell: TailCell) -> Self {
        Self { cell, at: None }
    }

    /// One pass: follow the frame's current ask and publish if anything moved.
    /// Returns whether the published tail changed — which is exactly when the
    /// face has something new to paint.
    pub fn pass(&mut self) -> bool {
        let Some((ws, agent)) = crate::state::asked(&self.cell) else {
            return self.clear();
        };
        self.reseat(&ws, &agent);
        let Some((tail, open)) = self.at.as_mut() else {
            return self.clear();
        };
        let Some(appended) = open.read_appended() else {
            return false;
        };
        tail.stream.absorb(appended);
        // Bytes moving is not the same as the tail moving: a `message_start`
        // or a tool-argument delta advances the offset and says nothing the
        // operator can see. The cell decides, so a frame is asked for only
        // when there is something new on it.
        crate::state::publish_tail(&self.cell, Some(tail.clone()))
    }

    /// Drop whatever was held and say whether that was a change. The one exit
    /// for every "there is nothing to follow" — no subject, no step file — so
    /// an unfocused frame and a conversation that has not opened a step read
    /// the same, which is the general path with empty input.
    fn clear(&mut self) -> bool {
        self.at = None;
        crate::state::publish_tail(&self.cell, None)
    }

    /// Point at `(ws, agent)`'s newest step file, starting the accumulator over
    /// if this is a different stream than the one held (see the module doc's
    /// one reset rule). A subject whose newest step file is unchanged keeps its
    /// offset and its text.
    fn reseat(&mut self, ws: &Path, agent: &str) {
        let path = latest_response_path(ws, agent);
        let same = match (&self.at, &path) {
            (Some((tail, open)), Some(path)) => {
                tail.ws == ws && tail.agent == agent && open.path == *path
            }
            _ => false,
        };
        if same {
            return;
        }
        self.at = path.map(|path| {
            (
                LiveTail {
                    ws: ws.to_path_buf(),
                    agent: agent.to_owned(),
                    stream: Stream::default(),
                },
                Open {
                    path,
                    offset: 0,
                    partial: Vec::new(),
                },
            )
        });
    }

    /// Run [`pass`](Self::pass) forever, waking the face whenever bytes landed
    /// — the derivation worker's shutdown shape exactly: a stop flag, an
    /// unpark, a join.
    pub fn spawn(mut self, repaint: impl Repaint + 'static) -> FollowThread {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                if self.pass() {
                    repaint.request();
                }
                std::thread::park_timeout(FOLLOW_POLL);
            }
        });
        FollowThread {
            stop,
            handle: Some(handle),
        }
    }
}

impl Open {
    /// The fold of the bytes appended since the last pass, or `None` when
    /// nothing whole arrived. A file shorter than the offset was truncated or
    /// replaced, so the read restarts from zero — the accumulated text is then
    /// wrong by exactly the bytes that vanished, and the next derivation is
    /// what corrects it.
    fn read_appended(&mut self) -> Option<Stream> {
        let len = std::fs::metadata(&self.path).ok()?.len();
        if len < self.offset {
            self.offset = 0;
            self.partial.clear();
        }
        if len == self.offset {
            return None;
        }
        let fresh = read_from(&self.path, self.offset)?;
        self.offset = len.min(self.offset + fresh.len() as u64);
        self.partial.extend_from_slice(&fresh);
        // Fold whole lines only; the tail of the buffer may be a line the
        // harness is still writing (§4.4 partial-write tolerance).
        let end = self.partial.iter().rposition(|&b| b == b'\n')? + 1;
        let folded = fold_stream(self.partial.get(..end)?);
        self.partial.drain(..end);
        Some(folded)
    }
}

/// The appended bytes from `offset` to end of file. Its own function because
/// the seek-and-read is the one place this module touches an fd, and a
/// half-open file (the step dir mid-creation) has to read as "nothing yet"
/// rather than as an error path with an opinion.
fn read_from(path: &Path, offset: u64) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path).ok()?;
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut fresh = Vec::new();
    file.read_to_end(&mut fresh).ok()?;
    Some(fresh)
}

/// The follower thread's handle; [`Drop`] signals stop, unparks and joins.
pub struct FollowThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for FollowThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}
