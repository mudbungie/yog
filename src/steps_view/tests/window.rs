//! The orphaned-tail state's **tool-window** shape (bl-abba): an executor
//! that died mid-tool-window leaves an assistant entry whose `tool_use`
//! nobody answered, its lock free and no hold mark — the third member of the
//! swallowed-error class, and until this the only one that painted nothing at
//! all.
//!
//! The arms that must NOT fire are the file's real work, because this shape's
//! near-misses are the ordinary states of the world: a turn that ended on
//! text is every resting conversation, an answered call is every healthy
//! turn, and a parked call is the capability control doing its job (§8.6).

use tempfile::tempdir;

use super::AGENT;
use super::orphan::{DECLINE, messages, write_driver_log, write_messages};
use crate::git_tree::AgentState;
use crate::steps_view::{ORPHANED_WINDOW, Orphan, Tail, build};

/// A committed assistant entry that calls a tool, in the bare-array envelope
/// litany commits (`transcript` module doc).
fn write_tool_window(ws: &std::path::Path, name: &str) {
    std::fs::write(
        messages(ws).join(name),
        br#"[{"type":"tool_use","id":"toolu_1","name":"bash","input":{"command":"ls"}}]"#,
    )
    .unwrap();
}

/// A bare workspace repo whose `refs/litany/held/<agent>` names the park the
/// capability control imposed — the shape `control::hold`'s own tests lay down,
/// which is what litany's seam writes (ARCH §3.3).
fn park(ws: &std::path::Path, agent: &str) {
    let repo = ws.join("repo.git");
    let git = |args: &[&str]| {
        crate::git_env::output(crate::git_env::git().arg("--git-dir").arg(&repo).args(args))
            .expect("git runs")
    };
    std::fs::create_dir_all(&repo).unwrap();
    git(&["init", "--bare", "-q"]);
    let staged = ws.join("mark.json");
    std::fs::write(
        &staged,
        br#"{"tool_use_id":"toolu_1","tool":"bash","reason":"open-world"}"#,
    )
    .unwrap();
    let hashed = git(&["hash-object", "-w", "--", &staged.to_string_lossy()]);
    let oid = String::from_utf8_lossy(&hashed.stdout).trim().to_owned();
    git(&[
        "update-ref",
        &format!("{}{agent}", crate::control::hold::HELD_PREFIX),
        &oid,
    ]);
}

/// **THE BALL**: the window an executor died inside is a rendered fact, and
/// the banner names the one gesture that recovers it — nobody deposits into
/// an agent that looks finished.
#[test]
fn an_unanswered_tool_window_with_no_driver_banners_and_names_the_remedy() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_messages(ws, &["001-user.md"]);
    write_tool_window(ws, "002-opus.json");
    write_driver_log(ws, DECLINE);

    let view = build(ws, AGENT, AgentState::Stopped);
    let Orphan::Spoke(Tail::ToolWindow, words) = &view.orphan else {
        panic!("expected Spoke(ToolWindow), got {:?}", view.orphan);
    };
    assert!(words.contains("unmatched"), "{words}");
    let banner = view.orphan.banner();
    assert!(banner.contains(ORPHANED_WINDOW), "{banner}");
    assert!(banner.contains("a message revives it"), "{banner}");
    assert!(banner.contains("unmatched"), "{banner}");
}

#[test]
fn a_mute_tool_window_still_banners_its_own_class() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_tool_window(ws, "001-opus.json");

    let view = build(ws, AGENT, AgentState::Stopped);
    assert_eq!(view.orphan, Orphan::Mute(Tail::ToolWindow));
    let banner = view.orphan.banner();
    assert!(banner.contains(ORPHANED_WINDOW), "{banner}");
    assert!(banner.contains("nothing on disk says why"), "{banner}");
}

/// A model turn that ended on text is a conversation at rest — the arm that
/// keeps this shape from alarming on every quiescent agent in the world.
#[test]
fn a_model_turn_that_called_nothing_is_no_orphan() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    std::fs::write(
        messages(ws).join("001-opus.json"),
        br#"[{"type":"text","text":"done"}]"#,
    )
    .unwrap();

    assert_eq!(build(ws, AGENT, AgentState::Stopped).orphan, Orphan::None);
}

/// A committed `tool_result` on the tail is a window that closed: the call
/// was answered, and what the branch now owes is a model call, not a tool.
#[test]
fn an_answered_tool_call_is_no_orphan() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_tool_window(ws, "001-opus.json");
    std::fs::write(
        messages(ws).join("002-tool.json"),
        br#"[{"type":"tool_result","tool_use_id":"toolu_1","content":"ok"}]"#,
    )
    .unwrap();

    assert_eq!(build(ws, AGENT, AgentState::Stopped).orphan, Orphan::None);
}

/// A **parked** call wears this exact shape and is waiting on purpose (§8.6):
/// the control answered `hold`, litany wrote the mark, and the operator is
/// the driver. Alarming there would turn every park into a crash.
#[test]
fn a_parked_tool_call_is_no_orphan() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_tool_window(ws, "001-opus.json");
    write_driver_log(ws, DECLINE);
    park(ws, AGENT);

    assert_eq!(build(ws, AGENT, AgentState::Stopped).orphan, Orphan::None);
}

/// A driver at work on the window is filling it, not abandoning it.
#[test]
fn a_driven_tool_window_is_never_an_orphan() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_tool_window(ws, "001-opus.json");
    for state in [AgentState::Live, AgentState::InFlight] {
        assert_eq!(build(ws, AGENT, state).orphan, Orphan::None);
    }
}
