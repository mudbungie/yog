//! Tests for the Inbox view-model and its widget: the forgiving envelope
//! parse, the listing's file facts (name + verbatim bytes), and the two render
//! modes — the parsed deposit and the §11 Raw toggle's unaltered bytes.
//! [`tail`] runs the same walk on a viewport too short for the backlog, where
//! the §11 tail anchor decides which deposits are on screen.

use super::render::render;
use super::*;
use tempfile::tempdir;

mod tail;

fn painted(entries: &[InboxEntry], raw: bool) -> String {
    crate::paint_probe::paint(|ui| render(ui, entries, raw))
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

#[test]
fn empty_inbox_shows_placeholder() {
    assert!(painted(&[], false).contains("(no deposits)"));
    assert!(painted(&[], true).contains("(no deposits)"));
}

#[test]
fn ordinary_deposit_renders_header_and_body() {
    let e = entry(
        "user-001.md",
        "---\nfrom: user\ndeposited_at: 2026-07-17T12:00:00Z\n---\nplease continue",
    );
    let text = painted(std::slice::from_ref(&e), false);
    assert!(
        text.contains("✉ user · 2026-07-17T12:00:00Z"),
        "got:\n{text}"
    );
    assert!(text.contains("please continue"));
    // No epitaph line for a plain deposit.
    assert!(!text.contains("epitaph:"));
}

#[test]
fn result_message_renders_every_epitaph_and_terminal_ref() {
    let cases = [
        (Epitaph::FinalResponse, "final-response"),
        (Epitaph::Stopped, "stopped"),
        (Epitaph::BudgetExhausted, "budget-exhausted"),
        (Epitaph::Died, "died"),
        (Epitaph::Unknown("wat".into()), "wat"),
    ];
    for (epitaph, label) in cases {
        let e = InboxEntry {
            name: "c-1-001.md".into(),
            raw: Vec::new(),
            deposit: Deposit {
                sender: Some("c-1".into()),
                deposited_at: Some("t".into()),
                epitaph: Some(epitaph),
                terminal_ref: Some("deadbeef".into()),
                body: String::new(),
            },
        };
        let text = painted(std::slice::from_ref(&e), false);
        assert!(text.contains(&format!("epitaph: {label}")), "got:\n{text}");
        assert!(text.contains("terminal: deadbeef"));
    }
}

#[test]
fn absent_fields_render_as_question_marks() {
    let e = entry("odd.md", "raw");
    let text = painted(std::slice::from_ref(&e), false);
    assert!(text.contains("✉ ? · ?"), "got:\n{text}");
}

/// S7-T1, Inbox half: Raw flips the tab to each deposit file's name and its
/// bytes exactly as they sit on disk — envelope included, nothing summarized
/// away and no parsed header in the way.
#[test]
fn raw_mode_paints_each_files_name_and_unaltered_bytes() {
    let bytes = "---\nfrom: user\ndeposited_at: t1\n---\nthe body\n";
    let e = entry("user-001.md", bytes);
    let text = painted(std::slice::from_ref(&e), true);
    assert!(text.contains("user-001.md"), "no filename header:\n{text}");
    assert!(text.contains(bytes), "bytes not verbatim:\n{text}");
    assert!(!text.contains("✉ user"), "parsed header in raw:\n{text}");
}
