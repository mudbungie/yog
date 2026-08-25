//! The selected-conversation center panel (§11 altitude 1). Coverage-excluded
//! glue: the auth-failed detection and every
//! inspector view-model are tested (`AppModel`, `nav::convs`, `login::auth`,
//! `inspector`); this file only wires widgets. The composer docks at the
//! bottom of this pane and the activity accessory at the window's
//! ([`super::input_bar`], [`super::activity`], bl-c038) —
//! yog's own plumbing never renders inline between conversation content.
//!
//! **The header is the identity line and nothing else** (bl-2e18: every
//! setting for a conversation moves to the bottom instead of the top). The
//! name, the when-seat, the live-activity
//! badge and the live mark ride it; every config-shaped row — the §9.4 model
//! line, the §3.5 spend figures — moved to the bottom stack beside the
//! composer ([`super::settings`]), which is also where the birth-config block
//! now answers the same question with an empty selection ([`super::birth`]).
//! **The centre renders no membership list** (bl-8905). It used to carry a
//! compact descent tree — one selectable row per member — which bl-fa82 turned
//! into a second rendering of the conversation list's own unfolded rows, on the
//! same screen, driving the same selection. The centre's membership reading is
//! now the header's live mark alone: one circle per agent, hue = what each is
//! doing, the roster on hover. Selecting a member is the list's gesture.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::nav::convs::Selection;
use crate::theme;

use super::ShellState;

