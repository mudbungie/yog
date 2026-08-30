//! The [`Fixture`] half that writes **plain files git never sees**: the step
//! records under `<workspace>/steps/<agent-id>/<NNN>/` and the inbox deposits
//! under `<workspace>/inbox/<agent-id>/` (ARCH §2.3, §2.11). Both live *outside*
//! every worktree and are never committed, so nothing here forks git — that is
//! [`super::fixture`]'s job, and the seam is the one ARCH draws.

use super::fixture::Fixture;
use std::fs;

impl Fixture {
    /// Write the diagnostic step record for `conv_id`'s first step at
    /// the workspace root (ARCH §2.3). Called by `build_agent`, also
    /// exposed for tests that need to seed a step record without
    /// building a full branch.
    pub(crate) fn write_step_record(&self, conv_id: &str, user_message: &str) {
        self.write_request(conv_id, 1, user_message);
    }

    /// Write `steps/<conv-id>/<seq>/request.json` — the file litany lands once,
    /// immediately before it invokes the model, so its mtime is that step's
    /// model-call start (§5.1 #28). Returns the path so a test can compare the
    /// snapshot's `call_start_unix` against the stamp it actually wrote.
    pub(super) fn write_request(
        &self,
        conv_id: &str,
        seq: u32,
        user_message: &str,
    ) -> std::path::PathBuf {
        let step_dir = self
            .path
            .join("steps")
            .join(conv_id)
            .join(format!("{seq:03}"));
        fs::create_dir_all(&step_dir).unwrap();
        let request_json = serde_json::json!({
            "model": "m",
            "messages": [{"role": "user", "content": user_message}],
        });
        let path = step_dir.join("request.json");
        fs::write(&path, serde_json::to_vec_pretty(&request_json).unwrap()).unwrap();
        path
    }

    /// Overwrite `agents/<conv-id>/goal.md` — the operator's payload in its one
    /// home (§3.3), which the enumerate pass reads as a plain file rather than
    /// out of the branch. `build_agent` seeds it from the same string it seeds
    /// the step record with; this parts the two, which is the only way to say
    /// which one a preview came from.
    pub(super) fn write_goal(&self, conv_id: &str, goal: &str) {
        let dir = self.path.join("agents").join(conv_id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("goal.md"), goal).unwrap();
    }

    /// Write `steps/<conv-id>/001/request.json` as the **assembled context**
    /// litany really sends (bl-368d): a block array whose first `text` block is
    /// the §3.7 pinned-instruction frame, the operator's message behind it
    /// inside its deposit envelope. Every other fixture writes `content` as a
    /// plain string, which is why nothing here ever saw the head the operator
    /// was shown.
    pub(super) fn write_assembled_request(&self, conv_id: &str) {
        let step_dir = self.path.join("steps").join(conv_id).join("001");
        fs::create_dir_all(&step_dir).unwrap();
        let request_json = serde_json::json!({
            "model": "m",
            "messages": [{"role": "user", "content": [
                {"type": "text",
                 "text": "<file path=\"instructions/00/AGENTS.md\">\nhouse rules\n</file>"},
                {"type": "text",
                 "text": "---\nfrom: operator\n---\n\nunbar the postern"},
            ]}],
        });
        fs::write(
            step_dir.join("request.json"),
            serde_json::to_vec_pretty(&request_json).unwrap(),
        )
        .unwrap();
    }

    /// Write a tool-call `input.json` (and optionally `output.json`)
    /// under `<workspace>/steps/<conv-id>/<seq>/tools/<tool_id>/`. Mirrors
    /// the executor's on-disk shape (ARCH §3.3): `input.json` lands first
    /// at dispatch, `output.json` only after the tool exits. Pass `None`
    /// for the in-flight case. Returns the `input.json` path — the record whose
    /// mtime is the call's start (§5.1 #28).
    pub(super) fn write_tool_call(
        &self,
        conv_id: &str,
        seq: u32,
        tool_id: &str,
        output: Option<&[u8]>,
    ) -> std::path::PathBuf {
        let tool_dir = self
            .path
            .join(format!("steps/{conv_id}/{seq:03}/tools/{tool_id}"));
        fs::create_dir_all(&tool_dir).unwrap();
        let input = tool_dir.join("input.json");
        fs::write(&input, br#"{"name":"Bash"}"#).unwrap();
        if let Some(out) = output {
            fs::write(tool_dir.join("output.json"), out).unwrap();
        }
        input
    }

    /// Deposit a pending message into `agent_id`'s inbox at
    /// `<workspace>/inbox/<agent-id>/<filename>` (ARCH §2.11). Mirrors a
    /// `<sender>-<NNN>.md` deposit the frontend counts for the
    /// pending-message indicator (§7.1).
    pub(crate) fn deposit_message(&self, agent_id: &str, filename: &str, body: &str) {
        let inbox = self.path.join("inbox").join(agent_id);
        fs::create_dir_all(&inbox).unwrap();
        fs::write(inbox.join(filename), body).unwrap();
    }

    /// Write a partial `response.json` for `conv_id`'s `seq`-th step.
    /// Each `event` is a JSONL line (no trailing newline); they are
    /// joined with `\n` and a trailing `\n` is appended, mirroring the
    /// shape the executor produces line by line. The fd closes when
    /// this helper returns — the harness's IN_CLOSE_WRITE semantics
    /// aren't reproduced here, but the on-disk snapshot the UI tails is
    /// identical, which is what the stateless-re-read view-model
    /// contract (ARCH §3.5) cares about.
    pub(super) fn write_response_events(&self, conv_id: &str, seq: u32, events: &[&str]) {
        let step_dir = self
            .path
            .join("steps")
            .join(conv_id)
            .join(format!("{seq:03}"));
        fs::create_dir_all(&step_dir).unwrap();
        let mut payload = events.join("\n");
        if !events.is_empty() {
            payload.push('\n');
        }
        fs::write(step_dir.join("response.json"), payload).unwrap();
    }
}
