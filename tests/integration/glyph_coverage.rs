//! The tofu guard (bl-24d3): **every non-ASCII character that appears in a
//! `src` string or char literal must have a real glyph in the fonts yog
//! installs** — in *both* font families.
//!
//! Why this test exists: egui ships Proportional without Hack, so a glyph
//! Hack alone covers (`●◐◈⋯▼→⇒`) painted in a proportional seat rendered as a
//! tofu box — silently, at runtime, on every conversation row. Inspection
//! could not see it; only a probe against `epaint::Fonts::has_glyph` can.
//! `theme::fonts` closed the hole by giving both families the same font *set*
//! (see its doc); this test is what keeps it closed the next time somebody
//! types a nice-looking arrow into a label.
//!
//! Why *both* families and not "the family that will paint it": which family
//! paints a given literal is a property of the call site, not of the source
//! text, so attributing it statically would be guesswork. `theme::fonts`
//! removes the question instead — the two families carry the same fonts, so
//! coverage is identical by construction. [`families_carry_the_same_fonts`]
//! asserts that premise; the sweep then checks both and needs no attribution.
//!
//! The literal scan is a real Rust lexer state machine, not a regex: comments
//! (which are full of `§ — …` and of glyph names quoted in prose) must be
//! excluded or the guard would fire on text nothing ever paints, and raw
//! strings/lifetimes must be told apart from ordinary strings/chars.
//! [`scanner_reads_rust_not_bytes`] pins that behaviour, and
//! [`the_sweep_is_not_vacuous`] proves the sweep sees the real source — a
//! scanner that quietly returned nothing would make this whole file a no-op.

// An integration-test root resolves `mod` against `tests/`, not against its own
// name, so the scanner's home is named outright.
#[path = "glyph_coverage/scan.rs"]
mod scan;

use egui::epaint::text::Fonts;
use egui::{FontFamily, FontId};
use scan::{Sites, scan, sweep_src};

#[test]
fn families_carry_the_same_fonts() {
    // The premise the sweep below rests on: after `theme::fonts`, Proportional
    // and Monospace list the same fonts (in different priority order), so a
    // glyph either renders in both seats or in neither. egui's own defaults
    // do NOT satisfy this — that asymmetry is the bug this guard closes.
    let defs = yog::theme::fonts();
    let sorted = |f: &FontFamily| {
        let mut v = defs.families.get(f).cloned().unwrap_or_default();
        v.sort();
        v
    };
    assert_eq!(
        sorted(&FontFamily::Proportional),
        sorted(&FontFamily::Monospace)
    );
    assert!(!sorted(&FontFamily::Proportional).is_empty());
    // And the defaults really are asymmetric, so the fold is load-bearing.
    let stock = egui::FontDefinitions::default();
    assert_ne!(
        stock.families.get(&FontFamily::Proportional),
        stock.families.get(&FontFamily::Monospace),
    );
}

#[test]
fn every_non_ascii_literal_in_src_has_a_glyph() {
    let fonts = Fonts::new(1.0, 4096, yog::theme::fonts());
    let mut tofu: Vec<String> = Vec::new();
    for (c, sites) in sweep_src() {
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            if !fonts.has_glyph(&FontId::new(14.0, family.clone()), c) {
                tofu.push(format!(
                    "U+{:04X} {c} has no glyph in {family:?} — used at {}",
                    c as u32,
                    sites.join(", ")
                ));
            }
        }
    }
    assert!(
        tofu.is_empty(),
        "these characters render as tofu boxes:\n{}\n\
         Fix by choosing a covered codepoint (probe candidates with \
         `Fonts::has_glyph`) — DESIGN §11 owns the badge vocabulary.",
        tofu.join("\n")
    );
}

#[test]
fn the_sweep_is_not_vacuous() {
    // A scanner that silently found nothing would make the guard above pass
    // forever. Pin it to glyphs the repo really paints.
    let found = sweep_src();
    assert!(found.len() > 20, "only found {:?}", found.keys());
    for c in ['●', '◐', '○', '■', '✔', '✖', '→', '✉'] {
        assert!(found.contains_key(&c), "sweep missed {c}, which src paints");
    }
}

#[test]
fn scanner_reads_rust_not_bytes() {
    let mut out = Sites::new();
    scan(
        concat!(
            "// comment ✗ ignored\n",
            "/* block ✗ /* nested ✗ */ still ✗ */\n",
            "let a = \"kept ★\";\n",
            "let b = \"escaped \\\" ⚑ kept\";\n",
            "let c = r#\"raw ⚙ \"quoted\" kept\"#;\n",
            "let d = '⏭';\n",
            "fn f<'a>(x: &'a str) -> Ref<'_> { rest ✗ }\n",
        ),
        "synthetic.rs",
        &mut out,
    );
    let mut got: Vec<char> = out.keys().copied().collect();
    got.sort_unstable();
    assert_eq!(got, vec!['⏭', '★', '⚑', '⚙']);
    // Lines are tracked through multi-line comments, so a report points at the
    // real site: the `r#"…"#` literal is on line 5.
    assert_eq!(out[&'⚙'], vec!["synthetic.rs:5".to_owned()]);
    // A leading `r` that is not a raw string must not swallow the file.
    let mut ident = Sites::new();
    scan("let ready = \"⚑\";", "i.rs", &mut ident);
    assert_eq!(ident.keys().copied().collect::<Vec<_>>(), vec!['⚑']);
}
