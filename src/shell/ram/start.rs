//! The start flow's own holder (§5.3 RAM carve-out): the per-project new-ball
//! drafts, the pending detached prompt, the fan's N and the seed the §3.3 name
//! prediction is drawn from. Split off [`super`] at §12's budget on the seam
//! its siblings already sit on — `ram/inspector`, `ram/login` and `ram/wall`
//! are each one holder in one file, and this was the fourth still inline.
//!
//! Inert data: nothing here paints, and the start pane owns its own seam.

use crate::start::Prepared;
use litany::mint::{Rng, SplitMix64};
use std::collections::HashMap;
use std::path::PathBuf;

/// The transient start-flow input (RAM, §5.3 carve-out): the per-project
/// new-ball drafts and, after [`start_pane`] runs `prepare`, the pending
/// detached prompt — its editable goal and the (workspace, worktree) it fires
/// against. Discarded on exit; nothing here is durable (§8.1 draft is RAM).
#[derive(Default)]
pub struct StartState {
    /// New-ball (title, body) drafts keyed by project path.
    pub new_ball: HashMap<PathBuf, (String, String)>,
    /// The composer's editable goal + targets, `Some` once `prepare` succeeds.
    pub pending: Option<Prepared>,
    /// The conversation-mint RNG seed (RAM, §5.3): held stable across frames so
    /// the composer's greyed name prediction (§3.3) predicts the name each
    /// frame *and* at fire — a fresh `SplitMix64::from_seed(mint_seed)` for both
    /// the pure preview read and the fire's own mint. **A seed lives exactly as
    /// long as the prediction it backs** ([`StartState::spend_mint`]).
    pub mint_seed: u64,
    /// The §3.8 fan's **N picker** (bl-77bc): how many isolated candidates the
    /// pending start fires as. `0` reads as 1 — the ordinary single start, the
    /// same fold with one input — and a landed fan resets it, so N is a fact
    /// about the *next* fire, never a sticky mode.
    pub fan_n: usize,
}

impl StartState {
    /// Retire the seed a landed fire just spent (bl-28ba) — called at the one
    /// point the old prediction dies, so the next preview predicts off a seed of
    /// its own. A refused or failed launch minted nothing, so its prediction
    /// stands and its seed is not spent.
    ///
    /// Held past its fire, one seed served the whole session: the mint takes ONE
    /// draw (§3.3), so every later fire re-drew the same start index, landed on
    /// the occupied slot and walked one forward — and the pool is
    /// first-word-major, so the walk paid out `recite-a`, `recite-b`, `recite-c`.
    ///
    /// The successor is **the seed's own stream**, not a second entropy read
    /// (bl-dd3d): one `SplitMix64` step off the spent value. Entropy enters a
    /// session exactly once, where the first seed is minted
    /// ([`ShellState::new`](super::ShellState::new)), which is what makes a
    /// *known* opening seed pin the whole run of names rather than only its
    /// first — the acceptance world pins that one read, and every later
    /// prediction follows from it. A second clock read here bought nothing (the
    /// successor was already unpredictable from a seed nobody publishes) and
    /// cost the suite determinism: it made "the third name differs from the
    /// first" a probabilistic assertion over litany's 541-word pool instead of
    /// a fact of the pinned seed.
    pub fn spend_mint(&mut self) {
        self.mint_seed = SplitMix64::from_seed(self.mint_seed).next_u64();
    }
}
