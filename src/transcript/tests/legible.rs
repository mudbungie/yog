//! **Anything hidden is hidden behind a triangle** (bl-7654) — the §11
//! transcript's half of QUALITY G1, asserted on the **laid galley**.
//!
//! Every assertion here reads [`crate::paint_probe::seen_of`]: the glyphs that
//! reached the glass, the rect they were laid into, the part of that rect the
//! clip let through, and the ink they were painted in. Never `Row::preview` /
//! `Row::body` — the standing hazard on this surface is a probe that reads the
//! projection and therefore cannot see elision at all (bl-bc06: 1815 tests
//! passed while covering no truncation).
//!
//! **Two seats, because the row may not depend on ambient state.** The shipped
//! transcript paints inside the centre, whose root sets §11 rule 1's
//! `Truncate` (`shell::row::bounded`, bl-5410) — that is what cut an *expanded*
//! 400-character answer to 67 glyphs at 420x320. A seat that declares nothing
//! is egui's `Extend`, which slices the same run mid-glyph with no ellipsis at
//! all. Both are real seats, so both are swept, at both of the window sizes
//! this surface has broken at (bl-5410, bl-9551).

use std::collections::HashSet;

use super::render::{entry, tx};
use crate::paint_probe::{Seen, screen_sized, seen_of};
use crate::transcript::{AutoExpand, Block, EntryKind, Transcript, Usage};

/// The two window sizes this surface has broken at.
pub(super) const SIZES: [(f32, f32); 2] = [(420.0, 320.0), (800.0, 500.0)];

/// The toggle glyphs, and the mark a row with nothing to fold wears instead.
pub(super) const OPEN: &str = "▼";
pub(super) const SHUT: &str = "▶";
const NO_FOLD: &str = "·";

/// A payload that fits its line and has nothing to fold.
const SHORT: &str = "pong";

/// A single-line model answer far wider than either window.
pub(super) fn long() -> String {
    "abcdefghij".repeat(40)
}

/// Every galley one settled render of `t` put on the glass. `bounded` is
/// whether the seat carries the centre's ambient `Truncate`; `false` is a seat
/// that declares nothing, where egui's horizontal default is `Extend`.
pub(super) fn seen(t: &Transcript, auto: AutoExpand, bounded: bool, w: f32, h: f32) -> Vec<Seen> {
    let ctx = egui::Context::default();
    let mut folds = HashSet::new();
    let mut frame = || {
        ctx.run(screen_sized(w, h), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if bounded {
                    // Verbatim what `shell::row::bounded` puts at the centre
                    // panel root (bl-5410) — the transcript's real seat.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                }
                super::plain(ui, t, false, auto, &mut folds);
            });
        })
    };
    let _ = frame();
    seen_of(&frame())
}

/// A run is **whole** when its seat showed all of what was laid into it. A run
/// laid wider than its clip rect is one the operator reads the front of, with
/// no ellipsis to say so — the galley was never truncated, so egui never added
/// one.
pub(super) fn whole(seen: &Seen) -> bool {
    seen.shown.width() >= seen.laid.width() - 1.0
}

/// The run carrying `needle`'s glyphs, whichever seat painted it.
pub(super) fn run(painted: &[Seen], needle: &str) -> Seen {
    let seats: Vec<&String> = painted.iter().map(|s| &s.text).collect();
    let hit = painted.iter().find(|s| s.text.contains(needle)).cloned();
    assert!(
        hit.is_some(),
        "nothing on the glass carries {needle:?}: {seats:?}"
    );
    hit.expect("asserted present just above")
}

/// A transcript of one model turn whose single text block is `payload`.
pub(super) fn answer(payload: &str) -> Transcript {
    tx(vec![entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![Block::Text(payload.into())],
    })])
}

/// **Finding 2 — the expanded body is shown in full.** A triangle that reveals
/// a still-cut payload is worse than no triangle, so the run the fold opens
/// onto carries every character of the answer and is shown entire.
#[test]
fn an_expanded_answer_reaches_the_glass_entire() {
    let payload = long();
    let t = answer(&payload);
    for bounded in [false, true] {
        for (w, h) in SIZES {
            let painted = seen(&t, AutoExpand::default(), bounded, w, h);
            let body = run(&painted, "abcdefghij");
            assert_eq!(
                body.text,
                payload,
                "the open fold shows {} of {} characters at {w}x{h} (bounded {bounded})",
                body.text.chars().count(),
                payload.chars().count()
            );
            assert!(
                whole(&body),
                "the open fold's body is sliced at {w}x{h} (bounded {bounded}): \
                 {:.0} pt laid, {:.0} pt shown",
                body.laid.width(),
                body.shown.width()
            );
            assert!(
                painted.iter().any(|s| s.text == OPEN),
                "an expanded row wears the open triangle at {w}x{h}"
            );
        }
    }
}

