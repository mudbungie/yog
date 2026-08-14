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
    BUILT_IN_ROWS_HINT, BrazenEditor, BrazenPaths, BzRunner, ProviderRow, RealBzRunner,
};
use crate::keymap::CenterTab;
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

/// The brazen pane's per-wall RAM (§5.3): the editor draft and the one answer
/// the open gesture reads once — brazen's effective provider table (§5.1
/// #20/#21) — plus the seams both go through, folded once from the wall's
/// lensed env.
///
/// **No credential column** (bl-20cb). Presence (§5.1 #22) is the Login
/// surface's fact and this pane no longer paints it, so it no longer reads it:
/// a field kept for a rendering that moved is the drift the roster seat was
/// reseated to end.
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
    }
}

/// The pane: what this file's rows add up to, and the raw TOML draft folded
/// behind it — the §9.5 raw fallback, because `bz` is the only lawful parser of
/// a versionless schema full of open valves, so a form over it would be a
/// second authority corrupting what it cannot model. The tally is what makes
/// the raw text no longer blind: it is the effect the file has, beside the
/// file.
///
/// Returns the §11 tab the operator asked for, if any — the caller spends the
/// focus, because the gesture is the target pane's freshness too
/// ([`super::super::center::focus`]).
pub(crate) fn render(ui: &mut egui::Ui, pane: &mut BrazenPane) -> Option<CenterTab> {
    ui.heading(egui::RichText::new("brazen config.toml").color(theme::integration_hue("bz")));
    if pane.editor.is_none() {
        ui.weak(NO_WALL_HINT);
        return None;
    }
    let route = provider_reference(ui, pane);
    ui.weak(BUILT_IN_ROWS_HINT);
    raw_fold(ui, pane);
    route
}

/// The §9.5 raw fallback, folded away: the TOML draft, its three verbs, and the
/// effective dump when one has been asked for.
///
/// **The header names the file and nothing else** (QUALITY G1). It read *"raw
/// config.toml — validated by bz before it lands"*, which at the documented
/// 420x320 minimum was laid 271 pt into a 194 pt pane and sliced there
/// mid-glyph with no ellipsis. §11 rule 1 cannot reach it — egui lays a
/// `CollapsingHeader`'s own text `Extend` whatever the style says — and it was
/// the widest run this surface laid, so it also set the **content width** of the
/// vertical `ScrollArea` around it and every row beneath then elided against a
/// width the viewport never had (the lernie pane's `models.yaml` path came out
/// truncated at 287 pt and clipped again at 194). Ten provider rows used to sit
/// above it and push both off the bottom of that window, which is the only
/// reason the audit never caught either. The clause it lost is a promise about
/// Apply, so it lives on the hover with the rest of them.
fn raw_fold(ui: &mut egui::Ui, pane: &mut BrazenPane) {
    egui::CollapsingHeader::new("raw config.toml")
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
             model call is routed through. Editing there changes nothing until Apply, \
             and bz validates it before it lands. Reachable by Tab, pressed with \
             Space.",
        );
}

/// What this file's rows add up to, and the one gesture that goes and reads
/// them — **not the roster itself** (bl-20cb, QUALITY H1).
///
/// The pane used to paint every row here: name, credential fact and blocked
/// reason, the identical `row_views` sentences the §8.3 Login pane paints. Ten
/// rows rendered twice is one rendering too many, and this was the copy that
/// could not be acted on — the seat that carries the sign-in verb is the seat
/// that owns the roster, so the roster lives there and this pane references it.
/// The two facts that survive here are the ones the *file* owns and Login does
/// not state: how many rows it ends up routing, and that the table is bz's
/// rather than this file's. What is gone was never this surface's fact —
/// "signed in" is a credential-store answer, and a blocked row's own sentence
/// (*"api-key provider — set the key in Config"*) was pointing at the very pane
/// it was painted in.
///
/// Returns the tab the operator asked for.
fn provider_reference(ui: &mut egui::Ui, pane: &BrazenPane) -> Option<CenterTab> {
    if pane.providers.is_empty() {
        ui.weak("brazen listed no provider rows — reopen the pane to ask again");
        return None;
    }
    // Counted from brazen's own answer, never pinned: the number is what this
    // file plus bz's built-ins come to, which is the whole of what the raw text
    // below is blind about.
    ui.weak(format!(
        "{} provider rows are effective in this workspace — the Login tab names them \
         and states each one's credential",
        pane.providers.len()
    ));
    ui.button(CenterTab::Login.label())
        .on_hover_text(CenterTab::Login.focus_hover())
        .clicked()
        .then_some(CenterTab::Login)
}
