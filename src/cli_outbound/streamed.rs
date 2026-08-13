//! The **streamed-piped** spawn class (DESIGN §8, §8.3): both output streams
//! line-buffered off a running child and delivered to the invoking surface as
//! whole lines arrive — each tagged with the stream it came from
//! ([`StreamedLine`]) — plus the terminal exit. This is the shape `bz --login`'s
//! OAuth flow renders through (§5.3 whitelists the live lines as instance-local
//! RAM: a login code is for the human at *this* keyboard), and bz writes that
//! entire flow to **stderr**, so carrying only stdout would render an empty pane.
//!
//! Non-blocking by construction: [`Streamed::poll`] drains only what the pump
//! threads have already buffered and returns at once, so the egui frame loop
//! never stalls waiting on the child. The stream itself is never logged
//! line-by-line — it **converges to one outcome row** at exit (§4.2); appending
//! that row is the caller's job ([`crate::login`]), keeping this crate free of
//! the `opslog` seam (the `actions`/`login` layer bridges the two).

use super::{Chunk, Stream, StreamPoll};

/// Hard cap on a single rendered line's byte length — the streamed class's
/// *bounded line length* (§8). A physical line reaching this cap stops accreting
/// (further bytes drop) and flushes with a [`TRUNC_MARK`] suffix, so a device
/// flow that never emits a newline cannot grow RAM without bound. Device codes
/// and URLs are far shorter; the cap only bites pathological input.
const MAX_LINE: usize = 4096;

/// Suffix stamped on a line truncated at [`MAX_LINE`] (§8), so the truncation is
/// visible in the pane rather than silent.
const TRUNC_MARK: &str = "…[truncated]";

/// A pure newline splitter with a bounded retained line (§8). Bytes accrete into
/// `partial`; each `\n` flushes it as one line. A line that reaches [`MAX_LINE`]
/// stops accreting and flushes with [`TRUNC_MARK`], bounding RAM. Flush is
/// lossy-UTF8 — a device code is ASCII, and a multibyte char torn at a read
/// boundary is a display glyph, never an error.
#[derive(Debug, Default)]
struct LineBuf {
    partial: Vec<u8>,
    truncated: bool,
}

impl LineBuf {
    /// Feed `bytes`, returning every line completed by a `\n` within them, in
    /// order. A trailing unterminated remainder stays buffered for the next feed
    /// (or [`finish`](Self::finish) at EOF).
    fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        for &b in bytes {
            if b == b'\n' {
                lines.push(self.flush());
            } else if self.partial.len() < MAX_LINE {
                self.partial.push(b);
            } else {
                self.truncated = true;
            }
        }
        lines
    }

    /// The buffered remainder as a final line at EOF, or `None` when empty — a
    /// device code printed without a trailing newline still surfaces.
    fn finish(&mut self) -> Option<String> {
        (!self.partial.is_empty()).then(|| self.flush())
    }

    /// Take `partial` as one lossy-UTF8 line, appending [`TRUNC_MARK`] when it was
    /// capped, and reset for the next line.
    fn flush(&mut self) -> String {
        let mut line = String::from_utf8_lossy(&self.partial).into_owned();
        if self.truncated {
            line.push_str(TRUNC_MARK);
        }
        self.partial.clear();
        self.truncated = false;
        line
    }
}

/// One whole line of a streamed child's output, tagged with the stream that
/// produced it (§8). **The tag is why both streams can be live at once:** `bz
/// --login` writes its entire human-facing flow — the device code, the
/// verification URL, and the terminal error/remedy line — to *stderr* (stdout
/// is reserved for its machine-readable discovery output), while `bz
/// --list-models --json` writes its payload to *stdout*. A class that carried
/// only stdout showed the login pane nothing at all. Carrying both, tagged,
/// lets the parsing consumer filter and the rendering consumer paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamedLine {
    pub text: String,
    /// `true` when the line came from the child's **stderr**.
    pub err: bool,
}

/// The joined text of every stdout line, newline-separated — what a consumer
/// that *parses* the child's payload reads (`--list-models`' JSON).
pub fn stdout_text(lines: &[StreamedLine]) -> String {
    text_of(lines, false)
}

