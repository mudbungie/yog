//! **The table itself** — the four resolution arms behind
//! [`keymap`](super::keymap): the bare plane as whatever holds the keyboard
//! leaves it, the unmodified bindings, and the two Command planes. Split from
//! [`super`] at §12's budget on the seam that module's own doc already drew:
//! the **vocabulary** a press is spelled in ([`Key`], [`Mods`], [`Held`],
//! [`KeyAction`]) is one subject, and the mapping between them is another.

use super::{CenterTab, Held, InspectorTab, Key, KeyAction, Mods, ZoomStep};

/// The §11 keymap: a press — a logical [`Key`] on a modifier plane, with
/// [`Held`] telling what currently holds the keyboard — to its gesture intent,
/// or `None` for an unbound press. Pure and total.
///
/// The suppression rule is [`bare_plane`] and lives nowhere else: the bare
/// plane bends to what holds the keyboard, a combo never does (so it still
/// works where it is most wanted). §11 rule 3 is what keeps the unsuppressed
/// plane safe — no combo fires a verb at the selection.
pub fn keymap(key: Key, mods: Mods, held: Held) -> Option<KeyAction> {
    match mods {
        Mods::Bare => bare_plane(key, held),
        Mods::Command => command(key),
        Mods::CommandShift => command_shift(key),
    }
}

/// The bare plane as whatever holds the keyboard leaves it (§11): the whole
/// table with the keyboard free, nothing at all under a text box, and — under a
/// modal — the one gesture aimed at the modal itself.
fn bare_plane(key: Key, held: Held) -> Option<KeyAction> {
    match held {
        Held::Nothing => bare(key),
        Held::TextBox => None,
        Held::Modal => matches!(key, Key::Escape).then_some(KeyAction::DismissModal),
    }
}

/// The unmodified table: arrows, Enter/Esc, the tab digits, and one letter per
/// altitude gesture (§11 rule 1).
fn bare(key: Key) -> Option<KeyAction> {
    Some(match key {
        Key::Up => KeyAction::ListPrev,
        Key::Down => KeyAction::ListNext,
        Key::Left => KeyAction::CollapseRow,
        Key::Right => KeyAction::ExpandRow,
        Key::Enter => KeyAction::Fire,
        Key::Escape => KeyAction::Cancel,
        Key::Digit(n) => KeyAction::Tab(InspectorTab::from_digit(n)?),
        Key::Char('i') => KeyAction::FocusComposer,
        Key::Char('n') => KeyAction::NewConversation,
        Key::Char('w') => KeyAction::NewWorkspace,
        Key::Char('s') => KeyAction::StartHead,
        Key::Char('x') => KeyAction::Stop,
        Key::Char('f') => KeyAction::Scan,
        Key::Char('c') => KeyAction::CloseBall,
        Key::Char('r') => KeyAction::ReleaseBall,
        Key::Char('b') => KeyAction::ToggleBalls,
        Key::Char('m') => KeyAction::ToggleModelPicker,
        Key::Char('g') => KeyAction::ToggleGrouping,
        Key::Char('a') => KeyAction::ToggleActivity,
        // Bare `+`/`-` are typing, never a gesture: zoom is combo-only.
        Key::Char(_) | Key::Plus | Key::Minus => return None,
    })
}

/// The Command plane: only what is safe to fire mid-typing (§11 rule 3) — the
/// list walk's continuation and the unfold beside it, the tab digits, the
/// composer focus, the everyday new, and the two panel folds.
fn command(key: Key) -> Option<KeyAction> {
    Some(match key {
        Key::Up => KeyAction::ListPrev,
        Key::Down => KeyAction::ListNext,
        Key::Left => KeyAction::CollapseRow,
        Key::Right => KeyAction::ExpandRow,
        Key::Plus => KeyAction::Zoom(ZoomStep::In),
        Key::Minus => KeyAction::Zoom(ZoomStep::Out),
        // Ctrl+0 is the browsers' reset, and digit 0 names no tab — so the
        // zoom arm takes it before the tab arm, which keeps 1–5 untouched.
        Key::Digit(0) => KeyAction::Zoom(ZoomStep::Reset),
        Key::Digit(n) => KeyAction::Tab(InspectorTab::from_digit(n)?),
        Key::Char('i') => KeyAction::FocusComposer,
        Key::Char('n') => KeyAction::NewConversation,
        Key::Char('b') => KeyAction::ToggleBalls,
        Key::Char('f') => KeyAction::Search,
        Key::Char('g') => KeyAction::ToggleGrouping,
        Key::Char('j') => KeyAction::ToggleActivity,
        _ => return None,
    })
}

/// The Command+Shift plane: the two "others". The **other new** — the
/// workspace raise, which Ctrl+W would have letter-matched and must not (§11
/// deliberately unbound) — and the **other tab strip**, the §11 center tabs
/// (bl-1ca2), since Command+digit is already spoken for by the altitude-2
/// inspector. Plus zoom-in, because on most layouts `+` *is* Shift+`=`, so the
/// gesture arrives here as often as on the bare Command plane.
fn command_shift(key: Key) -> Option<KeyAction> {
    match key {
        Key::Char('n') => Some(KeyAction::NewWorkspace),
        Key::Plus => Some(KeyAction::Zoom(ZoomStep::In)),
        Key::Digit(n) => CenterTab::from_digit(n).map(KeyAction::Center),
        _ => None,
    }
}
