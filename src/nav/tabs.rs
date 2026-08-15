//! The workspace tab-bar view-model (DESIGN §11 altitude 0, §15 Z9).
//!
//! Workspaces are regime walls — almost invisible: one tab per **named**
//! workspace under the top right, pinned first (in pin order), then name
//! order. Foreign and replay workspaces are real but not regimes, so they
//! live behind the overflow menu rather than widening the wall row; pinning
//! hoists one into the tabs. Pure over injected facts; the shell paints the
//! [`TabBar`] and the `new` name form beside it.
//!
//! **It folds an answer; it derives nothing** (REMOTE §9.7 class 2, bl-296f).
//! The bar is built out of [`WsRow`] — the `Query::Workspaces` reply, which has
//! carried the §6 rollups since bl-6233 and the §4.1 pin *rank* since this ball
//! — so the altitude-0 chrome is `nav::convs::visible`'s own shape one surface
//! over: the derivation is the boundary's, the ordering and the folding are the
//! seat's. It is the shape bl-7407 named as the one thing left between this bar
//! and the wire, and the reason it was refused until now was the join it would
//! have cost: painting off a reply while resolving each name back through the
//! engine's own table is two sources for one fact. Nothing here resolves
//! anything — a row arrives named, classified, counted and ranked.

use crate::binding::WorkspaceKind;
use crate::boundary::reply::WsRow;

/// The §6 attention-strip total (§11 altitude 0): attention-bearing agents
/// across every workspace, summed off the **same answer** the bar beside it is
/// built from — one standing question, two surfaces, and no chance of a strip
/// that counts a workspace the tabs do not show.
pub fn strip_total(rows: &[WsRow]) -> usize {
    rows.iter().map(|r| r.attention).sum()
}

/// **Where a bound ball can be re-homed** (§8.2 Move, REMOTE §9.7 bl-b4b5): the
/// answered enumeration's **named** workspaces minus the one that already holds
/// it. Foreign and replay workspaces carry no yog identity, so they are not move
/// targets.
///
/// One rule for the composer's `move to:` buttons, the §11 ball-row menu's
/// destination submenu and the board row's — so the visible carrier and its
/// accelerator can never offer different destinations, and neither ever offers a
/// move to where the ball already is. It was `AppModel::move_targets` over the
/// window's own workspace set; the fact is on this answer, so it is a selection
/// out of the same ask the tab bar above it already stands on.
pub fn move_targets(rows: &[WsRow], owner: &str) -> Vec<String> {
    rows.iter()
        .filter(|r| matches!(r.kind, WorkspaceKind::Named { .. }) && r.workspace != owner)
        .map(|r| r.workspace.clone())
        .collect()
}

/// **Whether the enumeration calls this workspace one of yog's own** (§3.1) —
/// the §3.6 scope gate every delete carrier reads before offering the verb.
/// `false` for a name the answer does not carry, which is what an unfetched or
/// deleted workspace already read as.
pub fn is_named(rows: &[WsRow], workspace: &str) -> bool {
    rows.iter()
        .any(|r| r.workspace == workspace && matches!(r.kind, WorkspaceKind::Named { .. }))
}

/// **The §2.2 config-lineage tip of one workspace** (§9.4) — the commit the next
/// conversation started here forks off, off the same answer. `None` for a name
/// the enumeration does not carry and for a workspace with no lineage derived
/// yet: both paint nothing, which is the same reading.
pub fn config_tip(rows: &[WsRow], workspace: &str) -> Option<crate::model_pick::ConfigTip> {
    rows.iter()
        .find(|r| r.workspace == workspace)?
        .config_tip
        .clone()
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
/// **A stale pin dissolves rather than being dropped** (bl-296f): a key naming
/// no enumerated workspace ranks no row, so there is nothing here to skip.
pub fn build(rows: &[WsRow], focused: Option<&str>) -> TabBar {
    let mut hoisted: Vec<&WsRow> = rows.iter().filter(|r| r.pinned.is_some()).collect();
    hoisted.sort_by_key(|r| r.pinned);
    let mut tabs: Vec<Tab> = hoisted.iter().map(|r| tab(r, focused)).collect();
    let mut named: Vec<Tab> = rows
        .iter()
        .filter(|r| r.pinned.is_none() && Kind::of(&r.kind) == Kind::Named)
        .map(|r| tab(r, focused))
        .collect();
    named.sort_by(|a, b| a.name.cmp(&b.name));
    tabs.extend(named);
    let overflow = rows
        .iter()
        .filter(|r| Kind::of(&r.kind) != Kind::Named)
        .map(|r| tab(r, focused))
        .collect();
    TabBar { tabs, overflow }
}

/// Project one answered row to a [`Tab`]. The name is the row's own — §3.1's
/// leaf, minted once at the boundary ([`crate::naming::leaf`]) and shared with
/// the addressing every gesture uses, so a tab and a gesture cannot disagree
/// about what this workspace is called.
fn tab(row: &WsRow, focused: Option<&str>) -> Tab {
    Tab {
        selected: focused == Some(row.workspace.as_str()),
        name: row.workspace.clone(),
        attention: row.attention,
        pinned: row.pinned.is_some(),
        kind: Kind::of(&row.kind),
    }
}

#[cfg(test)]
mod tests;
