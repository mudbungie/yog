//! The §11 **Config tab** (§9): the three write surfaces — brazen's
//! `config.toml`, lernie's global config, this workspace's config branches —
//! as **controls over facts** since §9.5: each setting the files declare gets the
//! widget its kind implies, and raw text survives only where §9.5 justifies it.
//!
//! **The three surfaces do not share a lifetime.** brazen's file is a
//! *workspace's* (§16.2 as amended), so its whole pane is per-wall RAM
//! ([`brazen_pane`], bl-5894); the lernie-global and cadence files are the
//! world's, one per install, so [`ConfigState`] holds exactly one draft of each
//! however many spheres the operator walks through. Two homes because there are
//! two lifetimes, not two mechanisms.
//!
//! Coverage-excluded glue: the editors ([`Editor`], the config-branch [`edit`])
//! and the typed view ([`form`]) are pure tested view-models over the injected
//! [`RealFileIo`] seam; these files wire them to widgets. `ConfigState` is RAM
//! (§3.5), folded once from the env snapshot and discarded on exit.
//!
//! **Nothing here reads disk, git or brazen per frame** (§7.2, bl-ee0a): the
//! provider table, the credential rows and the workflow listing are asked at
//! the pane's [`open`](ConfigState::open) gesture — which is already §9's
//! freshness rule — and the config-branch reads are a Browse click. The frame
//! renders what those gestures left behind.
//!
//! [`edit`]: crate::config_edit::branch::edit
//! [`form`]: crate::config_edit::form

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::config_edit::RealFileIo;
use crate::config_edit::branch::ConfigBranch;
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::brazen::row_names;
use crate::config_edit::lernie_global::{Editor, LernieGlobal};
use crate::shell::config_marks::{self, MarksPane};
use crate::xdg::Env;
use std::path::PathBuf;

mod branch_pane;
pub(crate) mod brazen_pane;
mod form_ui;
mod lernie_pane;
mod status;
mod yog_pane;

pub(crate) use brazen_pane::BrazenPane;

/// Reload, said once for both file editors (§9.1/§9.2).
const RELOAD_HINT: &str = "Throw away the edits in this box and re-read the file from disk. Nothing \
     is written. Reachable by Tab, pressed with Space; `/config <destination>` with no \
     text reads the same bytes.";

/// The new-workflow name field (§9.2).
const NEW_WORKFLOW_HINT: &str = "Name for a new `workflows/<name>.yaml`. Create opens an empty editor for \
     it; the file itself appears only when you Apply. Typed, it is \
     `/config workflow <name> <text…>`.";

/// The **world's** config editors and their RAM (§9). brazen's `config.toml` is
/// deliberately not here: it belongs to a workspace's wall, so it is
/// [`BrazenPane`] — one field of the per-wall RAM (§16.2 as amended, bl-5894) —
/// while the lernie-global and cadence files are the world's, one per install,
/// and there is exactly one draft of each no matter which sphere is focused.
pub struct ConfigState {
    io: RealFileIo,
    lernie: LernieGlobal,
    lernie_editor: Editor,
    lernie_status: String,
    /// The yog clock's own file (§7.2, bl-3381), on the same editor discipline.
    cadence_editor: Editor,
    cadence_status: String,
    workflows: Vec<PathBuf>,
    new_workflow: String,
    new_model: String,
    new_model_row: String,
    branches: Vec<ConfigBranch>,
    cb_files: Vec<String>,
    cb_name: String,
    cb_origin: EditOrigin,
    cb_path: String,
    cb_body: String,
    cb_status: String,
    /// The per-project no-marks knob (§16.3), its own pane in [`config_marks`].
    ///
    /// [`config_marks`]: super::config_marks
    marks: MarksPane,
}

impl ConfigState {
    /// Fold the editors from the env snapshot (§9). A missing lernie file loads
    /// as an empty draft (§9.2), never an error.
    pub fn new(env: &Env) -> std::io::Result<Self> {
        let io = RealFileIo;
        let lernie = LernieGlobal::resolve(env);
        let lernie_editor = Editor::load(lernie.models(), &io)?;
        // The clock's file (bl-3381): absent means the defaults, so a fresh
        // world's pane seeds the default template — every row renders, and the
        // first Apply creates the file it edits (the must-not-exist guard is
        // the seeded editor's own).
        let cadence_path = env.yog_state_root().join(crate::app::cadence::CADENCE_YAML);
        let mut cadence_editor = Editor::load(cadence_path.clone(), &io)?;
        if cadence_editor.is_new() {
            cadence_editor = Editor::seeded(cadence_path, crate::app::cadence::TEMPLATE.as_bytes());
        }
        Ok(Self {
            io,
            lernie,
            lernie_editor,
            lernie_status: String::new(),
            cadence_editor,
            cadence_status: String::new(),
            workflows: Vec::new(),
            new_workflow: String::new(),
            new_model: String::new(),
            new_model_row: String::new(),
            branches: Vec::new(),
            cb_files: Vec::new(),
            cb_name: String::new(),
            cb_origin: EditOrigin::Advance,
            cb_path: String::new(),
            cb_body: String::new(),
            cb_status: String::new(),
            marks: MarksPane::resolve(),
        })
    }

    /// Focus the §11 Config **tab** and re-read everything the pane renders.
    /// The tab focus is the only carrier ([`super::center::focus`], bl-1ca2 —
    /// it was a toggled overlay), and the config files carry no watch root
    /// (§7.1, bl-9130), so this gesture is their freshness: every editor whose draft is pristine follows
    /// disk, an edited one is left as typed. Since §9.5 the same gesture also
    /// asks brazen for its effective provider table — the candidate set every
    /// provider control offers — lists the workflows, and reads the focused
    /// workspace's config lineages, so no frame ever pays for any of them
    /// (§7.2: the config-branch listing used to spawn `for-each-ref` per frame).
    /// brazen's own half of that re-read is [`BrazenPane::open`], because the
    /// file is the focused wall's and this state is the world's.
    pub fn open(&mut self, workspace: Option<&std::path::Path>) {
        let _ = self.lernie_editor.refresh(&self.io);
        let _ = self.cadence_editor.refresh(&self.io);
        self.workflows = self.lernie.workflows(&self.io).unwrap_or_default();
        branch_pane::reread(self, workspace);
    }
}

/// Render the Config tab's content: the three surfaces, scrollable, and — at
/// the foot of the per-workspace surface — the §3.6 danger row (§11's visible
/// carrier for workspace deletion, the settings-danger-zone convention).
pub fn center(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut super::ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    // No `Config` heading: the tab strip directly above already names this
    // surface, and a heading restating its own tab is the same fact twice on
    // one surface (QUALITY H1). What the strip cannot say is the *stance*.
    ui.weak("every setting here edits the file that holds it");
    let rows = row_names(&state.wall.brazen.providers);
    // The brazen pane's roster reference (bl-20cb) asks for a tab rather than
    // taking it: the focus is spent after the scroll area closes, because it is
    // also the target pane's freshness gesture and must not run mid-paint.
    let mut route = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        route = brazen_pane::render(ui, &mut state.wall.brazen);
        ui.separator();
        lernie_pane::render(ui, &mut state.config, &rows);
        ui.separator();
        yog_pane::render(ui, &mut state.config);
        ui.separator();
        config_marks::render(ui, model, &mut state.config.marks, lernie, bl);
        ui.separator();
        branch_pane::render(ui, model, &mut state.config, &rows, lernie, bl);
        super::delete::danger_row(ui, model, state);
    });
    if let Some(tab) = route {
        super::center::focus(model, state, tab);
    }
}
