//! The workspace tab-bar view-model (DESIGN §11 altitude 0, §15 Z9).
//!
//! Workspaces are regime walls — almost invisible: one tab per **named**
//! workspace under the top right, pinned first (in pin order), then name
//! order. Foreign and replay workspaces are real but not regimes, so they
//! live behind the overflow menu rather than widening the wall row; pinning
//! hoists one into the tabs. Pure over injected facts; the shell paints the
//! [`TabBar`] and the `new` name form beside it.

use crate::binding::{Workspace, WorkspaceKind};
use crate::nav::ws_key;

/// The per-workspace facts a tab is built from — derived by the caller
/// ([`AppModel::tab_bar`](crate::AppModel::tab_bar)) from the classification +
/// attention rollup.
#[derive(Debug, Clone)]
pub struct Item {
    pub ws: Workspace,
    pub attention: usize,
}

/// Which of the §3.1 kinds a tab stands for — [`WorkspaceKind`] without its
/// payload (the tab's `name` already carries the minted name). **One field, not
/// a pair of `named`/`replay` bools**: bools admit a state that cannot exist and
/// no match over them is exhaustive, which is exactly what the §11 badge-seat
/// pattern forbids. The §3.6 delete seat reads it too (yog may not delete what
/// it did not place, and the tab menu is pointer-targeted, so the fact rides the
/// tab rather than the focus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Named,
    Foreign,
    Replay,
}

impl Kind {
    /// The §3.1 classification, payload dropped.
    fn of(kind: &WorkspaceKind) -> Self {
        match kind {
            WorkspaceKind::Named { .. } => Self::Named,
            WorkspaceKind::Foreign => Self::Foreign,
            WorkspaceKind::Replay => Self::Replay,
        }
    }

    /// **The mark a tab wears for its kind** — the §11 badge-seat pattern's one
    /// home for glyph *and* words, here collapsed into a single label because
    /// this seat has no hue and paints one string (the pattern's tuple exists so
    /// a seat can colour the glyph and hover the words; nothing here does).
    /// Exhaustive, so a fourth kind must decide what it says and cannot ship a
    /// glyph alone — the failure the bare `▶` suffix was (bl-9a01: `▶` reads
    /// "play", not "replay"). `Named` is the ordinary regime: an unmarked tab
    /// *is* a named workspace, so there is nothing to say. `Foreign` takes words
    /// with no glyph — no convention reads "another tool's workspace", and a
    /// glyph no first-time reader can read is worse than none.
    pub fn mark(self) -> Option<&'static str> {
        match self {
            Self::Named => None,
            Self::Foreign => Some("foreign"),
            Self::Replay => Some("⏮ replay"),
        }
    }
}

/// One tab or overflow entry (§11): the workspace **name** — the operator's own
/// name, or the auto-id leaf for foreign/replay, which §3.1 makes the identity
/// either way — its attention count, whether it is the focused workspace,
/// whether it is pinned (hoisted), and which §3.1 [`Kind`] it is.
///
/// **It carries no path** (REMOTE §9.7 class 2, bl-7407). The name is what a
/// click hands back to the focus, what a gesture addresses and what a reply
/// would spell, so a tab that also carried a path would be a second source for
/// one fact the moment these rows come off the wire. The two doors that need a
/// path resolve at the click: the focus ([`AppModel::focus_workspace`]) and
/// the pin key ([`AppModel::toggle_pin`], which resolves it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub name: String,
    pub attention: usize,
    pub selected: bool,
    pub pinned: bool,
    pub kind: Kind,
}

impl Tab {
    /// What this entry says about its kind, ready to paint after the name:
    /// ` · ⏮ replay`, ` · foreign`, or nothing for a named workspace. One home
    /// for the rendering as well as the wording, so a hoisted tab and its
    /// overflow row (bl-7e32: an entry appears in both) read identically.
    pub fn kind_suffix(&self) -> String {
        self.kind
            .mark()
            .map(|mark| format!(" · {mark}"))
            .unwrap_or_default()
    }
}

/// The full derived tab bar (§11): the wall-row `tabs` (pinned hoists + named
/// workspaces) and the `overflow` menu entries (**every** foreign + replay
/// workspace, pinned or not).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabBar {
    pub tabs: Vec<Tab>,
    pub overflow: Vec<Tab>,
}

impl TabBar {
    /// The overflow button's aggregate attention badge (§11): the sum over the
    /// entries **still folded away**, so a stirring foreign workspace is visible
    /// while hidden — and a pinned one, which now also lists here (the ★ fix
    /// below), is not counted twice against its own visible tab badge.
    pub fn overflow_attention(&self) -> usize {
        self.overflow
            .iter()
            .filter(|t| !t.pinned)
            .map(|t| t.attention)
            .sum()
    }
}

/// Build the §11 tab bar: pinned first (pin order), then named workspaces
/// (name order); foreign and replay entries into the overflow (path order — the
/// enumeration's derived order, I9). `focused` marks selection.
///
/// **A pinned foreign/replay entry appears in both** (bl-7e32): pinning changes
/// where an entry *also* appears, never where it lives, so the overflow keeps
/// listing it with `pinned` set — that is what makes the menu's ★ the visible
/// pin/unpin toggle and the tab's context-menu unpin a mere accelerator (§11
/// context-menu doctrine, which a menu-only unpin violated).
pub fn build(items: &[Item], pinned: &[String], focused: Option<&str>) -> TabBar {
    let mut tabs: Vec<Tab> = Vec::new();
    for key in pinned {
        if let Some(it) = items.iter().find(|it| ws_key(&it.ws.path) == *key) {
            tabs.push(tab(it, focused, true));
        }
    }
    let mut named: Vec<Tab> = items
        .iter()
        .filter(|it| !is_pinned(it, pinned) && Kind::of(&it.ws.kind) == Kind::Named)
        .map(|it| tab(it, focused, false))
        .collect();
    named.sort_by(|a, b| a.name.cmp(&b.name));
    tabs.extend(named);
    let overflow = items
        .iter()
        .filter(|it| Kind::of(&it.ws.kind) != Kind::Named)
        .map(|it| tab(it, focused, is_pinned(it, pinned)))
        .collect();
    TabBar { tabs, overflow }
}

fn is_pinned(it: &Item, pinned: &[String]) -> bool {
    pinned.iter().any(|k| *k == ws_key(&it.ws.path))
}

/// Project one [`Item`] to a [`Tab`]. The name is the path leaf, which §3.1
/// makes the workspace's identity — [`crate::naming::leaf`] is the one
/// spelling of that, shared with the boundary's addressing so a tab and a
/// gesture cannot disagree about what this workspace is called.
fn tab(it: &Item, focused: Option<&str>, pinned: bool) -> Tab {
    let name = crate::naming::leaf(&it.ws.path);
    Tab {
        selected: focused == Some(name.as_str()),
        name,
        attention: it.attention,
        pinned,
        kind: Kind::of(&it.ws.kind),
    }
}

#[cfg(test)]
mod tests;
