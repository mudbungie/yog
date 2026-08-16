//! **The wireless window's one surface** (bl-dc14; REMOTE §1.2, §8): what a
//! window paints when its engine got no wire up — a failed bind, a mint the
//! box cannot perform, a seat the material cannot open.
//!
//! Every read and every act crosses the wire since REMOTE §1.2 was executed
//! (bl-ae05), so a shell painted without one is a set of controls that only
//! *look* actionable — the inert window bl-dc14 refuses. And §8 already rules
//! that a terminal instruction in front of a desktop launch is not an answer,
//! so the refusal has to be paintable: this surface is that answer, rendered
//! INSTEAD of the shell, with the engine's own sentence verbatim (INV-2 — the
//! cause on top, the remedy beneath).
//!
//! Coverage-excluded egui glue like its siblings; the fact it renders
//! ([`AppModel::wire_refusal`]) is derived and covered on the model, and the
//! acceptance suite proves this surface reaches the paint layer
//! (`acceptance::refusal`).

/// The gap between the wordmark block and the refusal block — the same air
/// [`super::bootstrap`] puts between two blocks that are not one sentence.
const APART: f32 = 12.0;

/// What the operator can do about it, painted under the engine's own words.
/// The port-zero spelling is stated outright because the commonest cause is
/// exactly that: an `address` naming a fixed port another running yog holds.
const REMEDY: &str = "yog opened without its engine connection: every read and act crosses that \
     wire, so no control is offered instead of controls that would do nothing. \
     Fix the cause above and relaunch. If the address is a fixed port another \
     running yog holds, point the world's wire address at 127.0.0.1:0 — each \
     engine then binds its own kernel-chosen port.";

/// Paint the refusal INSTEAD of the shell: the wordmark, the one sentence every
/// wireless act receipt already carries ([`crate::wire::post::NO_WIRE`]), the
/// engine's own reason verbatim, and the remedy. No composer, no tabs, no
/// roster — nothing that looks actionable is offered (bl-dc14).
pub(super) fn render(ctx: &egui::Context, reason: &str) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            crate::theme::wordmark(ui);
            ui.weak(crate::theme::TAGLINE);
            ui.add_space(APART);
            ui.colored_label(crate::theme::ICHOR, crate::wire::post::NO_WIRE);
            ui.colored_label(crate::theme::ICHOR, reason);
            ui.add_space(APART);
            ui.weak(REMEDY);
        });
    });
}
