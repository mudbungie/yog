//! The §8.3 **Login** surface: brazen's browser sign-in flow.
//!
//! **It is a tab focus, not a fold in the roster** (bl-1ca2). It used to be a `ui.collapsing` inside the left
//! panel, where opening it put ten provider rows and a live command stream
//! into a column sized for conversation titles. It is now one of the §11
//! center tabs ([`super::center`]), reached by the left-panel entry that names
//! it, by the strip, or by Command+Shift+3 — and the auth-failed banner in the
//! conversation still renders the same [`login_section`] inline where the
//! wound is (§11), which is one machinery in two seats, not two surfaces.
//!
//! Coverage-excluded egui glue (the `src/shell/*` precedent, §12). Its one
//! mutating affordance is Login (§8.3 as amended, bz's sole interactive
//! surface): a per-provider button spawns
//! `bz --login --provider <row> --browser` through the streamed-piped class
//! ([`crate::login`]) and paints its lines live — **stderr included**, which is
//! where bz writes the authorize URL and any failure's reason and remedy —
//! converging to an outcome (and, on a non-zero exit, the exact run-by-hand
//! fallback).
//!
//! **Every row states its own state in words** (bl-402f): its credential fact
//! ("signed in" / "not signed in" / "no credential needed" / "no credential
//! stored") beside the name, and then either the Login verb or — for a row
//! brazen's table says cannot be signed in — the reason there is none
//! ("keyless — nothing to log in", "api-key provider — set the key in Config").
//! A dead button hiding its reason behind a hover was the defect; a row that
//! could only exit 78 now renders no verb at all. Both facts come from
//! [`ProviderRowView`] — the same derivation the §9.5 config rows render, so
//! the two surfaces cannot say different things about one provider.
//!
//! Everything it calls — the row derivation and its capability read, [`LoginRun`]
//! poll/finalize — is covered; only these widgets are not.
//!
//! *Was the toolchain pane* until §16.7 W13 deleted the phase-1 capability gate
//! it fronted: with the substrates embedded as exact-pinned crates there is no
//! host-tool verdict to render and no install command to show (§16.4), so what
//! remains of the pane is exactly the Login surface it always also carried.

use std::path::Path;

use crate::cli_outbound::Cli;
use crate::config_edit::brazen::ProviderRowView;
use crate::login::{self, LoginRun};
use crate::theme;

use super::LoginHolder;

/// The Login surface (§8.3): offer a Login per provider row, and paint the
/// active streamed run. The rows are brazen's effective provider table — read
/// **in-process** through the linked crate since §16.7 W10 (#20/#21, built-ins
/// included) — and they are already there: [`LoginHolder::new`] asks at
/// construction (bl-e290), so this surface opens populated and `↻ providers`
/// is a **re-**ask for after a config edit, not the way rows first appear.
/// `pub(super)`: the §11 Login tab and the conversation center's auth-failed
/// banner both paint it (§11) — one machinery, two seats.
pub(super) fn login_section(
    ui: &mut egui::Ui,
    login: &mut LoginHolder,
    bz: &Cli,
    state_root: &Path,
) {
    ui.label("Login (bz browser sign-in)");
    if ui
        .button("↻ providers + credentials")
        .on_hover_text(
            "Re-read brazen's provider table and credential rows, so a row you just \
             added in the config editor appears here. Nothing is signed in or out. No \
             key of its own: Tab reaches it, Space presses it.",
        )
        .clicked()
    {
        login.ask();
    }
    // An empty roster names the paved path (QUALITY H2). It earns a line now
    // that this is a whole tab rather than a fold in the roster: a surface the
    // operator focused on purpose must never answer with a blank, and "there
    // are no rows" has two remedies — re-ask, or author one in the Config tab.
    if login.rows.is_empty() {
        ui.weak(
            "brazen listed no provider rows — ↻ asks it again, and a row is authored in \
             the Config tab's brazen config.toml",
        );
    }
    // Collect the clicked provider first — the row loop borrows `rows`
    // immutably, the spawn writes `run` mutably.
    let mut chosen = None;
    for row in &login.rows {
        if provider_row(ui, row) {
            chosen = Some(row.name.clone());
        }
    }
    // The sign-in is fired inside the focused workspace's wall (§16.2 as
    // amended), so the credential brazen writes lands in that sphere and
    // nowhere else — signing in here never signs in another workspace.
    if let Some(provider) = chosen
        && let Ok(run) = login::start(
            &bz.and_env(login.wall.clone()),
            &provider,
            state_root,
            &super::now_ts(),
            login.workspace.as_deref(),
        )
    {
        login.run = Some(run);
    }
    if let Some(run) = login.run.as_mut() {
        run.poll();
        render_run(ui, run);
    }
}

