//! §11 rule 1b, driven through the real rows: **a control is never elided**
//! (bl-bc06).
//!
//! Both witnesses lay greedy text and a trailing control on one line inside a
//! panel whose root wrap mode is `Truncate` (rule 1), and both were painted at
//! the DEFAULT 1150×760 window — not a small one. Laid text-first the text took
//! the whole row and the control was handed one character:
//!
//! ```text
//! gated ▶ bl-b5ce: epic: the inspector rework   [ assig… ]
//! ready ▶ bl-7cdd: tune the transcript scroll anchor  [ … ]
//! claude-session-direct  auth oauth2 · not signed in  [ … ]
//! ```
//!
//! The evidence is the **paint layer's glyphs**, not its text: an egui galley
//! truncated to `…` still reports the whole string from `Galley::text()`, so
//! [`crate::paint_probe`] reads the laid-out glyphs instead (bl-bc06) and a
//! needle that is not on screen no longer matches. Each assertion is made in
//! both directions — the verb whole AND the text beside it elided — so neither
//! can pass by the row merely having had room; and each row is measured against
//! the panel's own edge, because rule 1b must not buy the verb by laying it
//! outside the panel, which is the bl-ac3d defect rule 1 exists to stop.
//!
//! L4's other question — not *whether* a row elides but **where** the cut
//! falls — is the same evidence about a different subject, and lives in
//! [`super::activity_tail`].

use super::super::{login_pane, start_rows};
use super::fixture::world;
use crate::cli_outbound::Cli;
use crate::config_edit::brazen::ProviderRowView;
use crate::paint_probe::{self, Painted};
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, StartInputs};

/// A panel narrow enough that a row must contend for space — wider and nothing
/// elides, which is why the defect needs a bounded panel to reproduce at all.
const PANEL: f32 = 240.0;

/// Paint `body` in a bounded panel carrying the side panel's own wrap mode
/// (§11 rule 1), and hand back every galley with where it landed.
fn in_a_bounded_panel(body: impl FnOnce(&mut egui::Ui)) -> Vec<Painted> {
    let ctx = egui::Context::default();
    let mut body = Some(body);
    let out = ctx.run(paint_probe::screen_sized(600.0, 400.0), |ctx| {
        egui::SidePanel::left("bounded")
            .exact_width(PANEL)
            .show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                if let Some(body) = body.take() {
                    body(ui);
                }
            });
    });
    paint_probe::painted_of(&out)
}

/// The same row in a seat that sets **no** wrap mode at all — the centre, where
/// egui's default is `Extend`. The Login rows paint here too: the auth-failed
/// banner renders `login_section` inline in the conversation (§11, one
/// machinery in two seats).
fn in_a_seat_with_no_wrap_mode(body: impl FnOnce(&mut egui::Ui)) -> Vec<Painted> {
    let ctx = egui::Context::default();
    let mut body = Some(body);
    let out = ctx.run(paint_probe::screen_sized(PANEL, 400.0), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(body) = body.take() {
                body(ui);
            }
        });
    });
    paint_probe::painted_of(&out)
}

/// What the row says, one galley per line.
fn says(painted: &[Painted]) -> String {
    let mut out = String::new();
    for (text, _) in painted {
        out.push_str(text);
        out.push('\n');
    }
    out
}

/// Nothing the row painted leaves the panel — **either side of it** (§11 rule
/// 1). Until bl-7414 this measured `rect.right()` alone, which is bl-36c3's
/// vacuity shape 3 (right axis, one direction): a `right_to_left` row that is
/// handed less width than it needs grows *leftwards* off its right edge, so
/// under a rule that only ever watched the right edge, overflowing left was
/// free. The panel is a span; the assertion is now the span.
fn assert_inside_the_panel(painted: &[Painted]) {
    for (text, rect) in painted {
        assert!(
            rect.right() <= PANEL + 1.0,
            "`{text}` is laid past the panel's right edge at {}",
            rect.right()
        );
        assert!(
            rect.left() >= -1.0,
            "`{text}` is laid past the panel's left edge at {} — a row that \
             cannot fit grows off the edge it was laid from, and the operator \
             gets a hard cut with no ellipsis to warn them",
            rect.left()
        );
    }
}

