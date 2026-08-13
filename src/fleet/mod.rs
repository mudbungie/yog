//! The armed loop (VISION §4.3, story rung V4 item 2): the backend's own
//! level-triggered pass that brings a workspace's drone count to match ready
//! work under a policy cap, and reaps by comparison.
//!
//! **It is off until the operator arms it, and arming is a gesture.** Unarmed —
//! the default, and what every existing world is — nothing here runs: no board
//! fact renders, no row is written and no verb is spawned. Severability is
//! deleting one `cadence.yaml` entry, never editing a code path ([`arming`]).
//! I7 (*yog never mutates except on explicit user action*) is preserved exactly
//! as §4.3 rules it: **the arm is the explicit action**, and an armed loop's
//! spawns are that action, continuing.
//!
//! **The loop spawns and reaps; it never diagnoses** (§4.3, verbatim). Both of
//! its moves are comparisons over facts that already have owners:
//!
//! - a **spawn** happens when the workspace holds fewer balls than its cap and
//!   the board has a ready one in its project. It is the ordinary §8.1 start
//!   flow through the ordinary boundary door, so the §3.5 spend ceiling gates
//!   it exactly as it gates a click (bl-56d5 owns that gate; this composes it);
//! - a **reap** happens when every conversation on a claimed ball has been
//!   quiet longer than the workspace's lease. It releases the claim — `bl
//!   unclaim`, the verb an operator would use — and touches the conversation
//!   not at all. Nothing running is ever stopped: killing mid-ball destroys
//!   uncommitted work, which is the same ruling the ceiling already carries.
//!   The reason it records is the **comparison itself** ("lease expired 14m
//!   ago"), never a judgement about why the drone went quiet.
//!
//! **Everything it knows is derived.** There is no cap field, no count field
//! and no tick record anywhere: the cap is the config entry, the count is the
//! board's own claimed rows, and the last tick is the newest row the loop left
//! ([`facts`]). Its only durable is one `ops.jsonl` line per action ([`row`]).
//!
//! The pieces: [`arming`] is the config tie-point, [`facts`] the derivation the
//! board renders, [`row`] the ops-row encoding, and [`pilot`] the off-thread
//! level trigger that fires at most one move per tick.

pub mod arming;
pub mod facts;
pub mod pilot;
pub mod row;

use std::path::PathBuf;

/// The loop's gestures, as the control boundary carries them (VISION §4.3,
/// DESIGN §8.5). One boundary [`Action`](crate::boundary::Action) variant holds
/// this enum rather than two holding its arms — the same fold the monitor's
/// family takes, and for the same reasons: one subject, one config file, one
/// trail, and the boundary's four tables stay one row wider instead of two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// Arm one workspace: write its `cadence.yaml` fleet entry, naming the
    /// project it takes work from and the cap it may hold.
    Arm {
        workspace: PathBuf,
        project: PathBuf,
        cap: usize,
    },
    /// Disarm it: delete that entry. Its own gesture rather than an arm with a
    /// cap of zero — a zero cap is an armed loop that spawns nothing and still
    /// reaps, which is a different instruction.
    Disarm { workspace: PathBuf },
}

pub use arming::Policy;
pub use facts::Facts;
pub use pilot::{Pilot, PilotCtx};
pub use row::Act;
