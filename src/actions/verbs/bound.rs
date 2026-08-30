//! The **workspace-bound litany seam** (DESIGN §8.2, §16.2) — [`Bound`].
//!
//! Every §8.2 `litany` verb is *about one workspace*, and a workspace-bound
//! spawn owes that workspace two env facts: its **wall** (`YOG_WALL`, §16.2 as
//! amended — brazen's config, sign-ins and model cache) and its **name**
//! (`YOG_NAME`, §8/§3.3 — the `--as` stamp the W9 shim writes). Before bl-bf79
//! each verb laid its own layer, and the three that revive a driver laid the
//! name without the wall: `litany message` deposits and then **detach-launches**
//! a driver when the branch is quiescent (litany ARCH §2.9 — there is no resume
//! verb; the deposit restarts a driver), that driver inherited yog's fold, and
//! its first `bz` died with
//!
//! ```text
//! bz: no workspace in this environment — providers, sign-ins and the model
//! cache belong to a workspace, and there is nothing shared to fall back to.
//! ```
//!
//! so every message that had to revive a quiescent conversation produced an
//! empty reply. The first turn worked only because `litany prompt` is fired from
//! [`boundary::dispatch`](crate::boundary::dispatch), which does lay the wall.
//!
//! **The fix is the seam, not three reminders.** A §8.2 workspace verb takes a
//! `Bound` and nothing else: there is no bare [`Cli`] in its signature to hand
//! it an unwalled spawn, and the workspace it runs against is the one this value
//! was constructed at. So the fold is stated once, at the edge that knows the
//! workspace, and a verb added later inherits it by construction rather than by
//! remembering — which is the whole of §16.2's *"set once, at the edge that
//! knows the workspace, and no downstream seat has to be told"*.
//!
//! **Uniform, with no exemption.** `stop` launches nothing, so the wall buys it
//! nothing — but "which workspace verbs may skip the wall" is exactly the
//! per-verb decision that produced the bug. A `Bound` is what a workspace verb
//! *is*; an inert layer on the one verb that ignores it is cheaper than an
//! asymmetry every future reader has to re-derive.

use std::path::{Path, PathBuf};

use crate::cli_outbound::Cli;

/// The workspace-scoped identity env (§8/§3.3): the W9 shim stamps `--as` onto a
/// revived driver's unstamped `bl` verbs from it.
const YOG_NAME: &str = "YOG_NAME";

/// A `litany` [`Cli`] bound to one workspace — the only handle the §8.2
/// workspace verbs accept. Constructing it lays the workspace's wall and name on
/// the spawn; carrying the workspace with it means the verb's cwd and its `<ws>`
/// argv are one fact rather than a repeated argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    cli: Cli,
    workspace: PathBuf,
}

impl Bound {
    /// Bind `litany` to `workspace` inside `world`: `YOG_WALL` (§16.2) and
    /// `YOG_NAME` (§3.3) layered **on top of** the world `Cli`'s standing
    /// nesting set, which [`Cli::and_env`] extends rather than replaces — and
    /// the §8.6 confinement wrapper when the workspace's live policy requires
    /// one, folded here for the same reason the wall is: stated once, at the
    /// edge that knows the workspace, so no verb — including one written later
    /// — can spawn a workspace-bound child outside the sandbox its policy
    /// demands. Empty when the policy states nothing (severability).
    pub fn at(litany: &Cli, world: &crate::xdg::Env, workspace: &Path) -> Self {
        let mut layer = crate::world::wall::pairs(world, workspace);
        layer.push((YOG_NAME.to_owned(), crate::naming::leaf(workspace)));
        Self {
            cli: litany
                .and_env(layer)
                .and_wrapper(crate::control::confine::wrapper(world, workspace)),
            workspace: workspace.to_path_buf(),
        }
    }

    /// The bound spawn surface. `pub(crate)` — an internal accessor, so it
    /// borrows rather than clones-to-own (AGENTS rule 2's own escape).
    pub(crate) fn cli(&self) -> &Cli {
        &self.cli
    }

    /// The workspace this is bound to: every verb's cwd (§8.2).
    pub(crate) fn workspace(&self) -> &Path {
        &self.workspace
    }

    /// The same workspace as the `<ws>` argv word every §8.2 litany verb takes.
    pub(crate) fn workspace_arg(&self) -> String {
        self.workspace.to_string_lossy().into_owned()
    }
}
