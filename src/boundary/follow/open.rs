//! The incremental read itself: one open file, a byte offset, and the trailing
//! bytes that were not a whole line yet.
//!
//! It was the §7.2 live-tail follower's (`app::live::follow`, bl-54f7) until
//! bl-73e7 moved it to the engine, where the read finally has a consumer that
//! reaches a seat. Nothing about the mechanism changed: **following, not
//! re-reading** — each pass reads only what was appended, folds the complete
//! lines through the one shared parser
//! ([`fold_stream`](crate::git_tree::fold_stream)) and hands back that fold for
//! [`absorb`](crate::git_tree::Stream::absorb), whose contract
//! (`fold(a).absorb(fold(b)) == fold(a ++ b)` on any line boundary) is what
//! makes the incremental read and a whole-file read one description.
//!
//! Re-reading and re-folding the whole response on every look is the naive
//! shape and it degrades as the answer grows — which is the difference between
//! a lane that costs the new bytes and one that costs the conversation, sixty
//! times a second.
//!
//! Partial-write tolerance is structural twice over: the remainder before the
//! last newline is held back, and the parser skips a line it cannot read.

use std::path::{Path, PathBuf};

use crate::git_tree::{Stream, fold_stream};

/// The response file being followed, and how far into it this reader has come.
pub(super) struct Open {
    /// Which file — the identity of the stream, so a step advancing is visible
    /// as this value changing rather than as anything remembered.
    pub(super) path: PathBuf,
    offset: u64,
    partial: Vec<u8>,
}

impl Open {
    /// Start at the beginning of `path`. There is no resume: a reader is minted
    /// per held connection, and a connection that dropped mid-answer re-asks
    /// for the whole tail rather than for a suffix nobody can address.
    pub(super) fn at(path: PathBuf) -> Self {
        Self {
            path,
            offset: 0,
            partial: Vec::new(),
        }
    }

    /// The fold of the bytes appended since the last look, or `None` when
    /// nothing whole arrived. A file shorter than the offset was truncated or
    /// replaced, so the read restarts from zero.
    pub(super) fn read_appended(&mut self) -> Option<Stream> {
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
