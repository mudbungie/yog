//! The boundary's typed answers (§8.5). A [`Reply`] is what
//! [`dispatch`](super::dispatch::dispatch) and
//! [`answer`](super::answer::answer) return — the datum both frontends consume:
//! the GUI reads the variant in RAM, the headless transport writes
//! [`encode`] to the deposit's reply file, and a seat that did not derive the
//! answer reads it back with [`decode`] (REMOTE §9 step 2, bl-7067).
//!
//! It was **encode-only** until that step, on the argument that a reply is
//! yog's own statement rather than an instruction it parses back — the one
//! exception being the `prepare` reply's `prepared` body, which deliberately
//! re-enters as the next [`Prompt`](super::Action::Prompt) gesture and shares
//! its codec spelling. The client/server split (REMOTE §9) retires that
//! argument: a thin seat holds no world, so every answer it renders is one it
//! was told, and the exception has become the general path.
//!
//! The spelling itself is [`encode`], split off at §12's budget (bl-6233): the
//! answer and the way it is said are two subjects, and only one of them is what
//! the window reads.

use super::codec::prepared_value;

/// The §11 conversation seat's own spelling, both directions (REMOTE §9.4,
/// bl-1eb0) — cut off the roster at the budget like `search` and `queue`.
mod agent;
/// The §11 balls section's row — its own file at the budget (bl-b4b5), for
/// the reason its own doc gives.
mod balls;
/// The V4 board row's own encoders — split at the §12 budget, on the seam that
/// board rows are the one reply whose rows carry derived sub-objects (gates,
/// drones, two §3.5 figures).
mod board;
/// The composer's draft-clearing predicate — its own file at the budget.
mod cleared;
/// What a §9 config read answers (bl-dc3f) — the file's two views as one type,
/// and both directions of their spelling.
mod config_view;
/// The whole surface's JSON spelling read back into the type (bl-7067) -- the
/// thin seat's half of the codec, cut on the same seam as the spelling itself.
mod decode;
/// The whole surface's JSON spelling, and the envelope helpers it shares.
mod encode;
/// The answer type itself (bl-1015): the enum and its variant docs, cut
/// off this file at §12's pre-split band on `start/model`'s seam — what an
/// answer *is*, beside the modules that say it.
mod model;
/// The §4.2 trail row in both directions (bl-4d81) — its own file on the seam
/// its own doc gives: the one row whose derived readings are the answer.
mod op_row;
/// The §6 decision queue's row encoder — the other reply whose rows carry a
/// derived list (its firing signals).
mod queue;
/// `pub(crate)` for the §5.1 agent-state token table, which the rail's own
/// encoder reads rather than keeping a second copy of (bl-6233).
pub(crate) mod rows;
/// The search reply's own address-flattening — split at the same budget.
mod search;

/// The one listing row the boundary itself owns — its own file at §12's budget
/// (bl-296f), for the reason its own doc gives.
mod ws_row;

pub use cleared::cleared;
pub use config_view::ConfigView;
pub use decode::decode;
pub use encode::{encode, refusal};
pub use ws_row::{Workspaces, WsRow};

pub use model::Reply;

#[cfg(test)]
pub(crate) mod tests;