/// One provider row: its name, its credential fact in words, and then the Login
/// verb — but **only** where `bz --login` can serve the row (§8.3 as amended by
/// bl-402f). Everywhere else the verb is replaced by the reason there is none,
/// in the same place the button would have been: a row is never a dim shape the
/// operator has to click to learn about. Returns whether Login was clicked.
///
/// The verb is laid **first** (§11 rule 1b, [`super::row::control_last`]): name
/// and fact are greedy text, and laid before the button they ate the row and
/// left `[ … ]` in place of Login — on `claude-session-direct`, the longest
/// name brazen's table carries, which is exactly the row an operator who is not
/// signed in has to press (bl-bc06).
///
/// **The trailing slot holds a control, never prose** (bl-5410). The blocked
/// branch used to pin the reason there, and a sentence pinned at its natural
/// width is not a control: at 420x320 the reason alone (221 pt) was wider than
/// the 194 pt pane, so the row was allocated from the right edge *leftwards past
/// its own left edge* and the whole row — name, fact and reason — was clipped
/// on the left, mid-glyph, with no ellipsis anywhere. Rule 1b buys the verb by
/// pinning it; it cannot buy a paragraph. So the reason takes the line beneath
/// the row and **wraps** there, where it is bounded by the pane on one axis and
/// free on the other, and nothing is cut at any width.
pub(super) fn provider_row(ui: &mut egui::Ui, row: &ProviderRowView) -> bool {
    let Some(why) = row.blocked.as_deref() else {
        return super::row::control_last(
            ui,
            |ui| {
                ui.monospace(&row.name);
                ui.weak(&row.fact);
            },
            |ui| {
                ui.button("Login")
                    .on_hover_text(
                        "Sign in to this provider: opens your browser to authorize it, and \
                         prints everything it says — the URL included — below. No key of \
                         its own: Tab reaches the row, Space presses it.",
                    )
                    .clicked()
            },
        )
        .1;
    };
    ui.horizontal(|ui| {
        // Stated here, not inherited (bl-bc06's rule for [`super::row`]): these
        // rows also paint inline in the conversation's auth-failed banner, and
        // a row that only elides where its seat happens to say so is a row that
        // extends wherever it does not.
        super::row::bounded(ui);
        ui.monospace(&row.name);
        ui.weak(&row.fact);
    });
    // Wrapped, not truncated: this is the one thing on the row that has to be
    // read in full — it is the whole answer to "why is there no button here" —
    // and the panel root's rule-1 `Truncate` would cut it (§11 rule 1's own
    // carve-out: prose that must be read whole says so locally).
    ui.scope(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.colored_label(theme::ASH, why);
    });
    false
}

/// Paint the active login run: every line bz printed — **both** streams, in
/// arrival order, verbatim (§8.3); bz's authorize URL and its terminal
/// error/remedy line both ride stderr, so both land here. Then, once settled,
/// the outcome and, on a non-zero exit, the run-by-hand fallback (§8.3).
fn render_run(ui: &mut egui::Ui, run: &LoginRun) {
    let view = run.view();
    for line in &view.lines {
        ui.monospace(&line.text);
    }
    if let Some(exit) = view.outcome {
        let color = if exit == 0 {
            theme::HYDRA
        } else {
            theme::ICHOR
        };
        ui.colored_label(color, format!("login exited {exit}"));
        if let Some(command) = &view.fallback {
            ui.small("or run by hand:");
            ui.code(command);
        }
    }
}