/// Render the center: the selected conversation — the identity header (the §3.3
/// display name, when it started, the live badge and mark — the mark being the
/// centre's one membership reading, one circle per agent, since bl-8905 retired
/// the descent tree that repeated the list),
/// the inline Login banner on an auth-failed latest step, and the Altitude-2
/// inspector (Transcript first). The conversation's config-shaped rows are not
/// here: they are the bottom stack's ([`super::settings`], bl-2e18).
pub fn center(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bz: &Cli,
) {
    let Some(ws) = model.focused_workspace() else {
        super::bootstrap::render(ui, model, state);
        return;
    };
    // "Replay is not a mode": the same view renders it, only the mutating
    // composer is withheld — a replay is read-only (§3.1).
    if model.focused_is_replay() {
        ui.weak("replay · read-only");
    }
    let Some(seat) = super::seat::selection(model) else {
        // Nothing selected is not an empty seat — but what fills it is the
        // birth-config block down in the settings rows (bl-824e, re-seated by
        // bl-2e18), where every config-shaped row now lives.
        ui.weak("select a conversation — or start one below");
        return;
    };
    let agent_id = seat.agent_id.clone();
    // The selection's own detail, over the wire (REMOTE §9.7, bl-48ae): the §6
    // marks are a fact about this one agent rather than about the list, so they
    // ride the standing `Query::Agent` and land an ask period after the name
    // above them. A frame the engine has not answered wears no marks, which is
    // the same honest empty state the transcript below already paints.
    let detail = super::seat::detail(model, &ws, &agent_id).value;
    let agent_marks = detail
        .as_ref()
        .map(|view| view.marks.clone())
        .unwrap_or_default();
    // The live mark's seats ride the same landed answer (bl-296f) — one ask,
    // two facts on it, and no second question for a mark that is about the very
    // conversation the marks above are.
    let live = detail.map(|view| view.seats).unwrap_or_default();
    // The §3.2 workspace balls, off the landed listing (bl-b4b5) — the strip
    // the header paints under the conversation's own ball.
    let ws_balls = super::chrome::focused_balls(model);
    header(ui, &seat, &live, &ws_balls);
    // The §6 marks the *focused* agent wears, said outright — this seat has the
    // room, and it is the one surface a jump-to-attention always lands on. It is
    // what answers "why was I sent here" after arriving acknowledged the signal:
    // the flag is gone, the fact is not (§6, §11 badge-seat pattern).
    for mark in agent_marks {
        let (_, hue, phrase) = theme::mark_badge(mark);
        ui.colored_label(hue, phrase);
    }
    // The focused agent's steps view — the **same standing question** the
    // inspector's own Steps tab declares (REMOTE §9.7, bl-13f9), keyed by its
    // encoded envelope, so two seats reading it are one ask and this pane pays
    // nothing for the tab's. The auth and wound banners below read it every
    // frame and no frame reads disk; a frame the engine has not answered yet
    // banners nothing, which is what an unread step honestly is. A refusal has
    // no seat here either — a banner that is not offered is silence, never a
    // claim — so it is dropped, and the tab beside it is where the engine's
    // sentence is painted.
    let steps = super::inspector::steps(model, &ws, &agent_id)
        .value
        .unwrap_or_default();
    let auth_failed = crate::login::auth::latest_step_auth_failed(&steps);
    let wound = crate::steps_view::latest_wound(&steps);
    // A conversation whose latest step is an auth-shaped failure banners Login
    // inline (§11/§8.3) — the Z8 machinery, one click away where the wound is.
    //
    // The banner names the row when one is derivable (bl-8e34), because "log in
    // below" against brazen's whole table is a question, not a remedy: the pane
    // offers every row and the operator has to know which one the step was
    // dispatched on. With the row named there is nothing left to pick — but the
    // pane still opens beneath, since a *wrong* derivation must never be the
    // only way out.
    if auth_failed.offered() {
        ui.colored_label(theme::ICHOR, auth_failed.banner());
        let state_root = model.state_root().to_path_buf();
        super::login_pane::login_section(ui, &mut state.wall.login, bz, &state_root);
    }
    // A conversation whose driver died before the model said anything banners
    // the §7.3 wound here rather than leaving a `0 attempts · 0 tok` step to
    // read as quiet (bl-7f2e). Since bl-55d8 it carries **the reason in
    // words** — the tail of that step's own `stderr.log`, which is the model
    // adapter's last words (lernie ARCH §2.3) — rather than pointing at the
    // activity trail, which for a turn continued by `lernie message` holds
    // nothing at all: that driver is launched by lernie, so there is no §8.1
    // per-spawn sink for the ops row to fold. The whole sentence is composed
    // in `steps_view::wound` so the words have one home (§11 badge-seat).
    //
    // Through the grace gate (bl-90bf): both halves of the predicate now ride
    // the snapshot (the steps view above is memoized per snapshot, bl-e90a),
    // so a healthy send still reads wounded until the §7.2 poll sees the
    // driver's lock. A wound that heals inside the cadence's grace window
    // never reaches the screen; one that outlives it banners.
    if state.wound_grace.paints(
        &ws,
        &agent_id,
        wound.wounded(),
        model.cadence().wound_grace(),
    ) {
        ui.colored_label(theme::ICHOR, wound.banner());
    }
    // A conversation whose newest transcript entry is owed an answer while
    // nobody holds the driver lock banners the **orphaned-tail** state
    // (bl-ace6, widened by bl-abba, `steps_view::orphan`): the driver that
    // should be answering died before creating a step — an unpaired-tail
    // decline, a lease fault, a crashed launch — or died *inside* a tool
    // window, leaving `tool_use` blocks nobody answered. Either way the
    // wound above has no step to hang on and the ops trail says `exit 0`
    // (the deposit succeeded; the failure was a grandchild lernie
    // launched). The sentence carries the tail of
    // `steps/<agent>/driver.log`, the one copy of that driver's words, and
    // for the tool-window shape it names the one gesture that recovers it —
    // a conversation that looks idle is the whole reason it needs saying.
    // **One banner and one gate for both shapes**: they are the same fact
    // about the same agent in the same seat, so a second grace field and a
    // second `paints` block would be two spellings of one rule. Same grace
    // discipline as the wound: both shapes are laid down under the driver's
    // own lock, so their healthy form is only ever the relaunch gap, and a
    // pair that outlives the grace window is real.
    if state.orphan_grace.paints(
        &ws,
        &agent_id,
        steps.orphan.orphaned(),
        model.cadence().wound_grace(),
    ) {
        ui.colored_label(theme::ICHOR, steps.orphan.banner());
    }
    ui.separator();
    super::inspector::tabs_and_content(ui, model, state, &ws, lernie);
}

