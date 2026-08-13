//! brazen's `config.toml` (§9.1) — the one Config surface that belongs to a
//! **workspace** rather than to the world (§16.2 as amended by the
//! blast-radius ruling): the raw draft, the effective provider table it
//! produces, and the credential column beside it.
//!
//! **Its RAM is the wall's, not the window's** (bl-5894, §5.3's "RAM, but per
//! target, not per box"). The whole pane — draft, status line, dumped effective
//! config, provider rows, credential presence — is one field of
//! [`WallRam`](crate::shell::WallRam), parked under the wall it was typed in
//! when focus moves and taken back out when focus returns. So an unsaved draft
//! survives A → B → A, and no row read under A can paint or be applied under B.
//! It used to be *re-loaded in place* on every focus change, which threw the
//! draft away; keeping one box and re-lensing it is exactly the shape this file
//! exists to make impossible.

use crate::config_edit::RealFileIo;
use crate::config_edit::brazen::{
    BUILT_IN_ROWS_HINT, BrazenEditor, BrazenPaths, BzRunner, ProviderRow, RealBzRunner, row_views,
};
use crate::theme;
use crate::xdg::Env;

use super::RELOAD_HINT;
use super::status::{describe_applied, reload_status, status_line};

/// What the pane says with no workspace in focus (§16.2 as amended): providers,
/// sign-ins and the model cache are a workspace's own settings, so there is
/// nothing to edit until a sphere is chosen — and nothing ambient to fall back
/// to.
const NO_WALL_HINT: &str = "focus a workspace to edit its providers — brazen's config, sign-ins and \
     model cache live inside a workspace, not on the machine";

/// The brazen pane's per-wall RAM (§5.3): the editor draft and the two answers
/// the open gesture reads once — brazen's effective provider table and the
/// credential presence beside it (§5.1 #20–#22) — plus the seams both go
/// through, folded once from the wall's lensed env.
pub struct BrazenPane {
    io: RealFileIo,
    bz: RealBzRunner,
    /// This wall's editor — `None` outside any workspace, where the pane has no
    /// file to edit and says so.
    pub(crate) editor: Option<BrazenEditor>,
    pub(crate) status: String,
    effective: String,
    /// The rows every provider control in the Config tab offers, so what the
    /// pickers know is on screen.
    pub(crate) providers: Vec<ProviderRow>,
    creds: Vec<(String, bool)>,
}

impl BrazenPane {
    /// Fold the editor and the `bz` runner from a **wall's** lensed env. A
    /// missing `config.toml` loads as an empty draft (§9.1), never an error, and
    /// an unexpected io error is the same emptiness — the pane is a draft over a
    /// file that may not exist yet, so there is nothing here a `Result` could
    /// tell the frame that the empty draft does not.
    pub fn new(wall: &Env) -> Self {
        let io = RealFileIo;
        Self {
            io,
            bz: RealBzRunner::resolve(wall),
            editor: BrazenPaths::of(wall).and_then(|paths| BrazenEditor::load(paths, &io).ok()),
            status: String::new(),
            effective: String::new(),
            providers: Vec::new(),
            creds: Vec::new(),
        }
    }

    /// Re-read everything the pane renders — the gesture that focuses the Config
    /// tab (§9's freshness rule, bl-1ca2). A pristine draft follows disk, an
    /// edited one is left as typed.
    pub(crate) fn open(&mut self) {
        if let Some(editor) = self.editor.as_mut() {
            let _ = editor.refresh(&self.io);
        }
        self.providers = self.bz.providers();
        self.creds = self
            .editor
            .as_ref()
            .map(|e| e.credential_presence(&self.providers, &self.io))
            .unwrap_or_default();
    }
}

/// The pane: the effective provider table as read-only rows, and the raw TOML
/// draft folded behind them — the §9.5 raw fallback, because `bz` is the only
/// lawful parser of a versionless schema full of open valves, so a form over it
/// would be a second authority corrupting what it cannot model. The rows are
/// what makes the raw text no longer blind: they are the facts the file
/// produces, beside the file.
pub(crate) fn render(ui: &mut egui::Ui, pane: &mut BrazenPane) {
    ui.heading(egui::RichText::new("brazen config.toml").color(theme::integration_hue("bz")));
    if pane.editor.is_none() {
        ui.weak(NO_WALL_HINT);
        return;
    }
    provider_table(ui, pane);
    ui.weak(BUILT_IN_ROWS_HINT);
    egui::CollapsingHeader::new("raw config.toml — validated by bz before it lands")
        .show(ui, |ui| {
            let Some(editor) = pane.editor.as_mut() else {
                return;
            };
            super::form_ui::raw_editor(ui, editor.draft_mut());
            ui.horizontal(|ui| {
                if ui
                    .button("Apply")
                    .on_hover_text(
                        "Hand this TOML to bz to validate, and write it to config.toml \
                         only if bz accepts it. A file changed underneath you is \
                         refused, not overwritten. Typed, it is `/config brazen \
                         <text…>`.",
                    )
                    .clicked()
                {
                    pane.status = describe_applied(editor.apply(&pane.bz, &pane.io));
                }
                if ui.button("Reload").on_hover_text(RELOAD_HINT).clicked() {
                    pane.status = reload_status(editor.reload(&pane.io));
                }
                if ui
                    .button("Effective (bz --dump-config)")
                    .on_hover_text(
                        "Ask bz what configuration it actually ends up with once every \
                         layer and built-in is folded in, and print it below. Reachable \
                         by Tab, pressed with Space.",
                    )
                    .clicked()
                {
                    pane.effective = editor.effective(&pane.bz).stdout;
                }
            });
            status_line(ui, &pane.status);
            if !pane.effective.is_empty() {
                ui.monospace(&pane.effective);
            }
        })
        .header_response
        .on_hover_text(
            "Open brazen's config.toml raw — the providers and credentials every \
             model call is routed through. Editing there changes nothing until Apply. \
             Reachable by Tab, pressed with Space.",
        );
}

/// brazen's effective rows (§5.1 #20–#22), read at the open gesture: the row
/// name, its credential fact in words — which its credential model phrases, and
/// which is also its login capability (§8.3) — and, where `bz --login` cannot
/// serve it, why. The words are [`row_views`], the same derivation the §8.3
/// Login pane renders (bl-402f): one fact, two seats.
fn provider_table(ui: &mut egui::Ui, pane: &BrazenPane) {
    if pane.providers.is_empty() {
        ui.weak("brazen listed no provider rows — reopen the pane to ask again");
        return;
    }
    for row in row_views(&pane.providers, &pane.creds) {
        ui.horizontal(|ui| {
            super::super::row::bounded(ui);
            ui.monospace(&row.name);
            ui.weak(&row.fact);
        });
        if let Some(why) = &row.blocked {
            // Its own **wrapped** line, not a third element on the row above
            // (bl-5410). Three greedy runs on one line overflowed it, and an
            // overflowing row in a vertical `ScrollArea` is a *ratchet*: the
            // content it could not fit becomes the content width egui lays the
            // next frame at, so the row spilled 15 pt further past the pane on
            // every frame until it settled 74 pt outside it — every label on it
            // then truncated against a width the pane never had, and was clipped
            // mid-glyph with the `…` it had earned sitting off the edge.
            ui.scope(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                ui.colored_label(theme::ASH, why);
            });
        }
    }
}
