//! The **wall's own RAM** (DESIGN §16.2 as amended by the
//! blast-radius ruling, §5.3): every cross-frame surface that belongs to one
//! workspace rather than to the window.
//!
//! The ruling, verbatim (§3.1): *"workspaces are an entirely separate space;
//! essentially an app-wide blast radius. Different sets of conversations,
//! settings, providers, all of it."* Three surfaces are settings in that sense
//! — brazen's config pane, the §8.3 Login pane, the §9.4 model picker — and
//! bl-c0e2 moved the *files* behind them into the wall while leaving the RAM
//! over those files as one box per window, re-lensed on focus. That was two
//! defects at once (bl-5894): a draft typed in workspace A died when focus
//! moved, and a live sign-in stream, an open picker and a fetched roster from A
//! stayed on screen and actionable under B.
//!
//! **The wall owns the box, so nothing has to be cleared.** A focus change
//! parks this bundle under the workspace it belongs to and takes that
//! workspace's own out ([`ShellState::focus_wall`](super::ShellState::focus_wall));
//! `None` — no workspace focused — is a wall like any other, holding the
//! surfaces' empty answers rather than a special case. Preservation and
//! isolation stop being two rules: A's state is preserved *because* it never
//! left A, and B cannot see it for the same reason.
//!
//! Still RAM (§3.5, §5.3): the map dies with the process, exactly like the one
//! box it replaces. This is the *key*, not persistence — the same shape
//! [`Drafts`](crate::actions::Drafts) already keys the composer by.

use crate::shell::{BrazenPane, LoginHolder, PickerState};
use crate::xdg::Env;

/// One workspace wall's surface RAM: brazen's config pane (its draft, status
/// and provider rows), the Login pane's runner and its live sign-in stream, and
/// the §9.4 model picker (its open flag, role, half-made pick and roster).
pub struct WallRam {
    pub brazen: BrazenPane,
    pub login: LoginHolder,
    pub picker: PickerState,
}

impl WallRam {
    /// Fold all three from `wall` — the lensed env of the workspace this bundle
    /// belongs to ([`crate::world::wall::env_opt`]), or the world with no wall
    /// standing when no workspace is focused. Infallible: every surface here
    /// answers a missing file with its own emptiness (§9.1), which is the truth
    /// the pane renders rather than an error the frame could not act on.
    pub(super) fn new(wall: &Env, workspace: Option<&std::path::Path>) -> Self {
        Self {
            brazen: BrazenPane::new(wall),
            login: LoginHolder::new(wall, workspace),
            picker: PickerState::new(wall),
        }
    }
}