/// The damaging witness: the provider that is NOT signed in is the one whose
/// name is longest, so the one row where the operator must press Login was the
/// row whose Login vanished (§8.3, QUALITY G1/H2).
#[test]
fn the_login_verb_stays_whole_on_the_row_whose_name_would_have_eaten_it() {
    let painted = in_a_bounded_panel(|ui| {
        login_pane::provider_row(
            ui,
            &ProviderRowView {
                name: "claude-session-direct".to_owned(),
                fact: "auth oauth2 · not signed in".to_owned(),
                blocked: None,
            },
        );
    });
    let said = says(&painted);
    assert!(
        said.contains("Login"),
        "the verb is on screen whole, not a bare `…`:\n{said}"
    );
    assert!(
        !said.contains("auth oauth2 · not signed in"),
        "and the row really was over-full — the fact beside it elided:\n{said}"
    );
    assert!(
        said.contains("claude-session-direct"),
        "the name, which is the row's identity, survives:\n{said}"
    );
    assert_inside_the_panel(&painted);
}

/// The balls-board witness: `assign → <ws>` after a `▶ <id>: <title>` label of
/// arbitrary length (§8.2). The label truncates in the verb's place, keeping
/// its glyph, its id, and its hover.
#[test]
fn the_assign_verb_stays_whole_behind_a_ball_title_of_any_length() {
    let litany = Cli::new("litany");
    let mut world = world();
    let ws = world.ws.clone();
    let inputs = StartInputs {
        workspace: ws.clone(),
        repo: Some(ws.clone()),
        payload: Payload::Ball {
            project: crate::naming::leaf(&ws),
            ball: BallSpec::Existing {
                id: "bl-b5ce".to_owned(),
                title: "epic: the inspector rework".to_owned(),
                body: String::new(),
                join: JoinState::ReadyStartable,
                tags: Vec::new(),
            },
        },
        home: ws.clone(),
        yog_data_root: world.yog_data.clone(),
        balls_state_root: ws,
        conversation_names: Vec::new(),
    };
    let to = world
        .model
        .focused_ws_name()
        .expect("the fixture focuses a workspace");
    let painted = in_a_bounded_panel(|ui| {
        start_rows::ready_row(ui, &mut world.model, &mut world.state, &litany, inputs);
    });
    let said = says(&painted);
    assert!(
        said.contains(&format!("assign → {to}")),
        "the assign verb names its target whole:\n{said}"
    );
    assert!(
        !said.contains("epic: the inspector rework"),
        "and the row really was over-full — the ▶ label elided:\n{said}"
    );
    assert!(
        said.contains("▶ bl-b5ce"),
        "keeping the verb glyph and the ball id at its head:\n{said}"
    );
    assert_inside_the_panel(&painted);
}

/// Rule 1b must not **depend** on rule 1 (bl-9551's `overlap` walk caught this
/// against bl-bc06's first shape). Pinning the trailing element right while the
/// text beside it is free to `Extend` does not merely let the text run off the
/// panel edge as it did before — it runs the text straight *through* the
/// control, turning an elision defect into an overlap, which is worse. So the
/// helper sets the truncation itself and the row holds in a seat that declares
/// no wrap mode: the centre, where the Login rows really do paint (the
/// auth-failed banner renders them inline in the conversation).
///
/// Driven on the **blocked** branch, whose trailing element is the reason
/// there is no verb rather than a button — the exact row whose two runs
/// shared pixels at 800×500.
#[test]
fn the_row_truncates_itself_in_a_seat_that_declares_no_wrap_mode() {
    let painted = in_a_seat_with_no_wrap_mode(|ui| {
        login_pane::provider_row(
            ui,
            &ProviderRowView {
                name: "openai-chatgpt".to_owned(),
                fact: "auth bearer · no credential stored".to_owned(),
                blocked: Some("bearer-token provider — set the token in Config".to_owned()),
            },
        );
    });
    let said = says(&painted);
    assert!(
        said.contains("bearer-token provider — set the token in Config"),
        "the reason there is no verb stays whole:\n{said}"
    );
    assert!(
        !said.contains("auth bearer · no credential stored"),
        "and the fact beside it elided rather than extending:\n{said}"
    );
    // The overlap itself, asserted here too rather than only in bl-9551's walk:
    // this is the row that produced it, so its own file should fail on it.
    for (i, (a_text, a)) in painted.iter().enumerate() {
        for (b_text, b) in painted.iter().skip(i + 1) {
            assert!(
                !a.intersects(*b),
                "`{a_text}` and `{b_text}` share pixels: {a:?} vs {b:?}"
            );
        }
    }
    assert_inside_the_panel(&painted);
}
