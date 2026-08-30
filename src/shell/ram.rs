//! The shell's cross-frame RAM (§3.5: the frontend holds no durable state;
//! every surface here is discarded on exit). Four holders, one file each —
//! [`StartState`], the start flow's drafts and pending prompt (`ram/start`);
//! the Altitude-2 inspector's viewport ephemera (`ram/inspector`); [`WallRam`],
//! one workspace's own surfaces (`ram/wall`, with the Login pane's in-process
//! runner inside it in `ram/login`); and [`ShellState`], defined here — the one
//! bundle the window's render entry takes so its param list cannot widen.
//! Inert data: no widget is painted here, and the editors these fold own their
//! own seams.
//!
//! **Two lifetimes, spelled structurally** (bl-5894): what belongs to the
//! window sits directly on [`ShellState`], what belongs to a workspace sits
//! inside [`WallRam`], and a focus change swaps the second whole — which is why
//! the two methods that perform that swap live beside `WallRam` in `ram/wall`
//! rather than here, where only the fields they move are declared.

use crate::actions::ActionsState;
use crate::app::WoundGrace;
use crate::shell::ConfigState;
use crate::ui_state::Clock;
use crate::xdg::Env;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use super::{DeleteAgentState, DeleteState, NewWsState, entropy_seed};

mod inspector;
mod login;
mod start;
mod wall;
pub use inspector::InspectorState;
pub use login::LoginHolder;
pub use start::StartState;
pub use wall::WallRam;

/// Every RAM surface the shell owns across frames (§3.5: the frontend holds no
/// durable state; this is all discarded on exit). Bundled so the window's one
/// render entry takes a single mutable handle instead of a widening param list.
pub struct ShellState {
    pub actions: ActionsState,
    pub start: StartState,
    pub inspector: InspectorState,
    /// The inbox-composer's queue RAM (§5.3, bl-929d): the pending-row fold
    /// overrides — keyed by the deposit's inbox path, dying with the pending
    /// row — and the snap machinery (the fold line's one measurement and the
    /// structurally-triggered ease). Viewport ephemera; nothing here reaches
    /// `ui.json`.
    pub composer: crate::composer::ComposerRam,
    /// Which §11 center tab the window is showing (bl-1ca2). Viewport ephemera
    /// (§13.1) — *which surface you are looking at*, never durable — and the
    /// **one** carrier: the three surfaces it seats each used to hold a toggle
    /// of their own, none aware of the others, which is what let a mode paint
    /// over the conversation.
    pub center: crate::keymap::CenterTab,
    /// The **world's** config editors (§9): litany's global file and the yog
    /// clock's, one draft of each per install. brazen's pane is not here — it is
    /// a workspace's, so it rides [`WallRam`] (§16.2 as amended, bl-5894).
    pub config: ConfigState,
    /// The focused workspace's own surface RAM (§16.2 as amended): brazen's
    /// config pane, the §8.3 Login pane and the §9.4 model picker. Swapped
    /// whole by [`focus_wall`](Self::focus_wall) — never re-lensed in place.
    pub wall: WallRam,
    /// Deferred request that the composer take the keyboard next frame — the
    /// **one** focus mechanism (§11; `super::focus` owns the rules and is the
    /// only module that touches this). Spent by whichever composer paints.
    pub focus_composer: bool,
    /// The **one-frame repeat** of [`Self::focus_composer`] (bl-58e4), and not a
    /// second request path: only `super::focus::take` ever sets it, only when
    /// the ask rode a bare arrow, and Escape cancels it. Why it exists is ruled
    /// there — egui walks the focus floor on an arrow the newly-focused box has
    /// not yet claimed, so the ask has to survive one frame to actually land.
    pub(crate) refocus_composer: bool,
    /// The conversation-list organizing view (§11, §15 Z9): `false` = flat by
    /// recency (the default), `true` = grouped by ball. Viewport ephemera (§13.1):
    /// which ordering you look at, not data — RAM, no `ui.json` field.
    pub group_by_ball: bool,
    /// The conversation list's **expanded set** (§11 unfold, bl-fa82): the agent
    /// ids whose descent children are painted as rows beneath them. Viewport
    /// ephemera (§13.1) in the mould of the jsonview collapse set — *which data
    /// you look at*, not data — so it is RAM and deliberately not `ui.json`'s
    /// `collapsed` array, which names a fixed handful of sections rather than
    /// one key per conversation that ever existed. Empty is the whole list
    /// collapsed, which is the seat's own pre-unfold rendering, so losing it
    /// loses nothing. Flipped only through [`crate::jsonview::toggle_path`], the
    /// crate's one disclosure toggle.
    pub expanded: HashSet<String>,
    /// Whether the activity accessory is expanded (§11, §13.0 viewport state).
    /// Held here rather than in egui's collapsing-header memory because the `a`
    /// binding and the header's own click must move one fact, not two.
    pub activity_open: bool,
    /// The §3.6 confirmation's RAM: which workspace is being unmade and the
    /// typed arming name. Both §11 carriers open this one dialog; no key does.
    pub delete: DeleteState,
    /// The §3.6 agent-delete confirmation's RAM (bl-f17a): the conversation
    /// under confirmation, its dry-run census, the typed arming name. Both
    /// §11 carriers open this one dialog; no key does.
    pub delete_agent: DeleteAgentState,
    /// The §11 `new` form's RAM: the workspace name being typed (§3.1). The
    /// tab and the `w` binding open this one form.
    pub new_ws: NewWsState,
    /// **The act this window has fired and not yet heard back about** (REMOTE
    /// §9.8, bl-1747): one hold, for the composer's box and the §8.5 line —
    /// the two seats whose receipt gates a frame-side fact rather than a
    /// sentence. `None` on every frame but the handful after a gesture, and the
    /// §3.6 dialogs hold their own beside their own confirmations.
    pub(super) acting: Option<super::acting::Acting>,
    /// The last §8.5 line's answer or refusal, verbatim (§5.3 RAM, per
    /// instance): a reply's own JSON, or the reason the line was not a gesture.
    /// It is what a typed control says back — the composer's counterpart of the
    /// ops-trail line an action writes anyway — and it is replaced by the next
    /// command, never durable and never converged.
    pub slash: Option<String>,
    /// The §7.3 wound banner's grace window (bl-90bf): the render-layer age
    /// gate that withholds the alarm until the wound has held long enough to
    /// outlive the snapshot's liveness lag. Clock-gated RAM, so it lives here
    /// rather than in the pure predicate ([`crate::app::WoundGrace`]).
    pub wound_grace: WoundGrace,
    /// The orphaned-mail banner's grace window (bl-ace6): the same age gate
    /// over the same liveness lag — a delivered message whose driver simply
    /// has not been seen yet must not alarm. Its own instance, because the
    /// two predicates share an (workspace, agent) key and one gate's timer
    /// must not answer for the other's.
    pub orphan_grace: WoundGrace,
    /// What this window has already told the desktop (§6 as amended, bl-e160).
    /// Viewport ephemera by construction: a desktop belongs to a window, so two
    /// instances each announce their own and neither converges (§13.1) — and a
    /// restart is a new window, which is exactly why a fresh one announces
    /// nothing it merely inherited.
    pub alerts: crate::alert::Announced,
    /// The composed world (§16.2) each wall's RAM is folded from the first time
    /// its workspace takes focus ([`ShellState::focus_wall`]).
    world: Env,
    /// Which workspace [`wall`](Self::wall) belongs to — the cursor over
    /// [`parked`](Self::parked), and the comparison that makes a swap a change
    /// event rather than a per-frame cost (§7.2).
    wall_at: Option<PathBuf>,
    /// Every *other* wall the operator has focused this session, holding its own
    /// surfaces exactly as it left them. The live wall is never in here: it is
    /// taken out on focus and put back on the way past, so one wall's RAM has
    /// one home at all times.
    parked: HashMap<Option<PathBuf>, WallRam>,
}

