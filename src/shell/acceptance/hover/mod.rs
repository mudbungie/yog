//! The §11 **discoverability invariant**: every interactive control says what
//! pressing it does, in operator terms, on hover.
//!
//! A shipped window prompted the question *what does the Scan button mean?*,
//! and the answer existed only in DESIGN. A per-button fix would have left the next control equally
//! mute, so the rule is machine-held here, in halves that check different
//! things:
//!
//! - [`live`] asks the invariant of the **running window** (bl-8e7a): it walks
//!   §11's focus floor around every surface the tab strips can show and
//!   requires each click-sensing `Response` to have opened a tooltip. It names
//!   no constructor at all — the set is whatever the window painted — so a
//!   widget built by a call nobody listed is judged the day it ships. It sees
//!   only what a fixture can reach, which is the half below.
//! - [`every_interactive_control_carries_a_hover`] is the same invariant read
//!   off the tree's own source: it finds every widget constructor that yields
//!   something the operator can press, type into or toggle, and walks the
//!   method chain hanging off it for an `on_hover_text`. A control shipped
//!   without one fails here even when no fixture can reach its seat — a
//!   provider Login row, a workflow file button. Its reach is
//!   the whole tree and its vocabulary is [`CONTROLS`], hand-listed, which is
//!   the blind spot [`live`] covers wherever the window can be driven.
//! - [`spelling`] is the same reading held to §11's *other* half: a control
//!   also says **how to press it without the mouse**, and the vocabulary it may
//!   say that in is derived from the binding table and the §8.5 verb roster
//!   rather than restated (bl-478d).
//! - [`the_hovers_reach_the_paint_layer`] is the wiring proof, in the
//!   established paint-layer idiom (bl-2d87): egui's `everything_is_visible`
//!   makes every tooltip paint unconditionally, so a hover hung on the wrong
//!   response — a neighbouring label rather than the button — never reaches the
//!   galleys and is caught.

pub(super) mod lex;
mod live;
pub(super) mod scan;
mod spelling;

use super::super::render;
use crate::cli_outbound::Cli;
use lex::skeleton;
use scan::{chain_of, rust_files, sites};
use std::path::Path;

/// Widget constructors whose product the operator interacts with. `ui.add(…)`
/// is on the list because that is how this tree builds a `TextEdit` and a
/// click-sensing `Label`; the builder entries (`ComboBox`, `CollapsingHeader`)
/// are matched at their `new`, since the chain walk below carries the hover
/// requirement all the way through `.show(…)` to `.response`.
///
/// **A hand-listed vocabulary, and it is the weak half** (bl-8e7a): a control
/// built by a call absent from here is not judged mute, it is not judged — the
/// bl-45c7 shape. It stays because it is what reads seats no fixture reaches,
/// and it is no longer alone: [`live`] judges the same rule off behavior, so a
/// new constructor in a seat the window can be driven to goes red whatever it
/// is spelled. What is left uncovered is precisely a new spelling in a seat no
/// drive reaches.
pub(super) const CONTROLS: &[&str] = &[
    "ui.button(",
    "ui.small_button(",
    "ui.checkbox(",
    "ui.selectable_label(",
    "ui.selectable_value(",
    "ui.radio_value(",
    "ui.toggle_value(",
    "ui.menu_button(",
    "ui.text_edit_singleline(",
    "ui.text_edit_multiline(",
    "ui.collapsing(",
    "ui.add(",
    "ui.add_enabled(",
    "ui.add_sized(",
    "ComboBox::from_id_salt(",
    "ComboBox::from_label(",
    "CollapsingHeader::new(",
];

/// Either seat of the same obligation: a live control states its job, a
/// disabled one states why it cannot do it (bl-e266 — an inert control that
/// says nothing is the mystery no-op).
pub(super) const HOVERS: &[&str] = &["on_hover_text", "on_disabled_hover_text"];

