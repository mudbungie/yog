//! The keyboard-navigation keymap (DESIGN §11): a **pure** key → intent table.
//!
//! Keyboard nav is split the same way every widget is (§12): a pure, tested
//! mapping here plus thin egui event plumbing in `src/shell/*` (coverage-
//! excluded) that lifts an `egui::Key` into a [`Key`] and dispatches the
//! resulting [`KeyAction`] to the matching [`AppModel`](crate::AppModel) call.
//! Nothing in this module touches egui, so every branch is table-tested.
//!
//! The bindings (§11 three altitudes): ↑/↓ step the focus through the focused
//! workspace's **visible conversation rows, in paint order** — the altitude-0/1
//! selection — ←/→ unfold the row that selection already names (bl-fa82), the
//! digit keys select an altitude-2 inspector tab, and a letter fires a verb on
//! the target that selection already names (§11's rule 1). Choosing *among*
//! several targets is a pointer gesture (rule 2), so nothing here needs a
//! second cursor.
//!
//! Since the focus ruling every selection lands the composer, so ↑/↓ are
//! paired on the Command plane too: a bare step surrenders the plane it was
//! pressed on, and Ctrl+↑/↓ is how the walk keeps going from inside the box.
//!
//! Every alphanumeric key is paired with a **combo** on the Command plane (⌘ on
//! macOS, Ctrl elsewhere) — except the five verbs that fire at the current
//! selection, which §11's rule 3 leaves deliberately unpaired. The suppression
//! rule lives here too, in [`keymap`]'s [`Held`] argument: a bare key is
//! skipped while a text box holds the keyboard, a combo is not, because working
//! mid-typing is the entire reason the combo plane exists.

/// A logical key the shell has already lifted out of an egui event (the
/// `egui::Key` → this translation is the excluded plumbing). `Digit` carries
/// the pressed digit's value (1–9) and `Char` the pressed letter (lowercase);
/// other keys are named directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    Digit(u8),
    Char(char),
    /// `+` (and the shift-free `=` most keyboards zoom in with, as browsers
    /// allow) — lifted to one logical key, since they mean one gesture.
    Plus,
    Minus,
}

/// A §11 zoom gesture's direction — one step of the whole-UI text size, or the
/// reset. The arithmetic is the model's ([`AppModel::zoom_nudge`](crate::AppModel::zoom_nudge));
/// the table names only the intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomStep {
    In,
    Out,
    Reset,
}

/// The modifier plane a press arrives on (§11). `Command` is egui's
/// `Modifiers::command` — ⌘ on macOS, Ctrl elsewhere — so one entry spells the
/// binding on both §10 targets. Anything the table does not model (a lone
/// Shift, Alt) lifts as [`Mods::Bare`], exactly as before combos existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mods {
    Bare,
    Command,
    CommandShift,
}

/// What holds the keyboard when a press arrives — the §11 suppression rule's
/// one input, and the whole of it.
///
/// [`Modal`](Self::Modal) is not "a text box, plus something": it is a
/// *different plane*. A modal owns the frame while it is up (nothing beneath it
/// is reachable by pointer or key), so the bare plane collapses to the one
/// gesture that acts on the modal itself — Escape, which dismisses it on the
/// **first** press. There is nothing left for suppression to protect: the box
/// the modal holds is inside the thing being dismissed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Held {
    /// Nothing — the bare-key plane is live.
    Nothing,
    /// A text box: bare keys are suppressed so typing never steals them.
    TextBox,
    /// A modal (§3.1's name form, §3.6's confirmation) owns the frame.
    Modal,
}

