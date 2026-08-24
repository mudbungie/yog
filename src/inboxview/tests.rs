//! Tests for the Inbox view-model and its widget. This file holds the claims
//! about **bytes** — the forgiving envelope parse and the listing's file facts
//! (name + verbatim bytes). [`paint`] holds the claims about **glyphs**: the
//! two render modes, the parsed deposit and the §11 Raw toggle's unaltered
//! bytes. [`tail`] runs the same walk on a viewport too short for the backlog,
//! where the §11 tail anchor decides which deposits are on screen.

use super::render::render;
use super::*;
use crate::nav::convs::Titles;
use tempfile::tempdir;

mod tail;

/// The widget over a roster with nothing in it: every sender then lands on the
/// §3.3 ladder's floor, which is what these tables are about. The named rung is
/// asserted where a name can exist at all — the §11 window (bl-b6d0,
/// `shell::acceptance::naming`).
fn painted(entries: &[InboxEntry], raw: bool) -> String {
    crate::paint_probe::paint(|ui| render(ui, entries, &Titles::default(), raw))
}

/// One listing entry carrying `body` as its parsed deposit and `raw` as the
/// bytes behind it — the shape the shell hands the widget.
fn entry(name: &str, bytes: &str) -> InboxEntry {
    InboxEntry {
        name: name.to_string(),
        raw: bytes.as_bytes().to_vec(),
        deposit: parse_deposit(bytes.as_bytes()),
    }
}

#[test]
fn parses_ordinary_deposit() {
    let d = parse_deposit(b"---\nfrom: user\ndeposited_at: 2026-07-17T12:00:00Z\n---\nhello there");
    assert_eq!(d.sender.as_deref(), Some("user"));
    assert_eq!(d.deposited_at.as_deref(), Some("2026-07-17T12:00:00Z"));
    assert_eq!(d.epitaph, None);
    assert_eq!(d.terminal_ref, None);
    assert_eq!(d.body, "hello there");
}

#[test]
fn epitaph_values_map_to_typed_variants() {
    let cases = [
        ("final-response", Epitaph::FinalResponse),
        ("stopped", Epitaph::Stopped),
        ("budget-exhausted", Epitaph::BudgetExhausted),
        ("died", Epitaph::Died),
        ("wat", Epitaph::Unknown("wat".to_string())),
    ];
    for (raw, want) in cases {
        let file =
            format!("---\nfrom: c\ndeposited_at: t\nepitaph: {raw}\nterminal_ref: sha\n---\nbody");
        let d = parse_deposit(file.as_bytes());
        assert_eq!(d.epitaph, Some(want));
        assert_eq!(d.terminal_ref.as_deref(), Some("sha"));
    }
}

#[test]
fn result_message_without_body_has_empty_body() {
    let d =
        parse_deposit(b"---\nfrom: c\ndeposited_at: t\nepitaph: died\nterminal_ref: sha\n---\n");
    assert_eq!(d.epitaph, Some(Epitaph::Died));
    assert_eq!(d.body, "");
}

#[test]
fn malformed_file_is_raw_body_with_absent_fields() {
    let d = parse_deposit(b"no frontmatter at all");
    assert_eq!(
        d,
        Deposit {
            body: "no frontmatter at all".to_string(),
            ..Deposit::default()
        }
    );
}

#[test]
fn opener_without_closer_is_raw_body() {
    let d = parse_deposit(b"---\nfrom: x but no closing fence");
    assert_eq!(d.sender, None);
    assert!(d.body.starts_with("---\n"));
}

#[test]
fn frontmatter_line_without_separator_is_skipped() {
    let d = parse_deposit(b"---\nfrom: user\nbare line\n---\nbody");
    assert_eq!(d.sender.as_deref(), Some("user"));
    assert_eq!(d.body, "body");
}

#[test]
fn list_inbox_orders_by_filename_skipping_temp_nonmd_and_unreadable() {
    let dir = tempdir().unwrap();
    let inbox = dir.path().join("inbox").join("a-1");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(
        inbox.join("user-002.md"),
        b"---\nfrom: user\ndeposited_at: t2\n---\nsecond",
    )
    .unwrap();
    std::fs::write(
        inbox.join("user-001.md"),
        b"---\nfrom: user\ndeposited_at: t1\n---\nfirst",
    )
    .unwrap();
    // Atomic-rename temp dotfile, a non-`.md` file, and a directory
    // named like a deposit (its read fails) are all excluded.
    std::fs::write(inbox.join(".user-003.md.tmp"), b"partial").unwrap();
    std::fs::write(inbox.join("notes.txt"), b"not a deposit").unwrap();
    std::fs::create_dir_all(inbox.join("sub.md")).unwrap();
    let entries = list_inbox(dir.path(), "a-1");
    let bodies: Vec<&str> = entries.iter().map(|e| e.deposit.body.as_str()).collect();
    assert_eq!(bodies, vec!["first", "second"]);
}

#[test]
fn list_inbox_missing_dir_is_empty() {
    let dir = tempdir().unwrap();
    assert!(list_inbox(dir.path(), "nobody").is_empty());
}

/// S7-T1, Inbox half: the listing carries each deposit file's **name** and its
/// **verbatim bytes** beside the parse, so the Raw toggle has something
/// unaltered to show. The envelope the parsed view drops is still in `raw`.
#[test]
fn list_inbox_carries_each_files_name_and_verbatim_bytes() {
    let dir = tempdir().unwrap();
    let inbox = dir.path().join("inbox").join("a-1");
    std::fs::create_dir_all(&inbox).unwrap();
    let bytes = "---\nfrom: user\ndeposited_at: t1\n---\nthe body\n";
    std::fs::write(inbox.join("user-001.md"), bytes).unwrap();
    let entries = list_inbox(dir.path(), "a-1");
    assert_eq!(entries.len(), 1);
    let entry = entries.first().unwrap();
    assert_eq!(entry.name, "user-001.md");
    assert_eq!(entry.raw, bytes.as_bytes(), "bytes must be unaltered");
    assert_eq!(entry.deposit.body, "the body\n");
}

/// **What the widget paints** — the two §11 render modes; its own file per
/// §12's budget, on the seam between a claim about bytes and a claim about
/// glyphs.
mod paint;
