//! Tri-state liveness probes (ARCH §3.5 / DESIGN §10 portability).
//!
//! The classifier ([`super::state::classify`]) asks two liveness questions —
//! "is anyone driving this agent?" (the executor lock, §2.11) and "is a model
//! call in flight right now?" (the `response.json` writer, §3.5/§4.4) — and
//! each answer is an *observation* that can fail. The traits live here (not
//! beside their platform impls) because they are the one contract every
//! backend implements; injecting them keeps the classifier testable without a
//! live driver (DESIGN §10: "the trait-injection pattern is the template for
//! every new effect").
//!
//! - The Linux procfs backends ([`super::lock_probe`], [`super::fd_probe`])
//!   scan `/proc`, which is always present (DESIGN §10), so they answer a
//!   definite [`Probe::Held`] or [`Probe::Free`] and **never**
//!   [`Probe::Unknown`].
//! - The macOS `lsof` backend (DESIGN §10) can fail to run; when it does it
//!   answers [`Probe::Unknown`] and the classifier degrades to a
//!   framing-only reading carrying an uncertainty flag, rather than reporting
//!   a false definite state.

use std::path::Path;

/// The result of a single liveness observation. `Held` and `Free` are
/// definite; `Unknown` means the backend could not observe (e.g. `lsof`
/// missing/failing on macOS, DESIGN §10) and the classifier must not invent a
/// definite state — it degrades to framing-only and flags uncertainty.
///
/// Public (re-exported alongside [`super::AgentState`]) because it is the
/// classification module's second output vocabulary and the contract every
/// probe backend returns; `Unknown`'s producer is the macOS `lsof` backend
/// (DESIGN §10 / ball Y10 `lsof.rs`), which lands separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// A process holds the observed resource (the lock fd / the writer fd).
    Held,
    /// No process holds it — a definite negative.
    Free,
    /// The observation could not be made; degrade to framing-only (§10).
    Unknown,
}

/// "Is anyone driving `<agent>`?" — does a process hold the agent's
/// inbox-directory fd open (the executor lock, §2.11)? Reports the lock
/// *observation*, so a backend that cannot look answers [`Probe::Unknown`].
pub(super) trait LockProbe {
    fn lock_state(&self, inbox_dir: &Path) -> Probe;
}

/// "Is a model call in flight right now?" — does a process hold `path` open
/// for *write* (the `response.json` fd, §3.5/§4.4)? Reports the writer
/// *observation*, so a backend that cannot look answers [`Probe::Unknown`].
pub(super) trait WriterProbe {
    fn writer_state(&self, path: &Path) -> Probe;
}
