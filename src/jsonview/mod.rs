//! Collapsible JSON row tree + a thin egui render — the uniform "every byte
//! inspectable" inspector widget (DESIGN §5.1 #13, §11 Altitude-2 Steps tab).
//!
//! A hand-rolled, zero-dependency `serde_json::Value → row tree`: [`flatten`]
//! projects a value into display rows in document order (pure, fully
//! testable), and [`render`] draws them as an indented monospace tree with a
//! collapse toggle per container. Reused everywhere a raw machine record is
//! shown — the Y13 steps inspector here, config-branch and message views
//! later — so JSON always reads the same way.
//!
//! Collapse state is a per-instance `HashSet<String>` **owned by the caller**
//! (DESIGN §5.3 / §13.1): which nodes you have expanded is viewport ephemera
//! — *which data you look at*, not data — so it lives in RAM and re-derives
//! at startup (an empty set = fully expanded), never in durable `ui.json`.
//! The set is keyed by each node's path from the tree root; the caller passes
//! a unique `root` namespace per rendered tree so several trees sharing one
//! set never collide (the path is the node's identity — single source, no
//! per-tree index).
//!
//! Widget-split note (§11): every other render fn keeps `.clicked()` in the
//! excluded shell because clicks read as unreachable headless. This widget is
//! the exception — its collapse toggle is intrinsic and its layout is
//! deterministic, so the click *is* headless-testable (see the render tests),
//! and the toggle lives here where the widget is reused, not smeared across
//! every caller. The split's purpose — full test coverage — is met either way.

use std::collections::HashSet;

use serde_json::Value;

/// Two spaces of indent per depth level (matches the git_tree tree view).
const INDENT: &str = "  ";
/// The root node's label — JSONPath's root token, so a whole-tree collapse
/// reads unambiguously.
const ROOT_LABEL: &str = "$";
/// Leaf marker: a non-clickable placeholder keeping scalar rows aligned
/// under the container toggles above them.
const SCALAR_MARK: &str = "·";
/// Container toggle glyphs — collapsed (children hidden) / expanded. The
/// crate's one home for the disclosure-fold vocabulary (§11 glyph doctrine:
/// the triangles pass on convention), shared with the transcript row folds.
pub(crate) const GLYPH_COLLAPSED: &str = "▶";
pub(crate) const GLYPH_EXPANDED: &str = "▼";

/// One flattened row of a JSON tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Nesting depth (root = 0) — drives the render indent.
    pub depth: usize,
    /// This node's object key, array index (`[i]`), or the root label.
    pub label: String,
    /// The node's scalar preview or container summary.
    pub node: Node,
    /// Stable collapse-state key: the node's path from the tree root, under
    /// the caller's `root` namespace. A container is collapsed iff this
    /// string is in the caller's collapsed set.
    pub path: String,
    /// `true` iff this is a container whose `path` is in the collapsed set;
    /// scalars are never collapsed.
    pub collapsed: bool,
}

/// A JSON node projected for one row: a formatted scalar, or a container with
/// its child count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// A leaf value, compact-JSON formatted (`42`, `"hi"`, `true`, `null`).
    Scalar(String),
    /// An object with this many keys.
    Object(usize),
    /// An array with this many items.
    Array(usize),
}

/// Flatten `value` into display rows in document order. `root` is the
/// caller-unique path namespace for this tree; every row's `path` is prefixed
/// with it. A container whose path is in `collapsed` contributes its own row
/// but none of its descendants (the collapse boundary).
pub fn flatten(value: &Value, root: &str, collapsed: &HashSet<String>) -> Vec<Row> {
    let mut rows = Vec::new();
    push_rows(value, ROOT_LABEL, root, 0, collapsed, &mut rows);
    rows
}

fn push_rows(
    value: &Value,
    label: &str,
    path: &str,
    depth: usize,
    collapsed: &HashSet<String>,
    rows: &mut Vec<Row>,
) {
    let node = node_of(value);
    let is_container = !matches!(node, Node::Scalar(_));
    let collapsed_here = is_container && collapsed.contains(path);
    rows.push(Row {
        depth,
        label: label.to_string(),
        node,
        path: path.to_string(),
        collapsed: collapsed_here,
    });
    if collapsed_here {
        return;
    }
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                push_rows(
                    child,
                    key,
                    &format!("{path}/{key}"),
                    depth + 1,
                    collapsed,
                    rows,
                );
            }
        }
        Value::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                push_rows(
                    child,
                    &format!("[{i}]"),
                    &format!("{path}/{i}"),
                    depth + 1,
                    collapsed,
                    rows,
                );
            }
        }
        _ => {}
    }
}

/// Project one value into its row [`Node`]: containers carry their length;
/// every scalar is its compact-JSON text (`Value`'s Display), so strings keep
/// their quotes and `null`/`true`/numbers read verbatim.
fn node_of(value: &Value) -> Node {
    match value {
        Value::Object(map) => Node::Object(map.len()),
        Value::Array(items) => Node::Array(items.len()),
        scalar => Node::Scalar(scalar.to_string()),
    }
}

/// Render `value` as an indented, collapsible monospace tree. `root` is the
/// path namespace (see [`flatten`]); `collapsed` is the caller-owned
/// per-instance collapse state. Clicking a container's toggle flips its path
/// in `collapsed` (taking effect on the next frame's re-flatten).
pub fn render(ui: &mut egui::Ui, value: &Value, root: &str, collapsed: &mut HashSet<String>) {
    for row in flatten(value, root, collapsed) {
        render_row(ui, &row, collapsed);
    }
}

fn render_row(ui: &mut egui::Ui, row: &Row, collapsed: &mut HashSet<String>) {
    ui.horizontal(|ui| {
        if row.depth > 0 {
            ui.monospace(INDENT.repeat(row.depth));
        }
        match &row.node {
            Node::Scalar(text) => {
                ui.monospace(SCALAR_MARK);
                ui.monospace(format!("{}:", row.label));
                ui.monospace(text);
            }
            Node::Object(len) => container_row(ui, row, collapsed, &format!("object {len} keys")),
            Node::Array(len) => container_row(ui, row, collapsed, &format!("array {len} items")),
        }
    });
}

fn container_row(ui: &mut egui::Ui, row: &Row, collapsed: &mut HashSet<String>, summary: &str) {
    let glyph = if row.collapsed {
        GLYPH_COLLAPSED
    } else {
        GLYPH_EXPANDED
    };
    // §11 discoverability: every control says what pressing it does, disclosure
    // triangles included — the glyph passes on convention, the hover says what
    // *this* one opens.
    let toggle = ui
        .add(egui::Label::new(egui::RichText::new(glyph).monospace()).sense(egui::Sense::click()))
        .on_hover_text(
            "Fold this record open or shut — its contents are shown below it. No key of \
             its own: Tab reaches it, Space presses it.",
        );
    if toggle.clicked() {
        toggle_path(collapsed, &row.path);
    }
    ui.monospace(format!("{}:", row.label));
    ui.monospace(summary);
}

/// Flip `path`'s membership in the collapse set — the one mutation the toggle
/// click performs. Pure and directly unit-tested so both arms stay covered.
pub(crate) fn toggle_path(collapsed: &mut HashSet<String>, path: &str) {
    if !collapsed.remove(path) {
        collapsed.insert(path.to_string());
    }
}

#[cfg(test)]
mod tests;
