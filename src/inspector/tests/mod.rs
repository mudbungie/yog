//! Acceptance proof for the inspector composition (§11), split at §12's cap
//! on the seam between the fixture and what each half asserts:
//!
//! - this file — the populated fixture workspace and the two [`TabData`]
//!   builders every half shares;
//! - [`tabs`] — cycling all five [`InspectorTab`] values, each tab's signature
//!   content reaching the paint layer;
//! - [`raw`] — S7-T1's Raw-toggle half;
//! - [`config`] — the governing-config edge arms (a branch tip / frozen past
//!   it / absent);
//! - [`pinned`] — S10's inspector half: the rail gutter and the pin banner
//!   that reach every tab.

use super::*;
use crate::config_edit::branch::GoverningConfig;
use crate::files_view::FilesView;
use crate::git_tree::GitTree;
use crate::git_tree::tests::fixture::Fixture;
use crate::nav::convs::Titles;

/// The conversation's §3.3 display name — the transcript's sender label for
/// every model turn (bl-2335).
pub(super) const SPEAKER: &str = "shudder-storeroom";

/// Render one tab of a [`TabData`] and return every painted galley's text.
pub(super) fn paint(tab: InspectorTab, data: &TabData) -> String {
    let mut eph = Ephemera::default();
    crate::paint_probe::paint(|ui| {
        render(ui, tab, data, &Titles::default(), &mut eph);
    })
}

/// A workspace populated across every inspector surface: a committed agent
/// branch, a `messages/` transcript, a step with a `response.json` + tool i/o,
/// and an inbox deposit.
pub(super) fn populated() -> (Fixture, String, String) {
    let fx = Fixture::new();
    fx.build_agent("c-1", "hello");
    // Transcript: a delivered message, a model reply, and a tool result.
    let messages = fx.path.join("agents/c-1/messages");
    std::fs::create_dir_all(&messages).unwrap();
    std::fs::write(messages.join("001-user.md"), "please ping").unwrap();
    std::fs::write(
        messages.join("002-opus.json"),
        br#"{"content":[{"type":"text","text":"pong reply"}]}"#,
    )
    .unwrap();
    std::fs::write(
        messages.join("003-tool.json"),
        br#"{"tool_use_id":"toolu_1","content":"tool said hi","is_error":false}"#,
    )
    .unwrap();
    // Step 001 response + meta + tool i/o (drill-in with tool input/output).
    // The meta commit is what the transcript's boundary rule shows (§5.1 #29).
    let step = fx.path.join("steps/c-1/001");
    std::fs::create_dir_all(step.join("tools/toolu_1")).unwrap();
    std::fs::write(
        step.join("meta.json"),
        br#"{"commit":"feedc0dedeadbeeffeedc0dedeadbeeffeedc0de","started_at":"t1","ended_at":"t2"}"#,
    )
    .unwrap();
    std::fs::write(
        step.join("response.json"),
        b"{\"type\":\"usage\",\"input_tokens\":10,\"output_tokens\":5}\n{\"type\":\"end\"}\n",
    )
    .unwrap();
    std::fs::write(step.join("tools/toolu_1/input.json"), br#"{"name":"Read"}"#).unwrap();
    std::fs::write(
        step.join("tools/toolu_1/output.json"),
        br#"{"exit_code":0}"#,
    )
    .unwrap();
    // Inbox deposit (the `✉n` explanation).
    fx.deposit_message(
        "c-1",
        "user-001.md",
        "---\nfrom: user\ndeposited_at: t0\n---\nfollow-up message",
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let agent = tree.agents.iter().find(|a| a.agent_id == "c-1").unwrap();
    (fx, agent.agent_id.clone(), agent.tip_oid.clone())
}

mod config;
mod pinned;
mod raw;
mod tabs;

/// A [`TabData`] carrying only the given view-models — every ephemeral field at
/// its empty value, the rest built fresh and moved in by the caller.
pub(super) fn empty_tab_data(
    transcript: Transcript,
    steps: StepsView,
    inbox: Vec<InboxEntry>,
    files: FilesView,
    governing: Option<GoverningConfig>,
) -> TabData {
    TabData {
        transcript: std::sync::Arc::new(transcript),
        speaker: SPEAKER.to_string(),
        raw: false,
        auto: AutoExpand::default(),
        steps,
        step_sel: None,
        step_detail: None,
        step_tab: StepTab::Meta,
        inbox,
        files,
        file_preview: None,
        science: Vec::new(),
        work_patch: None,
        governing,
        rail: Rail::default(),
        pin: None,
    }
}
