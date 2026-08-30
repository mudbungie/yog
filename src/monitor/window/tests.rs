//! The evidence gatherer's tables. The fold is pure over injected entries; the
//! delta is a real `git diff` against a real workspace, so it gets one.

use super::*;
use crate::transcript::{Entry, EntryKind, Usage};

fn model(name: &str, blocks: Vec<Block>) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: Vec::new(),
        kind: EntryKind::Model {
            model_id: "m".to_owned(),
            blocks,
            usage: Usage::new(),
        },
    }
}

#[test]
fn every_entry_kind_folds_to_its_quotation() {
    let transcript = Transcript {
        entries: vec![
            Entry {
                name: "001-you.md".to_owned(),
                raw: Vec::new(),
                kind: EntryKind::Delivered {
                    sender: "you".to_owned(),
                    epitaph: None,
                    body: "ship it".to_owned(),
                },
            },
            model(
                "002-m.json",
                vec![
                    Block::Text("on it".to_owned()),
                    Block::Thinking("hmm".to_owned()),
                    Block::ToolUse {
                        id: "t1".to_owned(),
                        name: "bash".to_owned(),
                        input_summary: "ls".to_owned(),
                    },
                ],
            ),
            Entry {
                name: "003-tool.json".to_owned(),
                raw: Vec::new(),
                kind: EntryKind::ToolResult {
                    tool_use_id: "t1".to_owned(),
                    content: "boom".to_owned(),
                    is_error: true,
                },
            },
            Entry {
                name: "«live»".to_owned(),
                raw: Vec::new(),
                kind: EntryKind::Streaming {
                    thinking: String::new(),
                    text: "partial".to_owned(),
                },
            },
            Entry {
                name: "«004»".to_owned(),
                raw: Vec::new(),
                kind: EntryKind::Compacted {
                    first: 4,
                    last: 4,
                    summary: "squashed".to_owned(),
                },
            },
            Entry {
                name: "junk".to_owned(),
                raw: Vec::new(),
                kind: EntryKind::Raw,
            },
        ],
    };
    let all = fold(&transcript, None);
    assert!(all.contains("[message from you]\nship it"));
    assert!(all.contains("on it") && all.contains("(thinking) hmm"));
    assert!(all.contains("(tool bash) ls"));
    assert!(all.contains("[tool result (error)]\nboom"));
    assert!(!all.contains("partial"), "no streaming text in a v1 window");
    assert!(
        all.contains("[record compacted here: entries 004\u{2013}004 deleted; litany's summary")
            && all.contains("squashed"),
        "the summary is what litany handed the agent in place of the span, quoted as \
         data under its own heading (VISION §4.9, bl-fde5):\n{all}"
    );
}

/// A gap whose summary rode an earlier mark (or was never written) still says
/// the entries are gone — the marker never depends on the summary existing.
#[test]
fn a_summaryless_mark_still_states_the_deletion() {
    let transcript = Transcript {
        entries: vec![Entry {
            name: "«005–007»".to_owned(),
            raw: Vec::new(),
            kind: EntryKind::Compacted {
                first: 5,
                last: 7,
                summary: String::new(),
            },
        }],
    };
    assert_eq!(
        fold(&transcript, None),
        "[record compacted here: entries 005\u{2013}007 deleted; no summary on this mark]\n"
    );
}

#[test]
fn a_delta_keeps_only_the_named_entries_plus_every_compaction_mark() {
    let transcript = Transcript {
        entries: vec![
            Entry {
                name: "«001»".to_owned(),
                raw: Vec::new(),
                kind: EntryKind::Compacted {
                    first: 1,
                    last: 1,
                    summary: "what was cut".to_owned(),
                },
            },
            model("002-m.json", vec![Block::Text("old".to_owned())]),
            model("003-m.json", vec![Block::Text("new".to_owned())]),
        ],
    };
    let only = vec!["003-m.json".to_owned()];
    let window = fold(&transcript, Some(&only));
    assert!(window.contains("new") && !window.contains("old"));
    assert!(
        window.contains("what was cut"),
        "the marker is standing context and no diff can name it — a delta that \
         dropped it would omit the compaction from the one check where it is news:\n{window}"
    );
}

#[test]
fn a_result_without_an_error_says_so_plainly() {
    let transcript = Transcript {
        entries: vec![Entry {
            name: "001-tool.json".to_owned(),
            raw: Vec::new(),
            kind: EntryKind::ToolResult {
                tool_use_id: "t1".to_owned(),
                content: "ok".to_owned(),
                is_error: false,
            },
        }],
    };
    assert!(fold(&transcript, None).contains("[tool result]\nok"));
}

#[test]
fn an_oversized_window_keeps_its_tail_and_says_it_elided() {
    let long = "x".repeat(WINDOW_MAX + 100);
    let folded = tail(&long);
    assert!(folded.starts_with("[earlier work elided]"));
    assert_eq!(
        folded.chars().count(),
        WINDOW_MAX + "[earlier work elided]\n".len()
    );
    assert_eq!(tail("short"), "short", "a window under the budget is whole");
}

#[test]
fn a_missing_agent_yields_empty_evidence_and_no_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(
        gather(dir.path(), "nobody", None, "deadbeef"),
        Evidence::default()
    );
}

/// A `since` git cannot answer for (no repo at all) falls back to the whole
/// transcript rather than to an empty window: a check must never silently see
/// less than it should.
#[test]
fn an_unanswerable_diff_falls_back_to_the_whole_transcript() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(delta(dir.path(), Some("aaa"), "bbb"), None);
    assert_eq!(delta(dir.path(), None, "bbb"), None);
}

/// The real thing: a workspace, an agent, a goal, and two committed messages
/// one commit apart. The first check reads everything; the second reads only
/// what the branch gained — derived from the sha, never remembered.
#[test]
fn the_delta_is_derived_from_the_last_checked_sha() {
    let fx = crate::git_tree::tests::fixture::Fixture::new();
    let id = "20260803T120000Z-a001";
    fx.build_agent(id, "close bl-1");
    let wt = fx.path.join("agents").join(id);
    let messages = wt.join("messages");
    std::fs::create_dir_all(&messages).expect("messages");
    let write = |n: &str, text: &str| {
        std::fs::write(
            messages.join(n),
            serde_json::json!([{ "type": "text", "text": text }]).to_string(),
        )
        .expect("message");
    };
    write("001-m.json", "the first turn");
    commit(&wt, "first");
    let first = head(&wt);
    write("002-m.json", "the second turn");
    commit(&wt, "second");
    let second = head(&wt);

    let all = gather(&fx.path, id, None, &second);
    assert_eq!(all.goal.trim(), "close bl-1", "the goal, verbatim");
    assert!(all.window.contains("the first turn") && all.window.contains("the second turn"));

    let since = gather(&fx.path, id, Some(&first), &second);
    assert!(
        !since.window.contains("the first turn") && since.window.contains("the second turn"),
        "only what the branch gained: {:?}",
        since.window
    );
}

fn commit(wt: &std::path::Path, message: &str) {
    for args in [vec!["add", "-A"], vec!["commit", "-q", "-m", message]] {
        let out = crate::git_env::output(crate::git_env::git().arg("-C").arg(wt).args(&args))
            .expect("git");
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }
}

fn head(wt: &std::path::Path) -> String {
    let out = crate::git_env::output(
        crate::git_env::git()
            .arg("-C")
            .arg(wt)
            .args(["rev-parse", "HEAD"]),
    )
    .expect("git");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}
