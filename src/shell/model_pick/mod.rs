//! The §9.4 model picker's widgets — coverage-excluded glue like the rest of
//! `src/shell/*`. Everything it decides is tested elsewhere: the block grammar
//! and the one-pick [`plan`](crate::model_pick::plan), the roster query's
//! settle arms, the [`fault`](crate::model_pick::grammar::fault) sentence a
//! dead role row paints, the row the settings seat wears
//! ([`crate::model_pick::header`]), and the two write pipelines in `config_edit`
//! (§9.2 provider gate + hash-guard + rename, §9.3 staged `lernie config`
//! drive).
//!
//! This wires them to widgets: [`seat`] paints the row both surfaces carry,
//! [`select`] holds the two brazen-sourced dropdowns and the role strip,
//! [`lines`] the derivation behind the row, [`ram`] the cross-frame state and
//! [`write`] the write half.
//!
//! **The row IS the picker's pair control** (bl-cd2a). The ruling: the model
//! selection in the conversation window carries both dropdowns, provider and
//! model, and the whole line becomes `<provider> - <model>` and nothing else.
//! So the two dropdowns are not
//! behind a *change…* button and are not duplicated on the row: they *are* the
//! row, always painted, and the pane keeps only what a row cannot hold — the
//! role strip that re-scopes them, the fault a dead assignment earns, the write
//! receipt, and the two routes out. One pair of dropdowns in the app, one state,
//! one write path.
//!
//! The pane has **no buttons at all** (bl-fb6b): choosing a model is the write,
//! and the role strip is the scope it writes to — so there is neither a Set
//! button to forget nor a per-role apply to pick the wrong one of.
//!
//! **The roster is asked when the model list is opened**, not when a surface
//! appears (bl-cd2a amends §9.4's "every time the picker is triggered"): the
//! dropdowns are on screen from the moment a conversation is, and a control that
//! fired a provider query on sight would spawn one per glance. Opening the list
//! is the trigger, and it still survives nothing — no candidate set outlives a
//! close, nor a role change onto another provider row. The two facts read
//! *about* the current assignment — brazen's provider rows and the global
//! `models.yaml` — are asked once per open of the pane and discarded with it
//! (§5.3), because they answer "is what you already have usable?", a question
//! only asked while the pane is on screen.
//!
//! **Two seats, one row** (bl-824e): the open conversation's settings rows and
//! the §11 birth-config block — since bl-2e18 the same bottom seat, one branch
//! on the selection apart. Only the scope claim differs, so [`seat`] takes that
//! sentence rather than deriving it: [`conversation_scope`] names a conversation
//! already frozen, [`birth_scope`] one not started yet. A second picker would be
//! a second authority on the same two files.

use crate::cli_outbound::Cli;
use crate::config_edit::branch::config_file;
use crate::config_edit::brazen::ProviderRow;
use crate::keymap::CenterTab;
use crate::model_pick::query::{self, RosterView};
use crate::model_pick::{
    BRANCH, ModelRow, PROVIDERS, WRITE_NOTE, birth_sentence, grammar, remedy, scope_sentence,
};
use crate::shell::ShellState;
use crate::theme;
use std::path::Path;

pub(super) mod lines;
mod marks;
mod ram;
/// The §9.4 role strip — the *scope* over the pair row, split from
/// [`select`] at §12's cap on the seam that file's own doc draws (bl-dd7f).
mod role;
mod seat;
mod select;
mod write;

/// One role row as painted: the assignment plus why its model is unusable, or
/// `None` when it is fine (§9.2's judgement, surfaced at the point of choice).
type Marked = (grammar::RoleModel, Option<String>);

pub use ram::PickerState;
pub(crate) use seat::{birth_seat, conversation_seat};