/// A gesture intent — the pure keymap's output, dispatched by the shell
/// (`src/shell/keys.rs`) to the same effect the matching widget's click calls
/// (§11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// ↑ / Ctrl+↑ — step the selection to the previous **visible** conversation
    /// row (bl-fa82).
    ListPrev,
    /// ↓ / Ctrl+↓ — step the selection to the next **visible** conversation row,
    /// in paint order: a collapsed subtree contributes one row, so the step
    /// skips it whole and never reveals anything (bl-fa82's ruling). It walked
    /// §6's attention-ranked roster across every workspace until then; the
    /// cross-wall walk is the jump's alone now.
    ///
    /// The combo is the walk's **continuation**: since the focus ruling a
    /// selection lands the composer whatever plane it rode, so a bare step
    /// surrenders its own plane and the second step needs one that survives text
    /// focus. Safe by rule 3 — stepping a selection repaints and fires no verb.
    ListNext,
    /// → / Ctrl+→ — unfold the selected conversation row's children into the
    /// list (bl-fa82). A verb on the target the selection already names (rule
    /// 1), and lawful on the combo plane by rule 3: it repaints a viewport and
    /// fires nothing.
    ExpandRow,
    /// ← / Ctrl+← — fold the selected row shut; on a row with nothing to
    /// collapse — a child, a leaf, an already-collapsed parent — it pages the
    /// selection **up to its parent row** instead, so `←` held down walks out
    /// of a descent the way `↑` walks up a list (bl-fa82).
    CollapseRow,
    /// A digit / Command+digit — select an inspector tab.
    Tab(InspectorTab),
    /// Command+Shift+digit — focus a §11 **center tab** (bl-1ca2). Combo-only
    /// and on the shifted plane: the plain Command+digit plane already carries
    /// the altitude-2 strip, and this is the other one. Lawful under rule 3 —
    /// focusing a tab repaints and fires no verb — which is what makes it
    /// reachable from inside the composer, where the keyboard rests.
    Center(CenterTab),
    /// `i` / Ctrl+I — hand the keyboard to whichever composer is painted,
    /// target unchanged.
    FocusComposer,
    /// `n` / Ctrl+N — `new conversation`: clear the agent selection, then focus
    /// the composer. The everyday new, so it earns the conventional combo.
    NewConversation,
    /// `w` / Ctrl+Shift+N — the `new` workspace raise: it opens the §11 name
    /// form for the deliberate sphere wall (§3.1/§3.4), spelled as the standard
    /// "other new", never Ctrl+W.
    NewWorkspace,
    /// `s` — ▶ Start the balls section's first ready row. No combo: it fires at
    /// the selection and spends a model call (§11 rule 3; Ctrl+S is save).
    StartHead,
    /// `x` — Stop the selected conversation (+children per its checkbox, §8.2).
    /// No combo (rule 3; Ctrl+X is the text box's cut).
    Stop,
    /// `f` — Flush the inbox: `lernie scan` on the focused workspace (§8.2).
    /// No combo (rule 3): Ctrl+F is [`Search`](Self::Search), and the two
    /// planes now carry the two meanings the letter has — the bare one
    /// mutates, the combo one only looks.
    Scan,
    /// Ctrl+F — the find reflex, answered (§8.5): open the composer on a
    /// `/search ` line. Combo-only, which is rule 3 satisfied rather than
    /// dodged — a query spends nothing, so it is safe mid-typing, and the bare
    /// `f` plane keeps the mutation it always had.
    Search,
    /// `c` — Close the focused conversation's bound ball (§8.2). No combo
    /// (rule 3; Ctrl+W's close reflex must never reach a `bl close`).
    CloseBall,
    /// `r` — Release (unclaim) the focused conversation's bound ball (§8.2).
    /// No combo (rule 3; Ctrl+R is reload, which yog has no concept of).
    ReleaseBall,
    /// `b` / Ctrl+B — fold / unfold the balls section (the persisted §4.1
    /// collapse); Ctrl+B is the editors' "toggle the side panel".
    ToggleBalls,
    /// `m` — open / close the §9.4 model picker for the focused workspace.
    /// Bare only: it is a verb at the selection, so §11 rule 3 keeps it off
    /// the combo plane.
    ToggleModelPicker,
    /// `g` / Ctrl+G — organizing view: recent ⇄ by ball (§15 Z9).
    ToggleGrouping,
    /// `a` / Ctrl+J — activity accessory: collapsed ⇄ expanded (§13.0);
    /// Ctrl+J is the bottom panel, and Ctrl+A belongs to the text box.
    ToggleActivity,
    /// Ctrl+`+` / Ctrl+`-` / Ctrl+`0` — the whole-UI text size (the persisted
    /// §4.1 `zoom`), the browser convention exactly. Combo-only: it repaints
    /// and nothing else (rule 3), and the bare `+`/`-`/`0` plane belongs to
    /// typing and to the tab digits.
    Zoom(ZoomStep),
    /// Enter — fire the pending start goal (the composer's Send, §8.1).
    Fire,
    /// Esc — the one **put-it-down** gesture (§11), aimed at whatever is
    /// currently up: the pending start goal, else a focused center tab, which
    /// it returns to the conversation (bl-1ca2 — QUALITY F3's "Escape
    /// dismisses", kept in tab form). With neither up it is the composer's own
    /// release, which is why it must not re-grab the box. Reaches the table
    /// only once egui has spent an Escape surrendering text focus.
    Cancel,
    /// Esc on the [`Held::Modal`] plane — dismiss the modal that owns the frame,
    /// dropping whatever was typed into it, and hand the keyboard back to the
    /// composer (§11 focus discipline).
    DismissModal,
}

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

mod center;
/// The table's own spellings (§11 rule 3). Read by the discoverability scan
/// and by nothing the window paints — a hover states its key in the seat's own
/// sentence — so it is compiled where its consumer is, and the tree carries no
/// second table a release build would have to strip.
#[cfg(test)]
pub(crate) mod spell;
mod tabs;
pub use center::CenterTab;
pub use tabs::InspectorTab;

#[cfg(test)]
mod tests;
