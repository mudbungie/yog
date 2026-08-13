//! Tool-call view-model derived from on-disk records.
//!
//! Per ARCH §3.3, each tool call writes
//! `<conv-repo>/steps/<conv-id>/<NNN>/tools/<tool-id>/input.json` first
//! and `output.json` after completion. A tool call is in-flight when
//! `input.json` exists but `output.json` does not — the same derivation
//! §3.3 names ("Tool in progress is derived state too — step N's
//! `response.json` carries a `tool_use` block with no matching
//! `output.json` yet"). No separate state file (§3.5: stateless re-read
//! on each tick).
//!
//! Scope is the latest step's `tools/` directory under the conversation,
//! mirroring the streaming-text pattern (§3.5 view-model is per-tick).

use std::path::Path;

use super::streaming::latest_step_dir;
use super::{STEPS_DIR, ToolCall, ToolCallState};

const TOOLS_SUBDIR: &str = "tools";
const INPUT_FILE: &str = "input.json";
const OUTPUT_FILE: &str = "output.json";

/// Read the latest step's tool-call records and return one [`ToolCall`]
/// per `<tool-id>/` subdir whose `input.json` is present. Sorted by
/// tool-id for deterministic ordering across ticks (the wire `tool_use.id`
/// monotonically encodes call order, but disk `read_dir` is unordered).
pub(super) fn tool_calls_from_disk(workspace: &Path, agent_id: &str) -> Vec<ToolCall> {
    let agent_steps = workspace.join(STEPS_DIR).join(agent_id);
    let Some(latest) = latest_step_dir(&agent_steps) else {
        return Vec::new();
    };
    let tools_dir = latest.join(TOOLS_SUBDIR);
    let Ok(entries) = std::fs::read_dir(&tools_dir) else {
        return Vec::new();
    };
    let mut calls: Vec<ToolCall> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let tool_id = entry.file_name().to_str()?.to_string();
            if !path.join(INPUT_FILE).exists() {
                return None;
            }
            let state = if path.join(OUTPUT_FILE).exists() {
                ToolCallState::Complete
            } else {
                ToolCallState::InFlight
            };
            let input = path.join(INPUT_FILE);
            let name = tool_name(&input);
            // The record's stamp rides out beside its name and its state, off
            // the one read of the one file: `input.json` lands atomically
            // immediately before the tool is spawned and is never rewritten, so
            // its mtime is when this call *started* (§5.1 #28 elapsed, bl-9dfb).
            // Nothing under `steps/` is git-tracked (§2.3), so no commit
            // timestamp exists to prefer over it.
            let start_unix = super::enumerate::mtime_unix(&input);
            Some(ToolCall {
                tool_id,
                name,
                start_unix,
                state,
            })
        })
        .collect();
    calls.sort_by(|a, b| a.tool_id.cmp(&b.tool_id));
    calls
}

