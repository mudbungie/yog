//! What the full window must SHOW (§11) — the three whole-window assertions,
//! split from [`super`] at the §12 line cap on the seam that directory already
//! uses for [`super::screen`]: **the harness is how a frame is run, the tests
//! are what a frame must show.** `super` keeps the shared driver (`input`, the
//! three-frame `painted` settle); this file keeps the claims made through it.
//!
//! The split was forced by two agents at once — bl-bc06's `elision` module and
//! bl-9551's `overlap` module each fit under the cap alone, and together put
//! the parent one line over it. Neither ball's content is the reason, so the
//! seam is the parent's own, not either ball's.

use super::fixture::{self, world};
use super::painted;
use crate::cli_outbound::Cli;
use crate::keymap::{CenterTab, InspectorTab};

#[test]
fn full_window_reaches_every_data_surface() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.state.inspector.step_sel = Some(0);

    // Altitude 0/1 chrome renders regardless of tab: the attention strip, the
    // tab bar's `new` verb and overflow (the fixture workspace is foreign), the
    // conversation list with its row, the balls section + Config entry, the
    // composer (message-targeted — an agent is focused), the activity chip,
    // and the budget-spent header.
    world.model.select_tab(InspectorTab::Transcript);
    let base = painted(&mut world, &litany, &bl);
    assert!(base.contains("need attention"), "attention strip:\n{base}");
    // The strip's jump control sits beside the legend it walks (§11 glyph
    // doctrine, bl-e266); the fixture bears attention, so it paints enabled.
    assert!(
        base.contains("⏭ next"),
        "the jump-to-next-attention control"
    );
    // The mint paints its own galley, so it is the bare line — `contains` would
    // be satisfied by the list's "new conversation" below it.
    assert!(
        base.lines().any(|line| line == "new"),
        "the tab bar's new mint"
    );
    assert!(base.contains('⋯'), "the foreign/replay overflow menu");
    // bl-2d87: the tab row names itself, left of the tabs.
    assert!(
        base.contains("Workspaces:"),
        "the tab bar's own label:\n{base}"
    );
    assert!(
        base.contains("new conversation"),
        "the list's new-conversation affordance"
    );
    assert!(base.contains("balls"), "the collapsible balls section");
    assert!(base.contains("⚙ Config"), "Config entry");
    // The composer NAMES its target (§11, §3.3, bl-2f30). The fixture goal
    // carries no identity stamp, so the ladder lands on rung two — the first
    // payload line — and the raw agent id must not reach the composer at all.
    assert!(
        base.contains("→ message hello"),
        "composer names the selected conversation:\n{base}"
    );
    assert!(
        !base.contains("→ message c-1"),
        "the raw agent id must not reach the composer:\n{base}"
    );
    assert!(base.contains("Message"), "message verb:\n{base}");
    assert!(base.contains("activity · 0 ops"), "activity chip:\n{base}");
    // bl-905f: the bottom in-flight strip is a *conditional panel*, so an
    // at-rest window paints no line and asks for no repaint (§7.2). The
    // fixture's lone agent is Stopped, which is the at-rest case.
    assert!(
        !base.contains("a model call is streaming"),
        "an idle conversation paints no in-flight strip:\n{base}"
    );
    assert!(base.contains("c-1"), "conversation row / header");
    assert!(base.contains("budget"), "budget-spent header");
    // The §11 inline Login affordance on the auth-failed latest step — the Z8
    // machinery rendered in the conversation view, verified on the real
    // kind:auth response shape.
    assert!(
        base.contains("failed on credentials"),
        "auth banner:\n{base}"
    );
    assert!(base.contains("Login (bz browser sign-in)"), "login surface");
    // bl-402f: every provider row states its own state in words — the
    // credential fact, and where bz cannot sign the row in, the reason there is
    // no verb (a dim button that no-ops silently was the defect).
    assert!(
        base.contains("auth none · no credential needed"),
        "a keyless row's credential fact:\n{base}"
    );
    assert!(
        base.contains("keyless — nothing to log in"),
        "and the reason it offers no Login:\n{base}"
    );
    // Transcript tab content.
    assert!(
        base.contains("please ping"),
        "transcript delivered:\n{base}"
    );
    assert!(base.contains("pong reply"), "transcript model reply");

    // Steps tab: the list figures and the selected step's tool drill-in.
    world.model.select_tab(InspectorTab::Steps);
    world.state.inspector.step_tab = crate::steps_view::StepTab::Tools;
    let steps = painted(&mut world, &litany, &bl);
    // bl-3ffc: the step table names its own columns — the figures under them
    // are bare, so the heading is what proves the token column reached paint.
    assert!(steps.contains("Tokens"), "steps token column:\n{steps}");
    assert!(steps.contains("toolu_1"), "steps tool i/o");

    // Inbox tab: the deposit listing beside its own Scan (flush) button
    // (bl-0b4b: moved off the composer, which acts on the conversation, not
    // the workspace's inbox).
    world.model.select_tab(InspectorTab::Inbox);
    let inbox = painted(&mut world, &litany, &bl);
    assert!(inbox.contains("follow-up message"), "inbox deposit");
    assert!(inbox.contains("Scan"), "scan verb:\n{inbox}");

    // Files tab: the agent worktree listing and the selected file's preview.
    world.model.select_tab(InspectorTab::Files);
    world.state.inspector.eph.files_sel = Some("goal.md".to_owned());
    let files = painted(&mut world, &litany, &bl);
    assert!(
        files.contains("goal.md"),
        "files worktree listing:\n{files}"
    );
    assert!(files.contains("hello"), "goal.md preview:\n{files}");

    // Config tab: the governing config frozen label.
    world.model.select_tab(InspectorTab::Config);
    assert!(
        painted(&mut world, &litany, &bl).contains("policy frozen at"),
        "governing config"
    );

    // The Config **tab**: the three surfaces and the workspace's config branch
    // listing. The listing is read at the tab's focus gesture, not per frame
    // (§9.5, bl-ee0a), so the tab is focused through the one gesture every
    // carrier spends (bl-1ca2) rather than by setting a flag beside a read.
    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Config);
    let editors = painted(&mut world, &litany, &bl);
    assert!(
        editors.contains("brazen config.toml"),
        "brazen editor:\n{editors}"
    );
    assert!(
        editors.contains("litany global config"),
        "litany-global editor"
    );
    assert!(
        editors.contains("workspace config branches"),
        "config-branch editor"
    );
    assert!(editors.contains("config/default"), "config branch listing");
}

/// bl-2f30: the composer's target line and its hint are **two spellings of one
/// fact** — the §3.3 display-name ladder over the selection's *conversation
/// root*, never the raw agent id. Here the ladder stops on rung one via the
/// legacy `You are <name>.` goal stamp (a pre-0.0.4 root — the modern carrier
/// is the litany `name` blob); the smoke test above covers rung two off the same
/// composer. That the selection is resolved to its root first (so a descent
/// CHILD still names the conversation it belongs to) is
/// [`crate::nav::convs::root_of`]'s own property, tested there — this fixture
/// prunes its compaction branch, so it holds one lone root.
#[test]
fn the_composer_names_the_conversation_never_the_agent_id() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = fixture::world_titled("You are stench-pug.\n\nfix the gate");
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let out = painted(&mut world, &litany, &bl);
    assert!(
        out.contains("→ message stench-pug"),
        "the stamped name is the target line:\n{out}"
    );
    assert!(
        out.lines().any(|line| line == "message stench-pug"),
        "and the box's hint is the same fact:\n{out}"
    );
    assert!(
        !out.contains("→ message c-1"),
        "no raw agent id in the composer:\n{out}"
    );
}
