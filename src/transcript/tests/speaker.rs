//! **The speaker is not the output** (bl-f3fc, operator ruling): a delivered
//! message's label wears the role's hue on a line of its own, above the payload.
//!
//! Asserted on the **laid galley** through [`crate::paint_probe::seen_of`], as
//! everything about this surface is (bl-bc06). A stripe is a `rect_filled` and
//! reads back through `paint_fills`; a coloured **label** reaches the glass as
//! glyphs, so what proves its hue is `Seen::ink` and what proves its line is
//! `Seen::laid`. Never `Row::prefix` — the projection cannot see paint at all.

use super::legible::{SHUT, run, seen};
use super::render::{entry, tx};
use crate::inboxview::Epitaph;
use crate::theme::{Role, role_badge};
use crate::transcript::{AutoExpand, Block, EntryKind, Transcript, Usage};

/// A window wide and tall enough to lay every row of [`speaking`] whole.
const WIDE: (f32, f32) = (800.0, 500.0);

/// A payload that fits its line, so the row has nothing to fold and its preview
/// is the payload entire — the plainest shape the two-line claim can be made on.
const SAID: &str = "words";

/// The ending a result deposit asserts in these fixtures.
const ENDING: Epitaph = Epitaph::Stopped;

/// One transcript carrying every role: the operator, the agent, a peer, and a
/// dispatched child's result deposit.
fn speaking() -> Transcript {
    let delivered = |sender: &str, epitaph| EntryKind::Delivered {
        sender: sender.into(),
        epitaph,
        body: SAID.into(),
    };
    tx(vec![
        entry(delivered("user", None)),
        entry(EntryKind::Model {
            model_id: "opus".into(),
            usage: Usage::default(),
            blocks: vec![Block::Text(SAID.into())],
        }),
        entry(delivered("peer", None)),
        entry(delivered("kid", Some(ENDING))),
    ])
}

/// Every speaker label this fixture paints, paired with the role it speaks for
/// — built from the same seats the projection builds its prefixes from, so a
/// relabelling moves both together.
fn labels() -> Vec<(String, Role)> {
    vec![
        ("user:".to_owned(), Role::User),
        (format!("{}:", super::rows::SPEAKER), Role::Model),
        ("peer:".to_owned(), Role::Peer),
        (format!("kid ended: {}", ENDING.label()), Role::Ended),
    ]
}

/// **The hue half.** The label carries the role's own colour, from the one
/// mapping the stripe and the pending queue already read (`theme::role_badge`)
/// — no hue is minted here, so the assertion names none.
#[test]
fn every_speaker_label_is_inked_in_its_own_role_hue() {
    let painted = seen(&speaking(), AutoExpand::default(), true, WIDE.0, WIDE.1);
    for (label, role) in labels() {
        let (hue, _) = role_badge(role);
        let seat = run(&painted, &label);
        assert_eq!(
            seat.ink, hue,
            "{label:?} must reach the glass in {role:?}'s hue, got {:?}",
            seat.ink
        );
    }
}

/// **The line half, and what it is FOR.** The label sits above the payload —
/// a strictly higher band, never overlapping it — and the payload keeps the
/// default body ink the label no longer shares, which is the whole complaint:
/// prefix and body wore identical ink on one line and the eye could not tell
/// speaker from output.
#[test]
fn the_speaker_label_sits_above_the_payload_and_is_not_its_ink() {
    let painted = seen(&speaking(), AutoExpand::default(), true, WIDE.0, WIDE.1);
    let payloads: Vec<_> = painted.iter().filter(|s| s.text == SAID).collect();
    assert_eq!(payloads.len(), 4, "one payload run per speaking row");
    for ((label, _), payload) in labels().into_iter().zip(payloads) {
        let speaker = run(&painted, &label);
        assert!(
            speaker.laid.bottom() <= payload.laid.top(),
            "{label:?} must own the line above its payload: label {:?}, payload {:?}",
            speaker.laid,
            payload.laid
        );
        assert_ne!(
            speaker.ink, payload.ink,
            "{label:?} must not wear the payload's own ink"
        );
    }
}

/// **The toggle rides the line it folds.** Splitting the row moved the label
/// off the payload's band; the triangle had to stay on it, or an elided preview
/// would be marked as cut with nothing beside it to turn (the bl-7654
/// invariant, swept whole in [`super::legible`]).
#[test]
fn the_fold_triangle_stays_on_the_payload_band() {
    let long = super::legible::long();
    let t = tx(vec![entry(EntryKind::Delivered {
        sender: "user".into(),
        epitaph: None,
        body: long.clone(),
    })]);
    let shut = AutoExpand {
        responses: false,
        others: false,
    };
    let painted = seen(&t, shut, true, WIDE.0, WIDE.1);
    let preview = run(&painted, "abcdefghij");
    let triangle = run(&painted, SHUT);
    let band = preview.laid.top() - 1.0..=preview.laid.bottom() + 1.0;
    assert!(
        band.contains(&triangle.laid.center().y),
        "the triangle is off its own payload's band: preview {:?}, toggle {:?}",
        preview.laid,
        triangle.laid
    );
    let speaker = run(&painted, "user:");
    assert!(
        !band.contains(&speaker.laid.center().y),
        "the speaker label must have left that band: {:?}",
        speaker.laid
    );
}

/// **Machinery is untouched.** Nobody is speaking on a thinking row or a tool
/// result, so there is no speaker to set apart: the prefix keeps its tone paint
/// and its seat on the payload's own line.
#[test]
fn a_machinery_row_keeps_one_line_and_no_role_hue() {
    let t = tx(vec![
        entry(EntryKind::ToolResult {
            tool_use_id: "t".into(),
            content: SAID.into(),
            is_error: false,
        }),
        entry(EntryKind::Model {
            model_id: "opus".into(),
            usage: Usage::default(),
            blocks: vec![Block::Thinking(SAID.into())],
        }),
    ]);
    let painted = seen(&t, AutoExpand::default(), true, WIDE.0, WIDE.1);
    let hues: Vec<_> = [Role::User, Role::Model, Role::Peer, Role::Ended]
        .into_iter()
        .map(|role| role_badge(role).0)
        .collect();
    for prefix in ["tool result", "thinking:"] {
        let seat = run(&painted, prefix);
        assert!(
            !hues.contains(&seat.ink),
            "{prefix:?} must wear no role hue, got {:?}",
            seat.ink
        );
        let band = seat.laid.top() - 1.0..=seat.laid.bottom() + 1.0;
        assert!(
            painted
                .iter()
                .any(|s| s.text == SAID && band.contains(&s.laid.center().y)),
            "{prefix:?} must stay on one line with its payload: {:?}",
            painted
                .iter()
                .map(|s| (&s.text, s.laid.top()))
                .collect::<Vec<_>>()
        );
    }
}
