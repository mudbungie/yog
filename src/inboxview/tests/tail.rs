//! §11 tail idiom on the Inbox tab: the listing is oldest-first, so a backlog
//! too tall for the viewport shows its newest deposits (bl-5cdb) — and a single
//! deposit sits on the same bottom edge that backlog ends on (bl-8c13).

use crate::inboxview::{InboxEntry, parse_deposit, render::render};

/// A viewport far shorter than `backlog()`.
const NARROW: (f32, f32) = (600.0, 140.0);

fn backlog() -> Vec<InboxEntry> {
    deposits(30)
}

fn deposits(n: usize) -> Vec<InboxEntry> {
    (0..n)
        .map(|i| {
            let bytes = format!("---\nfrom: user\n---\ndeposit-{i:03}");
            InboxEntry {
                name: format!("user-{i:03}.md"),
                raw: bytes.clone().into_bytes(),
                deposit: parse_deposit(bytes.as_bytes()),
            }
        })
        .collect()
}

#[test]
fn a_tall_inbox_shows_its_newest_deposits() {
    let entries = backlog();
    let painted = crate::paint_probe::paint_settled(NARROW.0, NARROW.1, |ui| {
        render(ui, &entries, &[], false);
    });
    assert!(
        painted.contains("deposit-029"),
        "newest deposit must be seen:\n{painted}"
    );
    assert!(
        !painted.contains("deposit-000"),
        "oldest deposit must have scrolled off:\n{painted}"
    );
}

/// The bottommost pixel `entries` paint in the narrow viewport.
fn bottom_of(entries: &[InboxEntry]) -> f32 {
    let painted = crate::paint_probe::painted_settled(NARROW.0, NARROW.1, |ui| {
        render(ui, entries, &[], false);
    });
    crate::paint_probe::span(&painted).1
}

#[test]
fn a_single_deposit_sits_on_the_same_bottom_edge_a_backlog_does() {
    let one = bottom_of(&deposits(1));
    let many = bottom_of(&backlog());
    assert!(
        (one - many).abs() < 1.0,
        "an underfull inbox must end where a full one does: {one} vs {many}"
    );
}
