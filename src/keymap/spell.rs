//! How the §11 table **spells itself**: the text an operator reads on a hover
//! and types on the keyboard (bl-478d).
//!
//! The ruling is one sentence: everything is keyboard-operable, and every
//! button's mouseover states the combo that fires it. A hover that names a key is naming a fact this module
//! owns, so the spelling is **derived from the table itself** — [`bindings`]
//! sweeps every press through [`keymap`] rather than restating a list — and a
//! rebinding cannot leave a lie on the surface, because there is no second list
//! to forget. The §11 discoverability scan
//! (`shell::acceptance::hover::spelling`) holds every control against exactly
//! this vocabulary.

use super::{Held, Key, KeyAction, Mods, keymap};

/// The three modifier planes, in §11 table order.
const PLANES: [Mods; 3] = [Mods::Bare, Mods::Command, Mods::CommandShift];

/// The **focus floor** this table is an accelerator over (§11): the frame
/// traverses its own controls with Tab / Shift+Tab and presses the focused one
/// with Space, so a gesture the table does not name is still keyboard-operable.
/// It is spelled here, beside the bindings, because it is the half of keyboard
/// rule 2 that holds for *every* control rather than per gesture — and because
/// a hover naming it is naming a §11 fact, not inventing one.
pub(crate) const FLOOR: &str = "Tab";

/// Every logical key the shell can lift — the sweep's whole domain, so a
/// binding cannot hide from it.
fn keys() -> Vec<Key> {
    let mut all = vec![
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Enter,
        Key::Escape,
        Key::Plus,
        Key::Minus,
    ];
    all.extend((0..=9).map(Key::Digit));
    all.extend(('a'..='z').map(Key::Char));
    all
}

/// Every press the §11 table binds, as an operator types it, paired with the
/// intent it lands. The [`Held::Nothing`] plane is the whole table: suppression
/// decides *when* a press is read, never how it is spelled.
pub(crate) fn bindings() -> Vec<(String, KeyAction)> {
    let mut found = Vec::new();
    for mods in PLANES {
        for key in keys() {
            if let Some(action) = keymap(key, mods, Held::Nothing) {
                found.push((press(key, mods), action));
            }
        }
    }
    found
}

/// Every spelling the table offers, deduplicated — the vocabulary a hover may
/// name (§11 discoverability rule 3). Order is the table's, not sorted: the
/// bare plane first, then the two combo planes.
pub(crate) fn spellings() -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (press, _) in bindings() {
        if !found.contains(&press) {
            found.push(press);
        }
    }
    found
}

/// One press, spelled the way §11 writes it: a bare letter or digit wears the
/// doc's parentheses (`(f)`), a named key stands alone (`Enter`, `Escape`, `↑`)
/// because it is already a word, and a combo is written `Ctrl+N` — egui's
/// command modifier, so ⌘ on macOS reads as the same row.
fn press(key: Key, mods: Mods) -> String {
    let name = name_of(key);
    match mods {
        Mods::Bare if name.len() == 1 => format!("({name})"),
        Mods::Bare => name,
        Mods::Command => format!("Ctrl+{}", name.to_uppercase()),
        Mods::CommandShift => format!("Ctrl+Shift+{}", name.to_uppercase()),
    }
}

/// A key's own name. One byte wide for exactly the keys §11 parenthesizes.
fn name_of(key: Key) -> String {
    match key {
        Key::Up => "↑".to_owned(),
        Key::Down => "↓".to_owned(),
        Key::Left => "←".to_owned(),
        Key::Right => "→".to_owned(),
        Key::Enter => "Enter".to_owned(),
        Key::Escape => "Escape".to_owned(),
        Key::Digit(n) => n.to_string(),
        Key::Char(c) => c.to_string(),
        Key::Plus => "+".to_owned(),
        Key::Minus => "-".to_owned(),
    }
}
