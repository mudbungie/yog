//! The environment a gesture executes in (§8.5) — [`Deps`], its own file per
//! §12's budget. It is built fresh at each call site (cheap clones, no held
//! state), which is what keeps [`dispatch`](super::dispatch) pure over its
//! inputs and deterministic under test.

use crate::app::Snapshot;
use crate::cli_outbound::Cli;
use crate::config_edit::brazen::BzRunner;
use std::path::PathBuf;
use std::sync::Arc;

/// What a gesture needs to execute (§8.5).
///
/// **The §3.3 mint seed is not here** (bl-1747). It was, and it was the one
/// field a *seat* filled: the window reached into `Deps` to hand over the seed
/// its greyed prediction had already been drawn off, so the preview and the
/// fired `--name` agreed. A window that posts its acts over the wire can reach
/// into nothing — the engine builds `Deps`, one per gesture — so the seed rides
/// [`Action::Prompt`](super::super::Action::Prompt) instead, which is where it
/// always belonged: it is a parameter of the gesture, not of the environment,
/// and a caller that predicted nothing carries `None` and takes the door's own
/// stamp-derived draw.
#[derive(Clone)]
pub struct Deps {
    pub lernie: Cli,
    pub bl: Cli,
    /// `ops.jsonl`'s root (§4.2).
    pub state_root: PathBuf,
    /// yog's own binary — the `$EDITOR` shim the §9.3 lineage write re-enters.
    pub yog_binary: PathBuf,
    /// The composed world (§16.2): what the §9 config family folds its
    /// destinations from, and the snapshot the **linked** brazen answers
    /// [`provider_rows`](Self::provider_rows) through.
    pub world: crate::xdg::Env,
    /// The bare rung's driver cwd (`~`), resolved at the process boundary.
    pub home: PathBuf,
    pub yog_data_root: PathBuf,
    pub balls_state_root: PathBuf,
    /// The published derivation the start/delete families read (§7.2): the
    /// occupied conversation names, the §3.6 confirmation's liveness + claims.
    pub snapshot: Arc<Snapshot>,
    /// Who is asking, and who else is connected (REMOTE §4, §5).
    pub caller: Caller,
}

/// **The connection facts a gesture runs under** (REMOTE §4, §5; bl-4e08,
/// bl-024b) — what is true of the *caller* rather than of the world, which is
/// why these ride here together and not on the snapshot: a derivation is
/// republished on the worker's cadence, and every one of these changes on a
/// peer's. Who is asking, who else is connected, and what is queued for whom.
///
/// The default is the in-world posture (§3): the reserved `local` identity, a
/// presence map nobody has entered and a mailbox nobody has posted to — which
/// is exactly right for a box with no wire provisioned, the deposit inbox and
/// every test. The general path with no input, not a case of its own.
///
/// **A face that builds its own `Deps` gets that default, and it is not the
/// engine's.** The window's click-glue and the §4.3 pilot each construct one,
/// so a gesture fired there reaches a presence map and a mailbox no listener
/// touches. That is right for every gesture they *can* fire — none names a
/// client — and it is the trap to know about before wiring
/// [`Action::Route`](crate::boundary::Action::Route) to a control: the routing
/// leg is answered through [`ConsumerCtx`](crate::boundary::consumer::ConsumerCtx),
/// which holds the engine's own handles, and nothing else may.
#[derive(Clone, Default)]
pub struct Caller {
    /// The identity the intake carries: a connection's certificate common name
    /// (REMOTE §4, read exactly where scoping reads it), or `local` for the
    /// window, the `gestures/` inbox and `yog gesture`.
    pub client: crate::registry::Client,
    /// Which clients hold a live connection right now (REMOTE §5) — the wire
    /// server's own RAM, shared by handle so an answer reads this instant's
    /// truth rather than a copy taken when the gesture arrived.
    pub presence: crate::registry::presence::Presence,
    /// What is queued for each client and what came back (REMOTE §5, bl-024b)
    /// — the engine's own RAM, shared by handle for presence's reason exactly:
    /// an invocation posted through one intake is drained through another, and
    /// a copy taken when the gesture arrived would be a hand-off to nobody.
    pub mailbox: crate::registry::mailbox::Mailbox,
}

impl Deps {
    /// This gesture's `lernie` bound to `workspace` (§8.2, §16.2) — the one
    /// composition point for a workspace-bound spawn, so the sphere's wall and
    /// name ride every §8.2 lernie verb by construction rather than by each
    /// arm remembering to layer them (bl-bf79,
    /// [`verbs::Bound`](crate::actions::verbs::Bound)).
    pub fn bound(&self, workspace: &std::path::Path) -> crate::actions::verbs::Bound {
        crate::actions::verbs::Bound::at(&self.lernie, &self.world, workspace)
    }

    /// brazen's effective provider table, by name (§5.1 #20) — **asked, never
    /// stored**: the rows are brazen's fact, so every gate that needs them
    /// (§9.2's apply, §9.4's pick, §9.5's provider control) reads the
    /// one answer at the moment it acts, against the wall of the workspace
    /// whose file it is judging (§16.2). This retires the headless interim §8.5
    /// recorded, where the consumer left the table empty and rode "empty gates
    /// nothing" because brazen was unasked; both faces now ask.
    pub fn provider_rows(&self) -> Vec<String> {
        crate::config_edit::brazen::row_names(
            &crate::config_edit::brazen::RealBzRunner::resolve(&self.world).providers(),
        )
    }
}