/// **The invariant.** Every control the tree paints carries a hover, and no
/// hover is the empty string. Enumerating nothing is itself a failure — the
/// same two-direction discipline `make rules-audit` keeps, so a rotted pattern
/// list cannot pass by matching zero call sites.
#[test]
fn every_interactive_control_carries_a_hover() {
    let mut mute = Vec::new();
    let mut seen = 0usize;
    for file in rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src")) {
        let source = std::fs::read_to_string(&file).unwrap();
        let skeleton = skeleton(&source);
        for hover in HOVERS {
            assert!(
                !skeleton.contains(&format!("{hover}(\"\")")),
                "{}: an empty hover string says nothing",
                file.display()
            );
        }
        for (at, control) in sites(&skeleton, CONTROLS) {
            seen += 1;
            if !chain_of(&skeleton, at)
                .iter()
                .any(|(m, _)| HOVERS.contains(&m.as_str()))
            {
                mute.push(format!("{}: {control}", file.display()));
            }
        }
    }
    assert!(
        seen > 50,
        "the scan matched {seen} controls — the pattern list has rotted"
    );
    assert!(
        mute.is_empty(),
        "these controls say nothing on hover — §11 requires every one of them to \
         state what pressing it does:\n{}",
        mute.join("\n")
    );
}

/// The paint-layer half (bl-2d87's idiom): with every tooltip forced visible,
/// the hovers of the controls this fixture actually paints reach the galleys.
/// A hover attached to the label beside a button rather than the button itself
/// compiles, reads fine, and fails here.
#[test]
fn the_hovers_reach_the_paint_layer() {
    let (litany, bl, bz) = (Cli::new("litany"), Cli::new("bl"), Cli::new("bz"));
    let mut world = super::fixture::world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.state.activity_open = true;
    // A draft in the box, so Message paints live rather than refused — both
    // seats of the enablement pair are then covered across this test. The
    // draft is the target's, not the box's (bl-a69a).
    world.state.actions.drafts.set(
        crate::actions::DraftKey::Message("c-1".to_owned()),
        "hello".to_owned(),
    );
    // Scan lives on the Inbox tab (it flushes the workspace's inbox, not the
    // composer's target), so that is the tab this fixture paints — the Raw
    // toggle and the tab strip's own hovers ride along.
    world.model.select_tab(crate::keymap::InspectorTab::Inbox);
    let ctx = egui::Context::default();
    ctx.memory_mut(|m| m.set_everything_is_visible(true));
    let frame = |world: &mut super::fixture::World| {
        let out = ctx.run(super::input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
        });
        crate::paint_probe::text_of(&out)
    };
    // The wire settled between frames (bl-44e9): a hover on a row of a migrated
    // list only exists once that list's answer has landed.
    frame(&mut world);
    world.settle();
    frame(&mut world);
    world.settle();
    frame(&mut world);
    world.settle();
    let painted = frame(&mut world);
    for phrase in [
        // The composer verbs — the question that opened this ball first. The
        // fixture's conversation is settled, so Stop is painted disabled and it
        // is the *disabled* seat that must say why (bl-e266).
        "writes a died epitaph for every driver",
        "there is no driver to kill",
        "Deposit this text in the selected conversation's inbox",
        // The roster and the top bar. The fixture workspace is foreign, so it
        // rides the overflow rather than a tab (see the smoke test).
        "open this conversation",
        "Every setting yog can write",
        "Workspaces that are real but not regimes",
        // The §11 center strip (bl-1ca2): the tab focuses that replaced the
        // full-cover overlays, each stating what it is. One home for the words
        // — the strip and the left-panel entry read the same `CenterTab::hint`,
        // which is why "Every setting yog can write" above covers both seats.
        "Escape from any other tab comes back here",
        "Sign in to a model provider",
        // Altitude 2.
        "What was said: one line per delivered message",
        "Show the files behind this tab exactly as they are on disk",
        // The bottom accessory.
        "Open the ops trail",
    ] {
        assert!(
            painted.contains(phrase),
            "no control painted the hover {phrase:?}:\n{painted}"
        );
    }
}