/// The conversation header (§11): the identity line — the display name, when it
/// started, the live-activity badge and the live mark — then the conversation's
/// own start-flow ball (§3.3, title/status/badge, coloured by the §3.5 join) and
/// the workspace-level bound balls (§3.2 claimant join — balls the agents picked
/// up, attributed to the workspace, not any one conversation).
///
/// **A binding is not a setting.** Which ball a conversation carries is who it
/// is, so it stays on the identity side of the settings-seat ruling; what those
/// balls have *spent* is a figure, and figures went to the bottom with the rest
/// of the config-shaped rows ([`super::settings`], bl-2e18).
fn header(
    ui: &mut egui::Ui,
    seat: &Selection,
    live: &[crate::nav::convs::Seat],
    ws_balls: &[crate::nav::BoundBall],
) {
    // §11 altitude 1: the id is the identifier, the name is the title — one
    // ladder (§3.3), the same one the §11 row and the §3.6 dialog read. The
    // live-activity indicator sits on that same line: the open conversation's
    // name is where the eye already rests, so the pane says what the list row
    // says, in the same seat, off the same derivation (§5.1 #28).
    let root = seat.root.as_str();
    ui.horizontal(|ui| {
        let heading = ui.heading(seat.name.clone());
        if seat.display_only {
            // The headline is the legacy §3.3 rung — prose no lernie name
            // fact backs — so the pane says here what the list row hovers:
            // this name is not a message target (bl-8068).
            heading.on_hover_text(theme::NAME_DISPLAY_ONLY);
        }
        // The id's stamp, read out for a human (bl-16da): the headline seat is
        // "when did this start", and the raw id — the branch name and on-disk
        // key — hovers behind it. Derived at render; the id is the storage.
        let started = crate::nav::convs::started_at(root);
        ui.weak(started.label).on_hover_text(started.hover);
        // Unlike the width-bound list row, this seat has the room — so per the
        // §11 badge-seat pattern it states the class outright rather than
        // hovering it. Only a conversation actually working asks for a repaint.
        //
        // **And it is the surface's ONE seat for those words** (bl-3f70,
        // QUALITY H1): the bottom in-flight strip printed the identical
        // sentence in the same hue two lines below, and the words stayed here
        // because this row is unconditional while the strip is a §11 rule 5
        // share the budget can decline, and because this line has the width the
        // strip's has not. The strip keeps what only it can say — the live
        // characteristics — and hovers the sentence.
        if let Some(class) = seat.flight {
            let (glyph, hue, says) = theme::flight_badge(class);
            let time = ui.ctx().input(|i| i.time);
            ui.colored_label(theme::pulse(hue, time), format!("{glyph} {says}"));
            ui.ctx().request_repaint_after(theme::PULSE_REPAINT_DELAY);
        }
        // The live mark, on the row it is a fact about (bl-d44e): one circle per
        // agent in this conversation's subtree, hue = what it is doing (§5.1
        // #28b). Right-aligned, so it holds the pane's own corner rather than
        // the window's — a conversation-scoped fact never belonged in the
        // altitude-0 chrome, which is about workspaces and totals. The badge
        // beside it says the subtree's ONE class in words; the mark says every
        // agent's own state at a glance. Different questions, one row.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            theme::live_mark(ui, live);
        });
    });
    // The per-conversation ball (source 1: the goal stamp), its title and
    // status — the selection's own field off the landed forest (bl-296f), which
    // is the row the §11 list is already painting rather than a second read of
    // the same stamp.
    if let Some(ball) = &seat.ball {
        super::conv_ball::header_ball(ui, ball);
    }
    // Workspace-level balls (§3.2): the claimant join, all the balls this
    // workspace's agents bound — the "one or more" per workspace, distinct from
    // the single per-conversation goal stamp above.
    if !ws_balls.is_empty() {
        ui.horizontal(|ui| {
            ui.weak("workspace balls:");
            for ball in ws_balls {
                match &ball.badge {
                    Some(b) => ui.weak(format!("{} · {b}", ball.id)),
                    None => ui.weak(&ball.id),
                };
            }
        });
    }
}
