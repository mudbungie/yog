//! The agent-delete tables: the gate over the conversation's members, the
//! amended §3.6 arming rule (typed name iff subtree), the `DeleteReport`
//! parse, and the two spawns — the unlogged dry-run census and the logged
//! removal.

use super::*;
use crate::opslog;
use crate::test_support::spawn_guard;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

const ROOT: &str = "r-aa";
const CHILD: &str = "r-aa-c-bb";

/// One agent row, the conversation-list fixture shape (lernie ARCH §2.3).
fn agent(id: &str, state: AgentState, ts: i64) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_string(),
        name: None,
        tip_oid: "a".repeat(40),
        tip_short_oid: "aaaaaaaa".into(),
        tip_timestamp_unix: ts,
        last_action_unix: ts,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state,
        state_uncertain: false,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        goal_name: None,
        call_start_unix: None,
    }
}

#[test]
fn the_gate_counts_every_running_or_uncertain_member_and_only_members() {
    let kid = agent(CHILD, AgentState::InFlight, 2);
    let ghost = agent("z-zz", AgentState::Live, 3);
    let mut root = agent(ROOT, AgentState::Stopped, 1);
    root.state_uncertain = true; // the §10 "?" counts as live — fail closed
    let confirm = confirmation(ROOT, &[root, kid, ghost]);
    assert_eq!(
        confirm.live,
        [ROOT, CHILD],
        "the other conversation is not ours"
    );
    assert!(confirm.refused());
    assert!(!confirm.subtree_armed(ROOT), "never armed while refused");
}

#[test]
fn a_settled_conversation_passes_and_the_typed_name_arms_the_subtree() {
    let mut root = agent(ROOT, AgentState::Stopped, 1);
    root.goal_name = Some("fix the parser".into());
    let confirm = confirmation(ROOT, &[root, agent(CHILD, AgentState::Quiescent, 2)]);
    assert!(!confirm.refused());
    assert_eq!(
        confirm.name, "fix the parser",
        "the §3.3 ladder names the dialog"
    );
    assert!(
        confirm.subtree_armed("  fix the parser "),
        "whitespace forgiven"
    );
    assert!(!confirm.subtree_armed("fix"), "nothing else is");
}

#[test]
fn an_absent_root_is_the_general_path_with_empty_inputs() {
    // lernie's delete of an absent agent is already its postcondition; yog's
    // gate mirrors that convergence rather than minting an error class.
    let confirm = confirmation("gone-id", &[]);
    assert_eq!(confirm.name, "gone-id", "rung three: its own id");
    assert!(!confirm.refused());
}

#[test]
fn the_refusal_names_the_live_members() {
    assert_eq!(
        live_refusal(&["r-aa".into(), "r-aa-c-bb".into()]),
        "refused \u{2014} live: r-aa, r-aa-c-bb \u{2014} stop them first"
    );
}

#[test]
fn the_report_parse_reads_both_moods_and_refuses_garbage() {
    let subtree = parse_report(
        "would delete r-aa; descendants: 2 (r-aa-c-bb, r-aa-d-ee); pending deposits: 3",
    )
    .unwrap();
    assert_eq!(subtree.descendants, [CHILD, "r-aa-d-ee"]);
    assert_eq!(subtree.pending_deposits, 3);
    let leaf = parse_report("deleted r-aa; descendants: 0; pending deposits: 0").unwrap();
    assert_eq!(
        leaf,
        Census {
            descendants: vec![],
            pending_deposits: 0
        }
    );
    for garbage in [
        "",
        "agent \"r-aa\" is being driven",
        "would delete r-aa; descendants: 1 (x; pending deposits: y",
    ] {
        assert!(parse_report(garbage).is_none(), "{garbage:?}");
    }
}

/// A fake `lernie` script: logs `$@` beside itself, prints `stdout`, exits
/// `code` — the `delete/exec` fixture idiom.
struct FakeLernie {
    dir: TempDir,
}

impl FakeLernie {
    fn new(stdout: &str, stderr: &str, code: i32) -> Self {
        let dir = tempdir().unwrap();
        let log = dir.path().join("argv.log");
        let path = dir.path().join("lernie");
        fs::write(
            &path,
            format!(
                "#!/bin/sh\necho \"$@\" > {}\nprintf '%s\\n' '{stdout}'\nprintf '%s' '{stderr}' 1>&2\nexit {code}\n",
                log.display()
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        Self { dir }
    }

    fn cli(&self) -> Cli {
        Cli::new(self.dir.path().join("lernie"))
    }

    fn argv(&self) -> String {
        fs::read_to_string(self.dir.path().join("argv.log"))
            .unwrap_or_default()
            .trim()
            .to_owned()
    }

    fn ws(&self) -> PathBuf {
        let ws = self.dir.path().join("ws");
        fs::create_dir_all(&ws).unwrap();
        ws
    }
}

#[test]
fn the_census_is_the_dry_run_subtree_form() {
    let _g = spawn_guard();
    let fx = FakeLernie::new(
        "would delete r-aa; descendants: 1 (r-aa-c-bb); pending deposits: 2",
        "",
        0,
    );
    let ws = fx.ws();
    let census = census(&fx.cli(), &ws, ROOT).unwrap();
    assert_eq!(census.descendants, [CHILD]);
    assert_eq!(census.pending_deposits, 2);
    assert_eq!(
        fx.argv(),
        format!("delete {} {ROOT} --children --dry-run", ws.display()),
        "the census asks the substrate, never re-derives"
    );
}

#[test]
fn a_declined_or_unreadable_census_fails_closed() {
    let _g = spawn_guard();
    let declined = FakeLernie::new("", "not a workspace", 2);
    let ws = declined.ws();
    assert_eq!(
        census(&declined.cli(), &ws, ROOT).unwrap_err(),
        "not a workspace"
    );
    let garbled = FakeLernie::new("all good", "", 0);
    let ws = garbled.ws();
    assert_eq!(
        census(&garbled.cli(), &ws, ROOT).unwrap_err(),
        "unrecognized delete report: all good"
    );
    let gone = Cli::new(declined.dir.path().join("no-such-lernie"));
    assert!(
        census(&gone, &ws, ROOT)
            .unwrap_err()
            .contains("No such file")
    );
}

#[test]
fn the_removal_is_the_logged_lernie_verb_bare_or_subtree() {
    let _g = spawn_guard();
    let state = tempdir().unwrap();
    let fx = FakeLernie::new("deleted r-aa; descendants: 0; pending deposits: 0", "", 0);
    let ws = fx.ws();
    let outcome = spawn(&fx.cli(), state.path(), "TS", &ws, ROOT, false).unwrap();
    assert!(outcome.ok());
    assert_eq!(
        fx.argv(),
        format!("delete {} {ROOT}", ws.display()),
        "bare: no subtree implied"
    );

    let armed = spawn(&fx.cli(), state.path(), "TS", &ws, ROOT, true).unwrap();
    assert!(armed.ok());
    assert_eq!(
        fx.argv(),
        format!("delete {} {ROOT} --children", ws.display())
    );

    let ops = opslog::tail(state.path(), 8);
    assert_eq!(ops.len(), 2, "each removal leaves its §4.2 row");
    assert_eq!(
        &ops[0].argv[1..],
        &["delete", &ws.display().to_string(), ROOT]
    );
    assert_eq!(ops[1].argv.last().map(String::as_str), Some("--children"));
}
