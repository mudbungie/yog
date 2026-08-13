//! Tests for the jsonview widget.
//!
//! [`flatten`] and [`toggle_path`] are pure and unit-tested directly; the
//! render fn is shape-walked headlessly (per the transcript/git_tree render
//! pattern) and its collapse toggle is exercised by a simulated pointer click
//! — the one interaction that lives in a covered widget module (see the
//! module doc), verified end-to-end here.

use std::collections::HashSet;

use serde_json::json;

use super::{Node, Row, flatten, render, toggle_path};

fn collapsed(paths: &[&str]) -> HashSet<String> {
    paths.iter().map(std::string::ToString::to_string).collect()
}

#[test]
fn flatten_scalar_root_is_a_single_row() {
    let rows = flatten(&json!(42), "", &collapsed(&[]));
    assert_eq!(
        rows,
        vec![Row {
            depth: 0,
            label: "$".into(),
            node: Node::Scalar("42".into()),
            path: String::new(),
            collapsed: false,
        }]
    );
}

#[test]
fn flatten_walks_object_keys_and_array_indices_in_order() {
    let value = json!({"a": 1, "b": [true, "x"]});
    let rows = flatten(&value, "r", &collapsed(&[]));
    let shape: Vec<(usize, &str, &Node, &str)> = rows
        .iter()
        .map(|r| (r.depth, r.label.as_str(), &r.node, r.path.as_str()))
        .collect();
    assert_eq!(
        shape,
        vec![
            (0, "$", &Node::Object(2), "r"),
            (1, "a", &Node::Scalar("1".into()), "r/a"),
            (1, "b", &Node::Array(2), "r/b"),
            (2, "[0]", &Node::Scalar("true".into()), "r/b/0"),
            (2, "[1]", &Node::Scalar("\"x\"".into()), "r/b/1"),
        ]
    );
}

#[test]
fn collapsed_container_hides_its_descendants_but_keeps_its_own_row() {
    let value = json!({"a": {"deep": 1}});
    // Collapse the nested object at path "r/a": its own row stays, its child
    // "deep" is gone.
    let rows = flatten(&value, "r", &collapsed(&["r/a"]));
    let paths: Vec<&str> = rows.iter().map(|r| r.path.as_str()).collect();
    assert_eq!(paths, vec!["r", "r/a"]);
    let collapsed_row = rows.iter().find(|r| r.path == "r/a").unwrap();
    assert!(collapsed_row.collapsed);
    assert_eq!(collapsed_row.node, Node::Object(1));
}

#[test]
fn scalars_are_never_marked_collapsed_even_if_path_is_in_the_set() {
    // A stale path pointing at a scalar must not flip its collapsed flag —
    // only containers collapse.
    let rows = flatten(&json!({"a": 1}), "r", &collapsed(&["r/a"]));
    let scalar = rows.iter().find(|r| r.path == "r/a").unwrap();
    assert!(!scalar.collapsed);
}

#[test]
fn empty_containers_flatten_to_one_row() {
    assert_eq!(
        flatten(&json!({}), "r", &collapsed(&[])),
        vec![Row {
            depth: 0,
            label: "$".into(),
            node: Node::Object(0),
            path: "r".into(),
            collapsed: false,
        }]
    );
    assert_eq!(
        flatten(&json!([]), "r", &collapsed(&[]))[0].node,
        Node::Array(0)
    );
}

#[test]
fn root_namespace_prefixes_every_path() {
    let rows = flatten(&json!({"k": 1}), "resp/3", &collapsed(&[]));
    assert_eq!(rows[0].path, "resp/3");
    assert_eq!(rows[1].path, "resp/3/k");
}

#[test]
fn toggle_path_inserts_then_removes() {
    let mut set = HashSet::new();
    toggle_path(&mut set, "r/a");
    assert!(set.contains("r/a"));
    toggle_path(&mut set, "r/a");
    assert!(!set.contains("r/a"));
}

// --- headless render shape-walk ------------------------------------------

use crate::paint_probe::{paint, screen};

/// Render once and concatenate every painted galley's text.
fn painted(value: &serde_json::Value, collapsed: &mut HashSet<String>) -> String {
    paint(|ui| render(ui, value, "", collapsed))
}

#[test]
fn render_paints_labels_scalars_and_container_summaries() {
    let value = json!({"a": 1, "b": ["x"]});
    let mut set = HashSet::new();
    let text = painted(&value, &mut set);
    assert!(text.contains("object 2 keys"), "got:\n{text}");
    assert!(text.contains("array 1 items"), "got:\n{text}");
    assert!(text.contains("a:"));
    assert!(text.contains('1'));
    assert!(text.contains("\"x\""));
    // Expanded container shows the expand glyph, not the collapsed one.
    assert!(text.contains('▼'));
    assert!(!text.contains('▶'));
}

#[test]
fn collapsed_container_paints_the_collapsed_glyph_and_hides_children() {
    // Collapse the root object (path "" under the empty namespace).
    let mut set = collapsed(&[""]);
    let text = painted(&json!({"a": 1}), &mut set);
    assert!(text.contains('▶'), "got:\n{text}");
    assert!(
        !text.contains("a:"),
        "collapsed root hides children:\n{text}"
    );
}

/// Two-frame pointer click at `pos`: frame one lays the widget out (egui
/// hit-tests against the previous frame's rects), frame two delivers the
/// click.
fn click_root_toggle(value: &serde_json::Value, collapsed: &mut HashSet<String>) {
    let ctx = egui::Context::default();
    let run = |input: egui::RawInput, collapsed: &mut HashSet<String>| {
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| render(ui, value, "", collapsed));
        });
    };
    run(screen(), collapsed);
    // The root container's toggle is the first widget in the first row, at the
    // panel's top-left (depth 0 draws no indent) — a click at (12, 12) lands
    // on the glyph.
    let pos = egui::Pos2::new(12.0, 12.0);
    let click = egui::RawInput {
        events: vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            },
        ],
        ..screen()
    };
    run(click, collapsed);
}

#[test]
fn clicking_a_container_toggle_collapses_then_expands_it() {
    let value = json!({"a": 1});
    let mut set = HashSet::new();
    // First click collapses the root (path "" under the empty namespace).
    click_root_toggle(&value, &mut set);
    assert!(set.contains(""), "click should collapse the root: {set:?}");
    // Second click expands it again.
    click_root_toggle(&value, &mut set);
    assert!(!set.contains(""), "second click should expand: {set:?}");
}
