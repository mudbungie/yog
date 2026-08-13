//! STORIES **S7-T4** inbox-and-progress: deposits parse `from` /
//! `deposited_at` / `epitaph`, the `✉n` badge **equals** the listing's length,
//! Flush dispatches `lernie scan <ws>`, and a committed `tool_use` with no
//! `tool_result` renders "tool in progress" (STORIES S7.5, DESIGN §2.11, §5.1
//! #11, §8.2).
//!
//! The progress half is the interesting one: it is a fact read **off the
//! transcript**, not a guess about a running process. A tool whose result has
//! not been committed is in progress whether or not anything is executing, and
//! that is the honest reading — yog can see the committed bytes and cannot see
//! somebody else's process table.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, Recorder, build_agents, write_deposit, write_message};
use tempfile::tempdir;
use yog::actions::verbs;
use yog::cli_outbound::Cli;
use yog::git_tree::GitTree;
use yog::inboxview::{self, Epitaph};
use yog::opslog;
use yog::transcript;

/// A model turn that calls a tool and never hears back.
const CALLS_TOOL: &str = r#"{"content":[
    {"type":"text","text":"let me look"},
    {"type":"tool_use","id":"toolu_1","name":"Read","input":{"path":"/etc/hosts"}}
]}"#;
/// The result, committed later under the reserved `tool` origin.
const TOOL_RESULT: &str = r#"{"tool_use_id":"toolu_1","content":"127.0.0.1 localhost"}"#;

/// STORIES **S7-T4** inbox-and-progress.
#[test]
fn s7_t4_deposits_parse_the_count_is_the_listing_and_progress_is_read_not_guessed() {
    let root = tempdir().unwrap();
    let ws = root.path().join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    build_agents(&ws, &[AgentFixture::new("c-1", "work\n")]);

    // --- Three undelivered deposits, one of each shape that matters.
    write_deposit(
        &ws,
        "c-1",
        "user-001",
        "---\nfrom: user\ndeposited_at: 2026-08-07T12:00:00Z\n---\nhave a look\n",
    );
    write_deposit(
        &ws,
        "c-1",
        "peer-002",
        "---\nfrom: c-2\ndeposited_at: 2026-08-07T12:05:00Z\nepitaph: final-response\n---\nall done\n",
    );
    // A deposit with no envelope at all: the whole file is the body, every
    // field absent — a value, not an error.
    write_deposit(&ws, "c-1", "bare-003", "just some text\n");

    let inbox = inboxview::list_inbox(&ws, "c-1");
    assert_eq!(inbox.len(), 3, "sorted by path");

    let first = &inbox[2].deposit; // user-001 sorts last by name among the three
    let by_sender = |who: &str| {
        inbox
            .iter()
            .find(|e| e.deposit.sender.as_deref() == Some(who))
            .unwrap_or_else(|| panic!("a deposit from {who}"))
    };
    let user = by_sender("user");
    assert_eq!(
        user.deposit.deposited_at.as_deref(),
        Some("2026-08-07T12:00:00Z")
    );
    assert_eq!(
        user.deposit.epitaph, None,
        "an ordinary message has no epitaph"
    );
    assert_eq!(user.deposit.body, "have a look\n", "the body is verbatim");

    let peer = by_sender("c-2");
    assert_eq!(peer.deposit.epitaph, Some(Epitaph::FinalResponse));
    assert_eq!(
        peer.deposit.epitaph.as_ref().unwrap().label(),
        "final-response",
        "the on-disk spelling round-trips"
    );

    let bare = inbox.iter().find(|e| e.deposit.sender.is_none()).unwrap();
    assert_eq!(
        bare.deposit.body, "just some text\n",
        "no envelope, all body"
    );
    assert_eq!(bare.deposit.deposited_at, None);
    // The header renders what it has and marks what it does not — never a guess.
    assert_eq!(
        inboxview::header_line(&user.deposit),
        "✉ user · 2026-08-07T12:00:00Z"
    );
    assert_eq!(inboxview::header_line(&bare.deposit), "✉ ? · ?");
    let _ = first;

    // --- `✉n` IS the listing's length. Not a stored count, not a second read:
    // the snapshot's `pending` and the tab's listing are one derivation.
    let tree = GitTree::from_repo(&ws).unwrap();
    let agent = tree.agents.iter().find(|a| a.agent_id == "c-1").unwrap();
    assert_eq!(agent.pending.len(), inbox.len(), "one listing, two seats");
    assert_eq!(agent.pending.len(), 3);

    // --- Flush is `lernie scan <ws>` — one verb, logged like every other (§4.2).
    let bin = tempdir().unwrap();
    let state = tempdir().unwrap();
    let lernie = Recorder::new(bin.path(), "lernie").on("scan", "", 0);
    let bound = verbs::Bound::at(
        &Cli::new(lernie.path()),
        &yog::world::compose(&yog::xdg::Env::from_env()),
        &ws,
    );
    verbs::scan(&bound, state.path(), "T0").unwrap();
    let inv = lernie.invocations();
    assert_eq!(inv.len(), 1);
    assert_eq!(inv[0].argv, ["scan", &ws.to_string_lossy()]);
    let ops = opslog::tail(state.path(), 8);
    assert_eq!(&ops[0].argv[1..], &["scan", &ws.to_string_lossy()]);

    // --- "Tool in progress" is read off the transcript. The model committed a
    // `tool_use` and no `tool_result` for it has landed yet.
    write_message(&ws, "c-1", "001-opus.json", CALLS_TOOL);
    let tx = transcript::build(&ws, "c-1");
    assert!(
        tx.tool_in_progress("toolu_1"),
        "a call with no committed result is in progress"
    );

    // Commit the result and the fact changes — because the bytes changed, not
    // because anything was polled.
    write_message(&ws, "c-1", "002-tool.json", TOOL_RESULT);
    let tx = transcript::build(&ws, "c-1");
    assert!(
        !tx.tool_in_progress("toolu_1"),
        "the committed result settles it"
    );
    // A different call is still outstanding — the fact is per `tool_use_id`,
    // never per conversation.
    assert!(tx.tool_in_progress("toolu_2"));
}