impl ShellState {
    /// Fold the config editors from the env snapshot (their paths + runners);
    /// the rest default. A missing brazen/litany file loads as an empty draft,
    /// not an error (§9), so this only fails on an unexpected io error.
    ///
    /// `clock` is the crate's one injected time source (§7.2), shared with the
    /// model — the shell's own timing decision (the §7.3 banner's grace) reads
    /// the same time the derivation it is waiting on does.
    pub fn new(env: &Env, clock: Arc<dyn Clock>) -> std::io::Result<Self> {
        Ok(Self {
            // The §11 birth-config block's work-directory box is born holding
            // the default it would otherwise only imply (bl-7927): the bare
            // rung's own driver cwd, §3.4's `~`, resolved here at the one
            // boundary that reads the env. Pre-filling it is what retires "dir
            // (optional)" — the field always states where the next start runs.
            actions: ActionsState {
                path_dir: env.home_dir().display().to_string(),
                ..ActionsState::default()
            },
            start: StartState {
                mint_seed: entropy_seed(),
                ..StartState::default()
            },
            inspector: InspectorState::default(),
            composer: crate::composer::ComposerRam::default(),
            center: crate::keymap::CenterTab::default(),
            config: ConfigState::new(env)?,
            // Launch focuses no wall yet, and the no-wall bundle is a wall like
            // any other (§11: a workspace may not be focused) — the general path
            // with an empty input, not a bootstrap case.
            wall: WallRam::new(&crate::world::wall::env_opt(env, None), None),
            // **Launch is not a special case** (§11 focus discipline): the
            // request stands from the start, so the first composer to paint
            // takes the keyboard — retiring the bootstrap's own memory flag.
            focus_composer: true,
            refocus_composer: false,
            group_by_ball: false,
            expanded: HashSet::new(),
            activity_open: false,
            delete: DeleteState::default(),
            delete_agent: DeleteAgentState::default(),
            new_ws: NewWsState::default(),
            acting: None,
            slash: None,
            alerts: crate::alert::Announced::default(),
            wound_grace: WoundGrace::new(clock.clone()),
            orphan_grace: WoundGrace::new(clock),
            world: env.clone(),
            wall_at: None,
            parked: HashMap::new(),
        })
    }
}
