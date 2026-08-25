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

use super::ShellState;
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

/// **The swap itself** (bl-5894): the two window-level methods that move a
/// wall's RAM in and out of [`ShellState::wall`]. They live beside [`WallRam`]
/// rather than beside the fields they move, because what they implement is this
/// file's own invariant — one wall's RAM has one home at all times — and the
/// fields are only where that home is recorded.
impl ShellState {
    /// Point every wall-bound surface at the focused workspace's **wall**
    /// (§16.2 as amended, §3.1's blast radius): brazen's config pane, the login
    /// roster and its live sign-in stream, and the §9.4 picker. One call,
    /// because they are one sphere's settings — switching workspace switches
    /// providers, sign-in state and model cache together or it switches none of
    /// them honestly.
    ///
    /// **A swap, not a re-lens** (bl-5894). The outgoing wall's RAM is parked
    /// under the workspace it was typed in and the incoming wall's is taken back
    /// out, so a draft, an open picker and a running sign-in survive A → B → A
    /// intact while none of them can paint or be acted on under B. Re-lensing
    /// one box could only ever pick one of those two: it lost the draft *and*
    /// carried the stream.
    ///
    /// Idempotent and change-driven: a frame whose focus has not moved does
    /// nothing at all, so this is not a per-frame cost (§7.2). A wall is folded
    /// from the world exactly once, the first time its workspace takes focus.
    pub fn focus_wall(&mut self, workspace: Option<&std::path::Path>) {
        if self.wall_at.as_deref() == workspace {
            return;
        }
        let next = workspace.map(std::path::Path::to_path_buf);
        let incoming = self.parked.remove(&next).unwrap_or_else(|| {
            WallRam::new(
                &crate::world::wall::env_opt(&self.world, workspace),
                workspace,
            )
        });
        let outgoing = std::mem::replace(&mut self.wall, incoming);
        let previous = std::mem::replace(&mut self.wall_at, next);
        self.parked.insert(previous, outgoing);
    }

    /// Unmake a wall's RAM with its workspace (§3.6). A wall's RAM lives exactly
    /// as long as its wall: §16.2 deletes the wall *directory* with the sphere
    /// precisely so a workspace created later under the same §3.1 name cannot
    /// inherit a dead one's credentials, and the box over that directory has to
    /// die on the same terms — the key here is the workspace path, which a
    /// same-named rebirth reoccupies exactly.
    ///
    /// Total over both homes, so there is no "was it focused?" case: the parked
    /// entry goes, and a live one is replaced by the no-wall bundle, which is
    /// the truth after unmaking the sphere you were standing in.
    pub fn forget_wall(&mut self, workspace: &std::path::Path) {
        self.parked.remove(&Some(workspace.to_path_buf()));
        if self.wall_at.as_deref() == Some(workspace) {
            self.wall = WallRam::new(&crate::world::wall::env_opt(&self.world, None), None);
            self.wall_at = None;
        }
    }
}
