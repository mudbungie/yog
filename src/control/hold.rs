//! The **hold mark** as yog reads it — `refs/lernie/held/<agent-id>` (lernie
//! ARCH §3.3, DESIGN §8.6).
//!
//! When the control answers `hold`, lernie's seam parks the invocation *before*
//! it executes and points this ref at a blob holding one line of JSON: the held
//! `tool_use` id, the tool the model named, and the control's own reason. That
//! is the parked branch's one non-derivable fact, and it is lernie's to write —
//! yog only ever reads it.
//!
//! Two readers, one parse:
//!
//! - the snapshot tick, which enumerates the whole namespace once per workspace
//!   ([`crate::git_tree`]) so the §6 attention predicate can see every park;
//! - the answer gesture, which reads **one** mark at fire time
//!   ([`read`]) — fail-closed, exactly as the §3.6 delete gate re-derives rather
//!   than trusting a dialog. A once-answer is scoped to the id that is parked
//!   *now*, which is what makes it unable to race.
//!
//! An unreadable or unparseable mark reads as absent, the discipline lernie's
//! own reader keeps: never a forged park, and never a panic on a mangled blob.

use std::path::Path;

use serde_json::Value;

/// Ref namespace for the mark (lernie ARCH §3.3). The one spelling; the
/// snapshot's enumeration and the single read below both name it from here.
pub const HELD_PREFIX: &str = "refs/lernie/held/";

/// What one parked invocation is. The operator's whole question — *what was it
/// about to do, and why did the control stop it* — answered without opening a
/// transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// The parked `tool_use.id`. Provider-unique, so the once-answer scoped to
    /// it needs no consumption and cannot be spent by a different invocation.
    pub tool_use_id: String,
    /// The tool the model named — for the operator's eyes.
    pub tool: String,
    /// The control's stated reason: the tool, an input summary, the computed
    /// class and the evidence, in one sentence ([`super::reason`]).
    pub reason: String,
}

/// Parse the mark's blob. `None` for anything that is not the three-field
/// object lernie writes.
pub fn parse(blob: &str) -> Option<Held> {
    let value: Value = serde_json::from_str(blob).ok()?;
    let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
    Some(Held {
        tool_use_id: text("tool_use_id")?,
        tool: text("tool")?,
        reason: text("reason").unwrap_or_default(),
    })
}

/// The mark one agent wears right now, read live off the workspace repo.
/// `None` when nothing is parked — the ordinary state of every branch — and
/// equally when git will not run or the blob is not the shape lernie writes.
pub fn read(workspace: &Path, agent_id: &str) -> Option<Held> {
    let repo = workspace.join("repo.git");
    let out = crate::git_env::output(crate::git_env::git().arg("--git-dir").arg(&repo).args([
        "cat-file",
        "blob",
        &format!("{HELD_PREFIX}{agent_id}"),
    ]))
    .ok()
    .filter(|out| out.status.success())?;
    parse(&String::from_utf8_lossy(&out.stdout))
}

#[cfg(test)]
mod tests;
