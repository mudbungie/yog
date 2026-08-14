//! The control boundary (VISION §4.8, DESIGN §8.5): the one typed surface
//! every operator gesture crosses.
//!
//! Three families, decided per gesture by the §4.8 ruling. **Actions** mutate
//! the world and are the `ops.jsonl` trail's rows (§4.2); **queries** populate
//! — most are a §2 I1 derivation over the published [`Snapshot`](crate::app::Snapshot),
//! and the §9 config family's reads (bl-0164) are the same on-demand read
//! their write already is, over [`dispatch::Deps`]'s world — returning the
//! same typed data both frontends render; **views** (focus, scroll, tab
//! selection, drafts — §5.3's closed RAM whitelist, and the §4.1 presentation
//! durables beside it) never cross the boundary and gain no representation
//! here.
//!
//! The carrier is a datum, not a convention: the GUI's click-glue constructs
//! [`Action`]/[`Query`] variants and [`dispatch`](dispatch::dispatch) /
//! [`answer`](answer::answer) are the chokepoints both frontends share. The
//! headless serialization is the [`codec`] JSON envelope, deposited as a
//! create-only file into the yog-watched `gestures/` inbox ([`deposit`]),
//! consumed off-frame ([`consume`], [`consumer`]) and answered as a [`reply`]
//! file; `yog gesture` ([`sugar`]) is deposit-and-wait sugar over exactly that.
//! One surface, two serializations, never two implementations (VISION §8).
//!
//! **A new gesture without a headless spelling fails to compile**: adding a
//! variant here leaves [`codec::encode`], [`codec::decode`] and the dispatch
//! match non-exhaustive until the spelling exists.

/// The windowless face's leading word (§8.5): `yog headless`. Named here, once,
/// because two spellings — the arm that dispatches it and the help that
/// advertises it — would be two facts.
pub const HEADLESS_SUBCMD: &str = "headless";

pub mod answer;
/// The §3.5 spend ceiling's one seat — the spawn gate.
pub mod ceiling;
pub mod codec;
pub mod config;
pub mod consume;
pub mod consumer;
/// The VISION §4.11 capability family's one executor — the hold answer's row
/// and its releasing `advance` — plus the confinement-required birth gate.
pub mod control;
pub mod deposit;
pub mod dispatch;
/// The VISION §4.10 mutating fan's two executors — spread and retire.
pub mod fan;
/// The VISION §4.3 armed loop's one executor — arming, which is a config write.
pub mod fleet;
pub mod help;
pub mod line;
/// The VISION §4.9 monitor's two executors — arming and flagging.
pub mod monitor;
pub mod reply;
pub mod sugar;
/// `pub(crate)` so the board's own corpus shares this one `Agent`/`Snapshot`
/// fixture rather than standing up a second of the same shape.
#[cfg(test)]
pub(crate) mod tests;

/// Anything that crosses the boundary: an action or a query. Views do not —
/// they are §5.3's whitelist and have no spelling here, by design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gesture {
    Act(Action),
    Ask(Query),
}

/// The mutating roster, its own file at §12's cap — the seam [`query`] is
/// already cut on, one enum over (bl-8746). The two rosters are the §8.5
/// taxonomy said in code.
mod action;
pub use action::Action;

/// The §8.2 after-verb ball-refresh target, one table over the action roster —
/// split out at §12's cap (bl-dc0c), because it is a *query on* the enum rather
/// than part of it.
mod project;

/// The populating-read roster, its own file at §12's cap (bl-765d). The seam is
/// the §8.5 taxonomy the help table is already cut along: actions mutate,
/// queries populate — two rosters, and only one of them can ever be wrong about
/// the world.
mod query;
pub use query::Query;
