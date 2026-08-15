//! **The shell's one spelling of a wire act** (REMOTE §1.2, §9.8; bl-4841) —
//! the write-side twin of [`super::wire`].
//!
//! Since the 2026-08-14 ruling the window is a client of its own engine over
//! loopback mTLS, and a frame may never wait on a socket. So a click no longer
//! *runs* a gesture and reads its `Reply` in the same frame: it **posts** one and
//! holds a [`Ticket`], and the receipt lands frames later
//! ([`AppModel::post_act`]).
//!
//! Two shapes, because acts come in two kinds and no more:
//!
//! - **[`fire`] — the act whose receipt is nothing.** Every §8.2 verb whose
//!   durable record is its own `ops.jsonl` line (INV-2): the ball verbs, the
//!   `lernie` short verbs, a fork's cohort. Nobody held the reply before either
//!   — it was discarded on the spot — so nobody holds a ticket now. The
//!   re-derivation the fire used to trigger is not lost: it moved to the receipt,
//!   where it belongs, and the model runs it for every act
//!   ([`AppModel::settle_acts`]).
//! - **[`Held`] — the act whose receipt is a sentence.** The four surfaces that
//!   paint what came back: the marks pane, the model picker's two writes, the
//!   lineage config editor. Each already held a status string across frames, so
//!   what it gains is one field beside it — the ticket — and the in-flight
//!   state is that same line saying so.
//!
//! **The `Cli` pair does not come here, and that is the point.** A dispatched
//! act needed `boundary_deps`, which carries the verb binaries this box
//! resolved; a posted one carries nothing but the gesture, because the engine
//! owns the binaries and a seat never did. A remote seat could fire every act
//! in this file.
//!
//! Coverage-excluded glue like the rest of `src/shell/*`: the posting, the
//! ticket and the receipt are covered where they live (`app::acts`,
//! `wire::post`).

use crate::AppModel;
use crate::boundary::{Action, reply::Reply};
use crate::wire::post::Ticket;

/// What marks a line whose act has not been answered yet. An **ellipsis on the
/// sentence the click already wrote**, rather than a second phrasing to learn:
/// the operator reads what this gesture means, with one mark saying the engine
/// has not confirmed it. A clean receipt drops the mark and nothing else moves;
/// anything else appends the reason.
const IN_FLIGHT: &str = " …";

/// Fire one act nobody is holding a receipt for (§8.2, INV-2): the ops trail is
/// the durable record, and a refusal reaches the operator as the §7.3 banner
/// reads that trail back.
pub(super) fn fire(model: &mut AppModel, action: &Action) {
    let _ = model.post_act(action);
}

/// One act a surface is holding: the ticket it was posted under, and the
/// sentence its landing means.
#[derive(Default)]
pub(super) struct Held {
    ticket: Option<Ticket>,
    said: String,
}

impl Held {
    /// Post `action`, and stand on `said` — **what a clean landing means**,
    /// written at the click because that is where it is known. A second fire
    /// while one is outstanding replaces the ticket: the earlier act still
    /// happens, an act being never unsent, and it is the newer one's answer
    /// this surface is now waiting on.
    pub(super) fn fire(&mut self, model: &mut AppModel, action: &Action, said: &str) {
        self.ticket = Some(model.post_act(action));
        said.clone_into(&mut self.said);
    }

    /// Take the receipt if it has landed — once. The surface folds it, because
    /// it is the only place that knows what this act's answer means.
    pub(super) fn landed(&mut self, model: &mut AppModel) -> Option<Result<Reply, String>> {
        let ticket = self.ticket?;
        let landed = model.act_receipt(ticket)?;
        self.ticket = None;
        Some(landed)
    }

    /// Replace the line — what a fold says when the receipt was not the clean
    /// landing the fire assumed.
    pub(super) fn say(&mut self, said: String) {
        self.said = said;
    }

    /// The line to paint: the fire's own sentence, marked while the engine has
    /// not answered.
    pub(super) fn line(&self) -> String {
        match self.ticket {
            Some(_) => format!("{}{IN_FLIGHT}", self.said),
            None => self.said.clone(),
        }
    }

    /// Whether there is anything to paint at all.
    pub(super) fn quiet(&self) -> bool {
        self.said.is_empty()
    }

    /// Forget the last act and its line — what a surface that is being reopened
    /// on a new subject does, the previous answer being about the previous one.
    pub(super) fn forget(&mut self) {
        *self = Self::default();
    }
}

/// **What a receipt says went wrong, if anything** — the arm three of the four
/// held surfaces share, so a refusal reads the same wherever it landed.
///
/// `None` is a clean landing, and a clean landing means the sentence the fire
/// already wrote. A non-zero exit is spelled through the one projection
/// (bl-afa9): a bare `-1` reads as a signal death rather than "ran, status not
/// observable".
pub(super) fn trouble(landed: &Result<Reply, String>) -> Option<String> {
    match landed {
        Err(said) => Some(said.clone()),
        Ok(Reply::Outcome(outcome)) if !outcome.ok() => Some(format!(
            "{} · {}",
            crate::opslog::exit::ExitKind::of(outcome.exit, "lernie").label(),
            outcome.stderr.trim()
        )),
        Ok(_) => None,
    }
}
