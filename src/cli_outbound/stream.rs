//! The live-handle half of [`super`]: [`Stream`], the running-subprocess handle
//! whose iteration yields [`Chunk`]s (final item always `Exited`) and whose drop
//! terminates the child (SIGTERM, then SIGKILL after a short grace, §2.9). Split
//! from [`super`] so the spawn half stays under the 300-line cap; the two meet
//! only at [`Stream::new`], which [`super::Cli`]'s spawn calls once per launch.

use super::{Chunk, ExitInfo};
use std::process::Child;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

const TERM_GRACE: Duration = Duration::from_millis(500);

/// One non-blocking read off a live [`Stream`] (the streamed-piped class, §8):
/// a buffered [`Chunk`], or [`Pending`](StreamPoll::Pending) when the child
/// still runs but nothing is ready yet. The terminal [`Chunk::Exited`] rides a
/// `Ready` exactly once (see [`Stream::try_next`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamPoll {
    Ready(Chunk),
    Pending,
}

/// Live handle to a running subprocess. Iterate to consume chunks;
/// `Exited` is always the final item. Drop to terminate early.
pub struct Stream {
    child: Option<Child>,
    rx: Receiver<Chunk>,
    exit_emitted: bool,
}

impl Stream {
    /// Wrap a freshly-spawned child and its pumped-chunk receiver — the one seam
    /// [`super::Cli`]'s spawn half constructs a [`Stream`] through (the pump
    /// threads already send on the paired sender).
    pub(super) fn new(child: Child, rx: Receiver<Chunk>) -> Self {
        Self {
            child: Some(child),
            rx,
            exit_emitted: false,
        }
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(std::process::Child::id)
    }

    /// Non-blocking peer of the blocking [`Iterator`] (§8 streamed-piped class):
    /// pull one buffered chunk without waiting. [`StreamPoll::Ready`] when one is
    /// available; [`StreamPoll::Pending`] when the child still runs but nothing is
    /// buffered yet; a `Ready(`[`Chunk::Exited`]`)` **exactly once** when both
    /// pump senders have dropped (the channel disconnected), after which it stays
    /// `Pending`. The egui frame loop calls this each frame so a device-code line
    /// paints the moment it lands without ever blocking the UI.
    pub fn try_next(&mut self) -> StreamPoll {
        if self.exit_emitted {
            return StreamPoll::Pending;
        }
        match self.rx.try_recv() {
            Ok(chunk) => StreamPoll::Ready(chunk),
            Err(TryRecvError::Empty) => StreamPoll::Pending,
            Err(TryRecvError::Disconnected) => {
                self.exit_emitted = true;
                let status = self.child.take().and_then(|mut c| c.wait().ok());
                StreamPoll::Ready(Chunk::Exited(exit_info(status)))
            }
        }
    }

    /// Build a [`Stream`] from a bare receiver with **no** child — the seam the
    /// streamed-class unit tests drive [`try_next`](Self::try_next) and
    /// [`Streamed`](super::Streamed) through deterministically, sending chunks by
    /// hand instead of racing a real process. Child-less, so [`Drop`] is a no-op.
    #[cfg(test)]
    pub(super) fn from_rx(rx: Receiver<Chunk>) -> Self {
        Self {
            child: None,
            rx,
            exit_emitted: false,
        }
    }
}

impl Iterator for Stream {
    type Item = Chunk;

    fn next(&mut self) -> Option<Chunk> {
        if self.exit_emitted {
            return None;
        }
        if let Ok(chunk) = self.rx.recv() {
            Some(chunk)
        } else {
            self.exit_emitted = true;
            let status = self.child.take().and_then(|mut c| c.wait().ok());
            Some(Chunk::Exited(exit_info(status)))
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        let pid = child.id() as i32;
        super::sys::sigterm(pid);
        let deadline = Instant::now() + TERM_GRACE;
        while Instant::now() < deadline {
            if let Ok(Some(_)) = child.try_wait() {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// `pub(super)` so the streaming tests white-box its `Unknown` arms (a missing
/// or stopped status is unreachable through the public [`Stream`] API).
pub(super) fn exit_info(status: Option<std::process::ExitStatus>) -> ExitInfo {
    use std::os::unix::process::ExitStatusExt;
    status.map_or(ExitInfo::Unknown, |s| match (s.code(), s.signal()) {
        (Some(c), _) => ExitInfo::Code(c),
        (_, Some(sig)) => ExitInfo::Signal(sig),
        _ => ExitInfo::Unknown,
    })
}