/// The wire `tool_use.name` off a landed `input.json` (`{"name":"Read",…}`).
/// `None` for an unreadable, unparsable or name-less record — the same
/// partial-write tolerance the streaming fold has, since `input.json` is a file
/// yog may catch mid-write.
fn tool_name(input: &Path) -> Option<String> {
    let bytes = std::fs::read(input).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    Some(value.get("name")?.as_str()?.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &std::path::PathBuf, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    fn tool_dir(root: &Path, conv: &str, seq: u32, tool_id: &str) -> std::path::PathBuf {
        root.join(STEPS_DIR)
            .join(conv)
            .join(format!("{seq:03}"))
            .join(TOOLS_SUBDIR)
            .join(tool_id)
    }

    #[test]
    fn input_only_yields_in_flight() {
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-aaaa";
        let t = tool_dir(dir.path(), conv, 1, "toolu_01a");
        write(&t.join(INPUT_FILE), br#"{"name":"Read","input":{}}"#);
        let calls = tool_calls_from_disk(dir.path(), conv);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_id, "toolu_01a");
        assert_eq!(calls[0].state, ToolCallState::InFlight);
        // The name rides with the state: one read of the record the §11 strip
        // says the running tool's name from.
        assert_eq!(calls[0].name.as_deref(), Some("Read"));
    }

    #[test]
    fn a_nameless_or_malformed_record_yields_no_name() {
        // Both degrade the same way — the strip drops the segment rather than
        // printing the opaque tool id.
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-nnnn";
        write(
            &tool_dir(dir.path(), conv, 1, "toolu_a").join(INPUT_FILE),
            b"{}",
        );
        write(
            &tool_dir(dir.path(), conv, 1, "toolu_b").join(INPUT_FILE),
            b"{partial",
        );
        write(
            &tool_dir(dir.path(), conv, 1, "toolu_c").join(INPUT_FILE),
            br#"{"name":7}"#,
        );
        let calls = tool_calls_from_disk(dir.path(), conv);
        assert_eq!(calls.len(), 3);
        assert!(calls.iter().all(|c| c.name.is_none()));
    }

    #[test]
    fn input_and_output_yield_complete() {
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-bbbb";
        let t = tool_dir(dir.path(), conv, 1, "toolu_01b");
        write(&t.join(INPUT_FILE), b"{}");
        write(&t.join(OUTPUT_FILE), b"{}");
        let calls = tool_calls_from_disk(dir.path(), conv);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].state, ToolCallState::Complete);
    }

    #[test]
    fn returns_empty_when_steps_dir_absent() {
        let dir = tempdir().unwrap();
        assert!(tool_calls_from_disk(dir.path(), "no-such-conv").is_empty());
    }

    #[test]
    fn returns_empty_when_tools_dir_absent() {
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-cccc";
        std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(conv).join("001")).unwrap();
        assert!(tool_calls_from_disk(dir.path(), conv).is_empty());
    }

    #[test]
    fn skips_entry_without_input_json() {
        // Mid-write race: the executor created `<tool-id>/` but hasn't
        // yet landed input.json. Until input.json appears the call isn't
        // ready to surface — drop it from the view-model.
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-dddd";
        let t = tool_dir(dir.path(), conv, 1, "toolu_01d");
        std::fs::create_dir_all(&t).unwrap();
        assert!(tool_calls_from_disk(dir.path(), conv).is_empty());
    }

    #[test]
    fn skips_non_directory_entries() {
        // A stray file at the tools/ level (editor backup, etc.) must
        // not produce a ToolCall — only `<tool-id>/` directories do.
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-eeee";
        let tools = dir
            .path()
            .join(STEPS_DIR)
            .join(conv)
            .join("001")
            .join(TOOLS_SUBDIR);
        std::fs::create_dir_all(&tools).unwrap();
        write(&tools.join(".keep"), b"");
        let real = tools.join("toolu_01e");
        std::fs::create_dir_all(&real).unwrap();
        write(&real.join(INPUT_FILE), b"{}");
        let calls = tool_calls_from_disk(dir.path(), conv);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_id, "toolu_01e");
    }

    #[test]
    fn reads_only_latest_step_tools() {
        // Earlier step's tools have already completed and shipped their
        // tool_result blocks into the next step's wire request; rendering
        // them again would double-surface. Latest step is the live one.
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-ffff";
        let old = tool_dir(dir.path(), conv, 1, "toolu_old");
        write(&old.join(INPUT_FILE), b"{}");
        write(&old.join(OUTPUT_FILE), b"{}");
        let new_inflight = tool_dir(dir.path(), conv, 2, "toolu_new");
        write(&new_inflight.join(INPUT_FILE), b"{}");
        let calls = tool_calls_from_disk(dir.path(), conv);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_id, "toolu_new");
        assert_eq!(calls[0].state, ToolCallState::InFlight);
    }

    #[test]
    fn sorts_calls_by_tool_id() {
        // `read_dir` order is filesystem-defined; deterministic render
        // order requires an explicit sort. tool_use.id is monotone in
        // wire order, so a string sort keeps oldest-call-first.
        let dir = tempdir().unwrap();
        let conv = "20260427T130000Z-gggg";
        for id in ["toolu_03", "toolu_01", "toolu_02"] {
            let t = tool_dir(dir.path(), conv, 1, id);
            write(&t.join(INPUT_FILE), b"{}");
        }
        let calls = tool_calls_from_disk(dir.path(), conv);
        let ids: Vec<&str> = calls.iter().map(|c| c.tool_id.as_str()).collect();
        assert_eq!(ids, vec!["toolu_01", "toolu_02", "toolu_03"]);
    }
}
