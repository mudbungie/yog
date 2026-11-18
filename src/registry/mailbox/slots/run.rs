//! **One run's text, as it arrives** (REMOTE §5.7, bl-d414): what a slot holds
//! between a foot's posts and its asker's reads.
//!
//! Split from the map at §12's per-file budget, on the seam the streamed form
//! drew: [`super`] decides *who* may post and read and *when* a slot is over,
//! and this is the text itself — appended in order, read from an offset,
//! dropped once an asker has passed it, bounded while nobody has.
//!
//! **A capture is the run absorbed.** One post carrying a capture and N posts
//! ending in one leave the same thing here, which is why the streamed form is
//! not a second form: the old single post is this file with N = 1.

use super::super::Capture;
use super::super::stream::{Offsets, Piece, Post, Standing};

/// **How long a slot is listened to, and how long a run may say nothing**:
/// thirty seconds, both. A post for a slot nobody polled in that long is
/// refused, and a run that posted and then went that long without a post is
/// answered in doubt — one figure, because each is the other end's silence.
pub(super) const LISTEN_SECONDS: i64 = 30;

/// **What one stream may hold unread**: the foot's old per-stream capture
/// limit (a sixteenth of a frame), which now bounds a post there and the
/// backlog here.
pub(super) const UNREAD_LIMIT: usize = crate::wire::frame::MAX_FRAME / 16;

/// One stream: the text an asker has not acknowledged, where it starts in the
/// run's own byte count, and how much the bound refused to keep.
#[derive(Debug, Clone, PartialEq, Default)]
struct Held {
    base: u64,
    text: String,
    dropped: usize,
}

impl Held {
    /// Append what fits, on a character boundary, and count the rest.
    fn push(&mut self, piece: &str) {
        let mut keep = piece
            .len()
            .min(UNREAD_LIMIT.saturating_sub(self.text.len()));
        while !piece.is_char_boundary(keep) {
            keep -= 1;
        }
        self.text.push_str(piece.get(..keep).unwrap_or_default());
        self.dropped += piece.len() - keep;
    }

    /// The text from the asker's own count on, **dropping what it has passed**
    /// — the offset is the acknowledgement. `None` for a count this stream
    /// cannot answer from: before what was already dropped, past what has
    /// arrived, or inside a character.
    fn from(&mut self, at: u64) -> Option<String> {
        let skip = usize::try_from(at.checked_sub(self.base)?).ok()?;
        self.text = self.text.get(skip..)?.to_owned();
        self.base = at;
        Some(self.text.clone())
    }

    /// Every byte this stream was ever handed and kept.
    fn arrived(&self) -> u64 {
        self.base + self.text.len() as u64
    }

    /// The sentence naming what the bound cost this stream, in the foot's own
    /// words for the same loss — and nothing when it cost nothing.
    fn elided(&self, name: &str) -> String {
        if self.dropped == 0 {
            return String::new();
        }
        format!(
            "\nyog: {name} exceeded this server's {UNREAD_LIMIT}-byte capture limit; \
             {} further bytes were dropped\n",
            self.dropped
        )
    }
}

/// One run: its two streams, when its machine last posted, and its verdict.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct Run {
    out: Held,
    err: Held,
    /// When a post last landed. `Some` **is the acknowledgement**: a machine
    /// received this slot, so it is never handed out again.
    posted: Option<i64>,
    /// The verdict. `Some` is *over* — by the foot's capture or by a doubt.
    exit: Option<i32>,
}

impl Run {
    pub(super) fn over(&self) -> bool {
        self.exit.is_some()
    }

    pub(super) fn acknowledged(&self) -> bool {
        self.posted.is_some()
    }

    /// Acknowledged, not over, and silent past the bound at `now`.
    pub(super) fn silent(&self, now: i64) -> bool {
        !self.over() && self.posted.is_some_and(|last| now - last > LISTEN_SECONDS)
    }

    /// Bytes arrived per stream — what a doubt names.
    pub(super) fn arrived(&self) -> Offsets {
        Offsets {
            stdout: self.out.arrived(),
            stderr: self.err.arrived(),
        }
    }

    /// **Absorb one post, in order.** `mark` is the redelivery sentence
    /// (REMOTE §5.6) and goes in front of the first text this run ever holds;
    /// a post carrying a capture ends the run.
    pub(super) fn absorb(&mut self, now: i64, mark: Option<String>, post: &Post) {
        let (stdout, stderr, exit) = match post {
            Post::Partial(piece) => (&piece.stdout, &piece.stderr, None),
            Post::Capture(capture) => (&capture.stdout, &capture.stderr, Some(capture.exit_code)),
        };
        if let Some(mark) = mark.filter(|_| self.posted.is_none()) {
            self.err.push(&mark);
            if exit.is_none() || !stderr.is_empty() {
                self.err.push("\n");
            }
        }
        self.out.push(stdout);
        self.err.push(stderr);
        self.posted = Some(now);
        if let Some(code) = exit {
            self.end(code);
        }
    }

    /// **The server ends the run itself**, in doubt: non-zero, what arrived
    /// kept, and `sentence` on stderr behind it.
    pub(super) fn doubt(&mut self, sentence: &str) {
        if !self.err.text.is_empty() {
            self.err.text.push('\n');
        }
        self.err.text.push_str(sentence);
        self.end(1);
    }

    /// Over: the verdict, and what the bound dropped said once, outside it.
    fn end(&mut self, code: i32) {
        let notes = format!("{}{}", self.out.elided("stdout"), self.err.elided("stderr"));
        self.err.text.push_str(&notes);
        self.exit = Some(code);
    }

    /// **The slot as its foot is answered it**: the capture as held once the
    /// run is over, and the bare lease fact while it is not.
    pub(super) fn receipt(&self) -> Standing {
        match self.exit {
            Some(_) => self.whole(),
            None => Standing {
                partial: Some(Piece::default()),
                capture: None,
            },
        }
    }

    /// **The slot as its asker is answered it.** Without `since`, nothing
    /// until the end and then everything held — the read as it always was.
    /// With it, the text from those offsets: `partial` while a posting
    /// machine runs it, `capture` once over. `None` is an offset no stream
    /// here can answer from.
    pub(super) fn answer(&mut self, since: Option<Offsets>) -> Option<Standing> {
        let Some(at) = since else {
            return Some(self.whole());
        };
        let piece = Piece {
            stdout: self.out.from(at.stdout)?,
            stderr: self.err.from(at.stderr)?,
        };
        Some(match self.exit {
            Some(exit_code) => Standing {
                partial: None,
                capture: Some(Capture {
                    stdout: piece.stdout,
                    stderr: piece.stderr,
                    exit_code,
                }),
            },
            None => Standing {
                partial: self.posted.map(|_| piece),
                capture: None,
            },
        })
    }

    /// Everything held, under the verdict — absent while there is none.
    fn whole(&self) -> Standing {
        Standing {
            partial: None,
            capture: self.exit.map(|exit_code| Capture {
                stdout: self.out.text.clone(),
                stderr: self.err.text.clone(),
                exit_code,
            }),
        }
    }
}
