//! **What the widget paints** — the two §11 render modes over the same
//! listing: the parsed deposit's header and body, every epitaph and terminal
//! ref, absent fields as question marks, and the Raw toggle's unaltered bytes.
//! Split from [`super`] at §12's budget on the seam that file already had: the
//! envelope parse and the listing's file facts are a claim about *bytes*, and
//! these are a claim about *glyphs*, read off the paint layer.

use super::super::render::{HOW_MAIL_ARRIVES, NO_DEPOSITS, render};
use super::super::*;
use super::{entry, painted};
use crate::nav::convs::Titles;

/// A pane tall enough that a bottom anchor is unmistakable — the audit's
/// witness had `(no deposits)` ~450 pt below the tab strip — and wide enough
/// that neither sentence wraps, so each is asserted as one painted run.
const PANE: (f32, f32) = (1400.0, 400.0);

/// **An empty inbox says what it is and how a deposit arrives, at the top of
/// the pane** (bl-71fc, QUALITY H2: "absence is named — an empty region says
/// what it is and names the paved path in full").
///
/// Both halves are asserted, in both render modes. The string alone was the
/// test this replaces, and it was vacuous against the complaint twice over: it
/// passed on a bare `(no deposits)` that named no path, and it said nothing
/// about *where* that run sat, which is the other half of the defect —
/// `tail::scroll`'s anchor pads an underfull body down onto the bottom edge,
/// so the one line an operator had was ~450 pt from the tab that produced it.
#[test]
fn an_empty_inbox_names_itself_and_the_paved_path_at_the_top_of_the_pane() {
    for raw in [false, true] {
        let painted = crate::paint_probe::painted_settled(PANE.0, PANE.1, |ui| {
            render(ui, &[], &Titles::default(), raw);
        });
        let said: Vec<&str> = painted.iter().map(|(text, _)| text.as_str()).collect();
        for want in [NO_DEPOSITS, HOW_MAIL_ARRIVES] {
            assert!(
                said.contains(&want),
                "raw={raw}: the empty inbox must paint `{want}` whole, got {said:?}"
            );
        }
        let (top, bottom) = crate::paint_probe::span(&painted);
        assert!(
            top < 20.0,
            "raw={raw}: the empty state is top-anchored, not pushed down the pane: \
             it starts at y {top} of a {} pt pane",
            PANE.1
        );
        assert!(
            bottom < PANE.1 / 2.0,
            "raw={raw}: and the whole of it sits in the pane's top half: ends at y {bottom}"
        );
    }
}

#[test]
fn ordinary_deposit_renders_header_and_body() {
    let e = entry(
        "user-001.md",
        "---\nfrom: user\ndeposited_at: 2026-07-17T12:00:00Z\n---\nplease continue",
    );
    let text = painted(std::slice::from_ref(&e), false);
    assert!(
        text.contains("✉ user · 2026-07-17T12:00:00Z"),
        "got:\n{text}"
    );
    assert!(text.contains("please continue"));
    // No epitaph line for a plain deposit.
    assert!(!text.contains("epitaph:"));
}

#[test]
fn result_message_renders_every_epitaph_and_terminal_ref() {
    let cases = [
        (Epitaph::FinalResponse, "final-response"),
        (Epitaph::Stopped, "stopped"),
        (Epitaph::BudgetExhausted, "budget-exhausted"),
        (Epitaph::Died, "died"),
        (Epitaph::Unknown("wat".into()), "wat"),
    ];
    for (epitaph, label) in cases {
        let e = InboxEntry {
            name: "c-1-001.md".into(),
            raw: Vec::new(),
            deposit: Deposit {
                sender: Some("c-1".into()),
                deposited_at: Some("t".into()),
                epitaph: Some(epitaph),
                terminal_ref: Some("deadbeef".into()),
                body: String::new(),
            },
        };
        let text = painted(std::slice::from_ref(&e), false);
        assert!(text.contains(&format!("epitaph: {label}")), "got:\n{text}");
        assert!(text.contains("terminal: deadbeef"));
    }
}

#[test]
fn absent_fields_render_as_question_marks() {
    let e = entry("odd.md", "raw");
    let text = painted(std::slice::from_ref(&e), false);
    assert!(text.contains("✉ ? · ?"), "got:\n{text}");
}

/// S7-T1, Inbox half: Raw flips the tab to each deposit file's name and its
/// bytes exactly as they sit on disk — envelope included, nothing summarized
/// away and no parsed header in the way.
#[test]
fn raw_mode_paints_each_files_name_and_unaltered_bytes() {
    let bytes = "---\nfrom: user\ndeposited_at: t1\n---\nthe body\n";
    let e = entry("user-001.md", bytes);
    let text = painted(std::slice::from_ref(&e), true);
    assert!(text.contains("user-001.md"), "no filename header:\n{text}");
    assert!(text.contains(bytes), "bytes not verbatim:\n{text}");
    assert!(!text.contains("✉ user"), "parsed header in raw:\n{text}");
}

/// bl-3aa1 / QUALITY L4: a deposit from a descent child must not lead its row
/// with the child's whole ancestry chain. The witness verbatim — a four-token
/// chain, 52 characters, at the head of a row whose other content is a
/// timestamp and a subject.
///
/// Asserted on the **paint layer**, and in both directions: the terminal
/// generation is on screen, and the chain that used to dominate the row is not.
/// Since bl-bc06 the probe reports the glyphs a galley actually laid, so a
/// needle matching here really is text the operator can read.
#[test]
fn a_deposit_row_floors_a_descent_chain_to_its_terminal_generation() {
    const CHAIN: &str = "20260807T214551Z-2a1181a3-20260727T090100Z-c0ffeeba";
    let e = entry(
        "child-001.md",
        &format!(
            "---\nfrom: {CHAIN}\ndeposited_at: 2026-08-07T22:03:25Z\n---\nmail nobody is driving"
        ),
    );
    assert_eq!(
        e.deposit.sender.as_deref(),
        Some(CHAIN),
        "the fact is kept whole on the deposit"
    );

    let said = painted(std::slice::from_ref(&e), false);
    assert!(
        said.contains("✉ 20260727T090100Z-c0ffeeba · 2026-08-07T22:03:25Z"),
        "the row leads with the child's own generation:\n{said}"
    );
    assert!(
        !said.contains(CHAIN),
        "and the ancestry chain no longer dominates the row:\n{said}"
    );
    assert!(
        said.contains("mail nobody is driving"),
        "the subject — what the row is actually about — is still there:\n{said}"
    );
}

/// The same call must leave a sender that is not an agent id alone: `user` has
/// no stamp grammar, so the floor spells it whole. The general path with input
/// the rule does not recognise, not a special case.
#[test]
fn a_deposit_from_the_operator_is_not_floored() {
    let e = entry(
        "user-001.md",
        "---\nfrom: user\ndeposited_at: 2026-08-07T22:03:25Z\n---\nfollow-up",
    );
    let said = painted(std::slice::from_ref(&e), false);
    assert!(said.contains("✉ user · 2026-08-07T22:03:25Z"), "{said}");
}
