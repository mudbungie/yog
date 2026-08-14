//! **Authoring the control into a workspace** (DESIGN §8.6, VISION §4.11 item 2):
//! the one write this module makes, and the reason every drone is born
//! adjudicated.
//!
//! An agent's policy is its `workflow.yaml`, frozen at the config commit its
//! branch forks off. So the control is authored **onto `config/default`, at
//! every start**: a workspace created a moment ago and a workspace created last
//! week both converge to a tip that names the shim, and every agent forked after
//! that commit is controlled. Agents already running keep the policy they froze
//! — that is lernie's law, not a gap this could close.
//!
//! **The ruling named a different file, and the tree says it cannot work.** The
//! ruling authored the block into `<LERNIE_HOME>/template/workflow.yaml`, on
//! the premise that `lernie prime` seeds that file and later seeding is
//! seed-if-absent. Verified against the pin, all three halves are false:
//!
//! - `prime` never touches `template/` — it is an *override* root, absent by
//!   default ("policy lives in config, not code", at lernie's own constant).
//! - The override is a whole-file `fs::copy`, not a merge. A `workflow.yaml`
//!   carrying only `tool_control:` would delete `events:` — and with it every
//!   dispatch — from every workspace born after it.
//! - Authoring a *complete* override would need lernie's embedded default, and
//!   the crate's `template` module is private. There is no lawful read of it.
//!
//! So the base is taken from where it is undeniably correct: the workspace's own
//! committed `workflow.yaml`, which is exactly what lernie put there. And the
//! write goes through the one lawful writer of `config/*` — the scripted-editor
//! `lernie config` drive (§9.3) — so yog still never writes inside a workspace.
//! This is *stronger* than the template route rather than a retreat from it: the
//! template only reached workspaces born after it, while this reaches every
//! workspace on its next start.
//!
//! Idempotence is by comparison, not by memory: [`authored`] is a fixed point,
//! so a tip that already carries the block computes to itself and nothing is
//! staged, nothing is spawned, and no commit is authored.

use std::path::Path;

use crate::config_edit::branch::config_file;
use crate::config_edit::branch::edit::DraftFile;

/// The config lineage every workspace is born on and every fresh agent forks
/// off (lernie ARCH §2.2).
pub const DEFAULT_CONFIG: &str = "default";
/// Its refspec in the bare workspace repo.
const DEFAULT_REF: &str = "refs/heads/config/default";
/// The control file inside a config commit.
const WORKFLOW_YAML: &str = "workflow.yaml";
/// The block's key, as lernie's workflow parser reads it.
const KEY: &str = "tool_control:";
/// The prefix of the one comment line yog authors. Stripped by the same pass
/// that strips the block, so authoring stays a fixed point: a note that
/// survived its own block would accrete one copy per start.
const MARK: &str = "# yog authors this block";

/// One committed control file, as `config/default` carries it — the base every
/// authoring starts from. `None` when the workspace has no config commit yet or
/// the file cannot be read: nothing to author onto, which is not an error, only
/// nothing to do. Shared with §3.7's `manifest.yaml` author: two files, one
/// read of the same lineage tip.
pub fn committed(workspace: &Path, file: &str) -> Option<String> {
    let bytes = config_file(workspace, DEFAULT_REF, file).ok()?;
    String::from_utf8(bytes).ok()
}

/// `base` with any existing top-level `tool_control:` block replaced by one
/// naming `shim`. A **fixed point**: authoring an authored file reproduces it
/// byte for byte, which is the whole convergence test.
pub fn authored(base: &str, shim: &Path) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in base.lines() {
        // A top-level line ends the block; an indented one continues it.
        if inside && !line.starts_with([' ', '\t']) {
            inside = false;
        }
        if line.starts_with(KEY) {
            inside = true;
        }
        if !inside && !line.starts_with(MARK) {
            out.push_str(line);
            out.push('\n');
        }
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    out.push_str(&block(shim));
    out
}

/// The block yog authors, with the note that says whose artifact it is.
fn block(shim: &Path) -> String {
    format!(
        "\n{MARK}: the capability control consulted before every granted tool \
         invocation executes, rewritten whenever it drifts from the installed \
         yog. Remove it and nothing is adjudicated.\n{KEY}\n  command: {}\n",
        shim.display(),
    )
}

/// `workspace`'s `workflow.yaml` drift against a tip naming `shim`, or `None`
/// when the tip already carries the block — the steady state, which reads one
/// file from git and stages nothing.
///
/// The drafted file is the **whole** `workflow.yaml`: the scripted editor
/// copies files over the checkout, so a fragment would truncate the policy.
/// Who *drives* the commit is [`crate::start::execute_ensure_workspace`], which
/// converges this drift and §3.7's `manifest.yaml` drift in one `lernie config`
/// pass — two files of one policy, one checkout, one commit, one ops row.
pub fn workflow_drift(workspace: &Path, shim: &Path) -> Option<DraftFile> {
    let base = committed(workspace, WORKFLOW_YAML)?;
    let want = authored(&base, shim);
    (want != base).then(|| DraftFile {
        rel_path: WORKFLOW_YAML.to_owned(),
        bytes: want.into_bytes(),
    })
}

#[cfg(test)]
mod tests;
