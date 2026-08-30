//! The §11 context-menu roster — the pure seat table (DESIGN §11
//! "Context-menu doctrine", bl-ef89; the conversation-row and ball-row seats
//! bl-7e32).
//!
//! A context menu is an **accelerator surface**: it carries object-scoped verbs
//! that already exist on a visible affordance or a key, retargeted at the row
//! under the pointer. The doctrine's test is *every verb must survive
//! context-menu deletion* — delete every menu and the UI loses clicks, never
//! capabilities — so an entry here is never a verb's sole carrier, and the
//! [`Entry::carrier`] each one names is that claim written down where a test can
//! read it.
//!
//! The roster is **closed**: extension is governed by the rule above, not by
//! taste. Seats are added as [`Seat`] variants and their verbs as [`Verb`]
//! variants; the shell's one dispatch maps each verb to the *same call its
//! visible carrier makes*, which is how "a destructive verb reached through a
//! context menu opens the §3.6 confirmation exactly as its visible carrier does"
//! holds by construction rather than by review.
//!
//! **Enablement is not re-decided here.** A ball-row entry is offered exactly
//! where its `bl` verb is offered beside the composer, because it reads the same
//! [`crate::actions`] predicate the button's `add_enabled` reads — one rule, two
//! surfaces, no drift.

use crate::actions::{assign_enabled, close_enabled, unclaim_enabled};
use crate::projects::join::JoinState;

/// A verb a context menu carries. Every variant has a visible carrier (see
/// [`Entry::carrier`]); the menu only saves the click of reaching it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// The §3.6 unmaking — **opens the typed-name confirmation**, exactly as the
    /// config-mode danger row does. A menu accelerates reaching the dialog,
    /// never past it (§11: no destructive verb fires from a menu).
    DeleteWorkspace,
    /// Drop a pinned hoist (§11 pin/unpin). The overflow menu's ★ is the
    /// visible toggle; the tab's entry is the accelerator.
    Unpin,
    /// `litany stop <ws> <root>` (+ `--stop-children`) on the conversation under
    /// the pointer (§8.2) — the composer's Stop, retargeted off the selection.
    Stop { children: bool },
    /// `litany scan <ws>` — flush the workspace's undelivered mail (§8.2).
    Flush,
    /// The §3.6 class one conversation deep (bl-f17a) — **opens the
    /// confirmation dialog**, exactly as its visible carrier does. A menu
    /// accelerates reaching the dialog, never past it.
    DeleteAgent,
    /// `bl claim <id> --as <workspace>` — bind a ready ball (§8.2). Carries its
    /// destination, so the dispatch resolves no name of its own.
    Assign(String),
    /// `bl unclaim <id> --as <claimant>` — release a bound ball (§8.2).
    Release,
    /// `bl close <id> --as <claimant>` — deliver a bound ball (§8.2).
    CloseBall,
}

/// One rendered menu row: its worded label, the visible affordance that carries
/// the same verb (the doctrine's claim, per entry), and the verb clicking it
/// fires. A row is a button and nothing else — the roster is flat, so nothing
/// can hide behind a nested level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub label: String,
    pub carrier: String,
    pub verb: Verb,
}

/// The object a menu was opened on — the §11 closed seat roster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seat {
    /// A workspace tab. `named` decides the §3.6 entry (named workspaces only —
    /// yog may not delete what it did not place); `pinned` the unpin hoist.
    WorkspaceTab { named: bool, pinned: bool },
    /// A conversation row. `stoppable` is [`crate::actions::stop_enabled`] on
    /// **this row's** root (not the selection — the menu is pointer-targeted),
    /// `has_children` is [`crate::actions::stop_children_offered`] on it, and
    /// `named` gates the §3.6 delete entry — the workspace-scope rule the
    /// workspace tab's own entry reads (named workspaces only).
    ConversationRow {
        stoppable: bool,
        has_children: bool,
        named: bool,
    },
    /// A ball row — a ready one in the start affordances or a bound one in the
    /// balls section. `state` is the §3.5 join state the enablement predicates
    /// read and `assign_to` the focused workspace an Assign would bind to
    /// (`None` when none is focused).
    BallRow {
        state: JoinState,
        assign_to: Option<String>,
    },
}

/// The entries a seat carries, in render order. An **empty** result means the
/// object has no menu at all — the shell paints none rather than an empty popup.
pub fn entries(seat: Seat) -> Vec<Entry> {
    match seat {
        Seat::WorkspaceTab { named, pinned } => workspace_tab(named, pinned),
        Seat::ConversationRow {
            stoppable,
            has_children,
            named,
        } => conversation_row(stoppable, has_children, named),
        Seat::BallRow { state, assign_to } => ball_row(state, assign_to),
    }
}

/// The workspace tab's seat (§11 roster row 1).
fn workspace_tab(named: bool, pinned: bool) -> Vec<Entry> {
    let mut out = Vec::new();
    if named {
        out.push(fires(
            "delete this workspace…",
            "config mode's per-workspace danger row",
            Verb::DeleteWorkspace,
        ));
    }
    if pinned {
        out.push(fires(
            "unpin",
            "the overflow menu's ★ toggle, lit while pinned",
            Verb::Unpin,
        ));
    }
    out
}

/// The conversation row's seat (§11 roster row 2): Stop (+children), the
/// composer's selection-targeted affordance aimed at the row instead, Flush —
/// the Inbox tab's Scan button, retargeted at the row under the pointer — and,
/// on a named workspace, the §3.6 delete entry last (the danger-zone tail).
/// Flush is workspace-scoped and unconditional, so this seat is never empty.
fn conversation_row(stoppable: bool, has_children: bool, named: bool) -> Vec<Entry> {
    let mut out = Vec::new();
    if stoppable {
        out.push(fires(
            "stop",
            "the composer's Stop button and the `x` key",
            Verb::Stop { children: false },
        ));
        if has_children {
            out.push(fires(
                "stop + children",
                "the composer's Stop with its `children` checkbox",
                Verb::Stop { children: true },
            ));
        }
    }
    out.push(fires(
        "flush the inbox",
        "the Inbox tab's Scan button and the `f` key",
        Verb::Flush,
    ));
    if named {
        out.push(fires(
            "delete this conversation…",
            "the inspector Config tab's per-conversation danger row",
            Verb::DeleteAgent,
        ));
    }
    out
}

/// The ball row's seat (§11 roster row 3): Assign / Release / Close, each
/// offered exactly where the composer's own button is enabled (§8.2/§3.5).
fn ball_row(state: JoinState, assign_to: Option<String>) -> Vec<Entry> {
    let mut out = Vec::new();
    if let Some(to) = assign_to.filter(|_| assign_enabled(state)) {
        out.push(fires(
            &format!("assign → {to}"),
            "the ready ball row's `assign → <workspace>` button",
            Verb::Assign(to),
        ));
    }
    if unclaim_enabled(state) {
        out.push(fires(
            "release",
            "the composer's ball row Release button and the `r` key",
            Verb::Release,
        ));
    }
    if close_enabled(state) {
        out.push(fires(
            "close",
            "the composer's ball row Close button and the `c` key",
            Verb::CloseBall,
        ));
    }
    out
}

fn fires(label: &str, carrier: &str, verb: Verb) -> Entry {
    Entry {
        label: label.to_owned(),
        carrier: carrier.to_owned(),
        verb,
    }
}

#[cfg(test)]
mod tests;
