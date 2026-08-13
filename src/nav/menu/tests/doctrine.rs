//! The §11 doctrine, mechanized as a sweep: for **every** seat the roster can
//! be opened on, no entry is its verb's sole carrier and no entry is unworded.
//! A property over the whole seat space — [`super`] holds the per-seat example
//! tables the same `entries` call is pinned to.

use super::*;

/// Every seat the roster can be opened on — the table the doctrine test sweeps.
/// A new seat is added here and to [`entries`], nowhere else.
fn every_seat() -> Vec<Seat> {
    let mut seats = Vec::new();
    for named in [false, true] {
        for pinned in [false, true] {
            seats.push(Seat::WorkspaceTab { named, pinned });
        }
    }
    for stoppable in [false, true] {
        for has_children in [false, true] {
            for named in [false, true] {
                seats.push(Seat::ConversationRow {
                    stoppable,
                    has_children,
                    named,
                });
            }
        }
    }
    for state in JOIN_STATES {
        for assign_to in [None, Some("alba-koi".to_owned())] {
            for move_to in [Vec::new(), vec!["zeta-pug".to_owned()]] {
                seats.push(Seat::BallRow {
                    state,
                    assign_to: assign_to.clone(),
                    move_to,
                });
            }
        }
    }
    seats
}

/// Walk an entry and its submenu rows — the doctrine holds at every depth.
fn assert_carried(entry: &Entry) {
    assert!(
        !entry.carrier.is_empty(),
        "{entry:?} names no visible carrier"
    );
    assert!(!entry.label.is_empty(), "{entry:?} has no worded label");
    if let Action::Submenu(children) = &entry.action {
        assert!(!children.is_empty(), "{entry:?} is an empty submenu");
        for child in children {
            assert_carried(child);
        }
    }
}

#[test]
fn no_entry_is_its_verbs_sole_carrier() {
    // The §11 doctrine, mechanized: a menu-only verb fails exactly the test
    // a glyph-only state badge fails, so every entry names where else it lives.
    for seat in every_seat() {
        for entry in entries(seat) {
            assert_carried(&entry);
        }
    }
}