/// The workspace's leaf name — what both scope sentences call the blast radius.
fn leaf_of(ws: &Path) -> String {
    ws.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// The scope a pick claims when the picker was opened from an open
/// conversation's model line (§9.4): the branch moves, this conversation does
/// not.
pub(crate) fn conversation_scope(ws: &Path, frozen_oid: &str) -> String {
    scope_sentence(&leaf_of(ws), BRANCH, frozen_oid)
}

/// The scope a pick claims when the picker was opened from the §11 birth-config
/// block: the same branch write, said in the terms the birth surface has —
/// there is no frozen conversation to exempt, and the workspace default moves
/// (bl-824e; lernie 0.0.3 offers no per-conversation config).
pub(crate) fn birth_scope(ws: &Path) -> String {
    birth_sentence(&leaf_of(ws), BRANCH)
}

/// The picker pane (§9.4) — since bl-cd2a, only what the row cannot hold: the
/// role strip that re-scopes the row's two dropdowns, the fault a dead
/// assignment earns, and the scope the write claims. Returns the §11 tab the
/// operator asked to be taken to (the §9.1 brazen editor); the caller owns
/// surface routing, so the pane names the request rather than performing it.
pub(super) fn pane(
    ui: &mut egui::Ui,
    picker: &mut PickerState,
    ws: &Path,
    scope: &str,
    role: &str,
) -> Option<CenterTab> {
    if !picker.open {
        return None;
    }
    ui.separator();
    ui.strong("Model");
    ui.weak(scope);
    ui.weak(WRITE_NOTE);
    let tip = format!("config/{BRANCH}");
    let text = picker.tip_providers.clone().unwrap_or_else(|| {
        let raw = config_file(ws, &tip, PROVIDERS).unwrap_or_default();
        let text = String::from_utf8_lossy(&raw).into_owned();
        picker.tip_providers = Some(text.clone());
        text
    });
    let marked = marks::mark_roles(picker, &text);
    if marked.is_empty() {
        ui.colored_label(
            theme::ICHOR,
            format!("cannot read `roles:` from {tip}:{PROVIDERS} — use the Config editors"),
        );
        return None;
    }
    role::select_role(ui, picker, &marked);
    let (_, fault) = marked.iter().find(|(r, _)| r.role == role)?;
    if let Some(why) = fault {
        ui.colored_label(theme::ICHOR, format!("⚠ {why}"));
    }
    None
}

/// Fire the roster query when there is none, or when the selected role moved to
/// a different provider row — the §9.4 "every time the picker is triggered".
pub(super) fn refresh(picker: &mut PickerState, provider: &str, bz: &Cli) {
    let stale = picker
        .roster
        .as_ref()
        .is_none_or(|r| r.provider() != provider);
    if stale {
        picker.roster = Some(query::start(&bz.and_env(picker.wall.clone()), provider));
    }
}

/// Poll the roster and hand back its **settled** view. `None` while no query has
/// been fired or one is still in flight — the row paints the pulse inside the
/// open list rather than beside it (bl-cd2a), because that is the only place the
/// operator is waiting for it. A *failed* roster settles to a view with an empty
/// model list, so the list's custom-id entry stays reachable and a provider that
/// cannot be listed is not a dead end.
pub(super) fn settled(picker: &mut PickerState) -> Option<RosterView> {
    let roster = picker.roster.as_mut()?;
    roster.poll();
    let view = roster.view();
    (!view.in_flight).then_some(view)
}

/// Paint a settled roster's failure and, when it is auth-shaped, the way out of
/// it (bl-91f1): what this row needs in yog's own words and the control that
/// goes and does it. Returns the §11 tab the operator asked for.
///
/// The three layers are deliberate. brazen's sentence stays **verbatim** on top
/// (INV-2 / §7.3 — a failure renders as itself), the run-by-hand command stays
/// **underneath** (§8.3's fallback grammar), and the remedy sits between them:
/// additive, so §8.3 rule 5's "a wrong derivation must never become the only
/// way out" holds here too. `row` is `None` only where the selected provider is
/// not in brazen's table at all, which is the one case with nothing to say
/// about its credentials.
pub(super) fn roster_fault(
    ui: &mut egui::Ui,
    view: &RosterView,
    row: Option<&ProviderRow>,
) -> Option<CenterTab> {
    let error = view.error.as_ref()?;
    ui.colored_label(theme::ICHOR, format!("⚠ {error}"));
    let mut asked = None;
    if let Some(remedy) = row.and_then(|row| remedy(row, error)) {
        // The reason is painted, never hidden behind the button's hover:
        // bl-402f's finding was that a control whose reason needs a mouseover
        // is the mystery no-op. The hover says what pressing it *does*.
        if let Some(reason) = &remedy.reason {
            ui.weak(reason);
        }
        if ui
            .button(&remedy.verb)
            .on_hover_text(remedy.tab.focus_hover())
            .clicked()
        {
            asked = Some(remedy.tab);
        }
    }
    if let Some(command) = &view.fallback {
        ui.small("or run by hand:");
        ui.code(command);
    }
    asked
}