/// **Finding 1(b) — a contracted preview is marked, and the mark is on
/// screen.** The preview stands in for a body, so it may end in `…`; what it
/// may never be is cut *silently*. The triangle beside it is the affordance
/// the ellipsis promises.
#[test]
fn a_contracted_preview_says_it_is_abridged_and_wears_a_triangle() {
    let t = answer(&long());
    let auto = AutoExpand {
        responses: false,
        others: false,
    };
    for bounded in [false, true] {
        for (w, h) in SIZES {
            let painted = seen(&t, auto, bounded, w, h);
            let preview = run(&painted, "abcdefghij");
            assert!(
                whole(&preview),
                "the preview is cut with no ellipsis at {w}x{h} (bounded {bounded}): \
                 {:.0} pt laid, {:.0} pt shown, {:?}",
                preview.laid.width(),
                preview.shown.width(),
                preview.text
            );
            assert!(
                preview.text.ends_with('…'),
                "an abridged preview must say so at {w}x{h} (bounded {bounded}): {:?}",
                preview.text
            );
            assert!(
                painted.iter().any(|s| s.text == SHUT),
                "the abridged row wears the shut triangle at {w}x{h}"
            );
        }
    }
}

/// **Finding 3 — the fade means "there is more behind this".** A payload shown
/// whole is a complete statement and paints solid; an abridged preview paints
/// at the one §11 solidity (`theme::tone_solidity`), read back off the glass
/// and tied to that single home rather than restated as a number here.
#[test]
fn only_an_abridged_preview_is_faded() {
    let shut = AutoExpand {
        responses: false,
        others: false,
    };
    for (w, h) in SIZES {
        let complete = seen(&answer(SHORT), AutoExpand::default(), true, w, h);
        let solid = run(&complete, SHORT);
        assert_eq!(
            solid.ink.a(),
            u8::MAX,
            "a row with nothing to fold IS the whole content and must not read \
             as abridged at {w}x{h}: {:?}",
            solid.ink
        );
        assert!(
            complete.iter().all(|s| s.text != SHUT && s.text != OPEN),
            "the fixture must have nothing to fold, or this proves nothing: {:?}",
            complete.iter().map(|s| &s.text).collect::<Vec<_>>()
        );

        let abridged = seen(&answer(&long()), shut, true, w, h);
        let faded = run(&abridged, "abcdefghij");
        let want = crate::theme::tone_solidity(crate::transcript::Tone::Weak);
        assert!(
            (f32::from(faded.ink.a()) / f32::from(u8::MAX) - want).abs() < 0.02,
            "an abridged preview paints at the §11 solidity {want} at {w}x{h}, \
             got alpha {} ({:?})",
            faded.ink.a(),
            faded.ink
        );
    }
}

/// **The invariant itself, swept over every row class the transcript has.**
/// For every run on the glass: either it is whole, or it ends in `…` with a
/// disclosure triangle on its own row. Nothing may be cut silently, and
/// nothing marked as cut may be unreachable.
#[test]
fn no_run_is_hidden_without_a_triangle_beside_it() {
    let t = super::parity::mixed(&long());
    for bounded in [false, true] {
        for auto in [
            AutoExpand::default(),
            AutoExpand {
                responses: false,
                others: false,
            },
            AutoExpand {
                responses: true,
                others: true,
            },
        ] {
            for (w, h) in SIZES {
                let painted = seen(&t, auto, bounded, w, h);
                for s in &painted {
                    let seat = format!("{:?} at {w}x{h} (bounded {bounded}, {auto:?})", s.text);
                    assert!(whole(s), "cut with no ellipsis: {seat}");
                    if !s.text.ends_with('…') {
                        continue;
                    }
                    // The triangle has to be on THIS line: an open fold's body
                    // is a row of its own, and the toggle above it opens
                    // nothing further.
                    let band = s.laid.top() - 1.0..=s.laid.bottom() + 1.0;
                    assert!(
                        painted.iter().any(|g| {
                            (g.text == SHUT || g.text == OPEN) && band.contains(&g.laid.center().y)
                        }),
                        "marked as cut with no triangle to turn: {seat}"
                    );
                }
            }
        }
    }
}

/// The row with nothing to fold still wears the alignment mark, not a
/// triangle — the fold vocabulary is unchanged by any of the above, and this
/// is what keeps the sweep's "a triangle on the row" from passing on a surface
/// that had simply grown triangles everywhere.
#[test]
fn a_row_with_nothing_to_fold_wears_the_alignment_mark() {
    let painted = seen(&answer(SHORT), AutoExpand::default(), true, 800.0, 500.0);
    assert!(painted.iter().any(|s| s.text == NO_FOLD));
}

/// **Raw has no triangles at all, so its bytes must be whole.** The toggle is
/// the escape from a parse (§11) — bytes cut at the pane edge are not the
/// bytes, and there is nothing to turn to get the rest.
#[test]
fn the_raw_view_shows_its_verbatim_bytes_whole() {
    let payload = long();
    let t = Transcript {
        entries: vec![crate::transcript::Entry {
            name: "001-opus.json".into(),
            raw: payload.clone().into_bytes(),
            kind: EntryKind::Raw,
        }],
    };
    let ctx = egui::Context::default();
    let mut folds = HashSet::new();
    let mut frame = || {
        ctx.run(screen_sized(420.0, 320.0), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                super::plain(ui, &t, true, AutoExpand::default(), &mut folds);
            });
        })
    };
    let _ = frame();
    let painted = seen_of(&frame());
    let bytes = run(&painted, "abcdefghij");
    assert_eq!(bytes.text, payload, "the Raw view shows every byte");
    assert!(whole(&bytes), "the Raw view is cut: {:?}", bytes.text);
    assert!(
        painted.iter().all(|s| s.text != SHUT && s.text != OPEN),
        "Raw carries no fold to turn, which is why it may hide nothing"
    );
}
