//! The shell's cross-frame RAM (§3.5: the frontend holds no durable state;
//! every surface here is discarded on exit). Four holders — the start flow's
//! drafts and pending prompt, the Altitude-2 inspector's viewport ephemera
//! (`ram/inspector`), [`WallRam`], one workspace's own surfaces (`ram/wall`,
//! with the Login pane's in-process runner inside it in `ram/login`), and
//! [`ShellState`], the one bundle the window's render entry takes so its param
//! list cannot widen. Inert data: no widget is painted here, and the editors
//! these fold own their own seams.
//!
//! **Two lifetimes, spelled structurally** (bl-5894): what belongs to the
//! window sits directly on [`ShellState`], what belongs to a workspace sits
//! inside [`WallRam`], and a focus change swaps the second whole.

use crate::actions::ActionsState;
use crate::app::WoundGrace;
use crate::shell::ConfigState;
use crate::start::Prepared;
use crate::ui_state::Clock;
use crate::xdg::Env;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use super::{DeleteAgentState, DeleteState, NewWsState, entropy_seed};

mod inspector;
mod login;
mod wall;
pub use inspector::InspectorState;
pub use login::LoginHolder;
pub use wall::WallRam;

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
}

impl StartState {
    /// Retire the seed a landed fire just spent, re-rolling from entropy
    /// (bl-28ba) — called at the one point the old prediction dies, so the next
    /// preview predicts off a seed of its own. A refused or failed launch minted
    /// nothing, so its prediction stands and its seed is not spent.
    ///
    /// Held past its fire, one seed served the whole session: the mint takes ONE
    /// draw (§3.3), so every later fire re-drew the same start index, landed on
    /// the occupied slot and walked one forward — and `names::pair` is
    /// first-word-major, so the walk paid out `recite-a`, `recite-b`, `recite-c`.
    pub fn spend_mint(&mut self) {
        self.mint_seed = entropy_seed();
    }
}

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
    /// The **world's** config editors (§9): lernie's global file and the yog
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
    /// the rest default. A missing brazen/lernie file loads as an empty draft,
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
            slash: None,
            alerts: crate::alert::Announced::default(),
            wound_grace: WoundGrace::new(clock),
            world: env.clone(),
            wall_at: None,
            parked: HashMap::new(),
        })
    }

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
    /// precisely so a workspace minted later under the same §3.1 name cannot
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