/// The joined text of every stderr line — the `ops.jsonl` outcome row's stderr
/// field (§4.2) and a failure's rendered reason. Single-sourced from the same
/// tagged lines the surface paints, so the log and the pane can never name
/// different text.
pub fn stderr_text(lines: &[StreamedLine]) -> String {
    text_of(lines, true)
}

fn text_of(lines: &[StreamedLine], err: bool) -> String {
    lines
        .iter()
        .filter(|l| l.err == err)
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Tag a line buffer's flush with the stream it came from.
fn tag(texts: Vec<String>, err: bool) -> Vec<StreamedLine> {
    texts
        .into_iter()
        .map(|text| StreamedLine { text, err })
        .collect()
}

/// One non-blocking read of a [`Streamed`] (§8): new whole lines arrived, nothing
/// is ready yet, or the child exited. The terminal [`Done`](StreamedPoll::Done)
/// carries any final flushed lines and the exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamedPoll {
    Lines(Vec<StreamedLine>),
    Pending,
    Done(StreamedOutcome),
}

/// The terminal fact of a streamed run (§4.2 outcome row): the shell-convention
/// exit code and any final unterminated lines flushed at EOF. There is no
/// separate stderr field — the run's stderr *is* its stderr-tagged lines
/// ([`stderr_text`]), held once by the consumer that already accumulates them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamedOutcome {
    pub lines: Vec<StreamedLine>,
    pub exit: i32,
}

/// A running child consumed as the streamed-piped class (§8): **both** output
/// streams line-buffered live and tagged, exit captured. Wraps a [`Stream`]
/// whose Drop still SIGTERM/SIGKILLs the child — closing the login surface
/// aborts the flow, consistent with its instance-local nature (§5.3).
pub struct Streamed {
    stream: Stream,
    out: LineBuf,
    err: LineBuf,
}

impl Streamed {
    /// Wrap a freshly-spawned [`Stream`] (stdin already null via
    /// [`Cli::run`](super::Cli::run)) for line-buffered live consumption.
    pub fn new(stream: Stream) -> Self {
        Self {
            stream,
            out: LineBuf::default(),
            err: LineBuf::default(),
        }
    }

    /// Non-blocking: drain whatever the pumps have buffered right now, then return.
    /// New whole lines from either stream come back as [`StreamedPoll::Lines`];
    /// nothing-yet as [`Pending`](StreamedPoll::Pending); the child's exit as
    /// [`Done`](StreamedPoll::Done) with any final flushed lines + exit. Idempotent
    /// after `Done` (the underlying [`Stream::try_next`] stays `Pending`), so a
    /// stray extra poll yields `Pending`, never a second `Done`.
    pub fn poll(&mut self) -> StreamedPoll {
        let mut lines = Vec::new();
        loop {
            match self.stream.try_next() {
                StreamPoll::Ready(Chunk::Stdout(b)) => lines.extend(tag(self.out.push(&b), false)),
                StreamPoll::Ready(Chunk::Stderr(b)) => lines.extend(tag(self.err.push(&b), true)),
                StreamPoll::Ready(Chunk::Exited(e)) => {
                    lines.extend(tag(self.out.finish().into_iter().collect(), false));
                    lines.extend(tag(self.err.finish().into_iter().collect(), true));
                    return StreamedPoll::Done(StreamedOutcome {
                        lines,
                        exit: e.shell_code(),
                    });
                }
                StreamPoll::Pending => {
                    return if lines.is_empty() {
                        StreamedPoll::Pending
                    } else {
                        StreamedPoll::Lines(lines)
                    };
                }
            }
        }
    }

    /// Build a [`Streamed`] over a bare receiver with no child — the seam the
    /// streamed-class and login unit tests drive [`poll`](Self::poll) through
    /// deterministically (chunks sent by hand, no process to race). `pub(crate)`
    /// so [`crate::login`]'s tests reach it; test-only, so it costs no coverage.
    #[cfg(test)]
    pub(crate) fn from_rx(rx: std::sync::mpsc::Receiver<Chunk>) -> Self {
        Self::new(Stream::from_rx(rx))
    }
}
