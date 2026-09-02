//! **Authoring the control into a workspace** (DESIGN §8.6, VISION §4.11 item 2):
//! the one write this module makes, and the reason every drone is born
//! adjudicated.
//!
//! An agent's policy is its `workflow.yaml`, resolved from the config lineage
//! its branch forked off — at that lineage's **head, at every step boundary**
//! (litany's follow-the-tip ruling, upstream bl-403b; yog bl-e654). So the
//! control is authored **onto `config/default`, at every start**: a workspace
//! created a moment ago and a workspace created last week both converge to a
//! tip that names the shim, and every agent on that lineage is controlled —
//! including the ones already running, from their next step. That last clause
//! is new. It used to read *agents already running keep the policy they froze*,
//! and the convergence-at-every-start shape was the only reach this write had;
//! the shape survives the ruling unchanged and now reaches further than it was
//! designed to, which is the outcome to want from an inversion, not a reason to
//! revisit it.
//!
//! **The ruling named a different file, and the tree says it cannot work.** The
//! ruling authored the block into `<LITANY_HOME>/template/workflow.yaml`, on
//! the premise that `litany prime` seeds that file and later seeding is
//! seed-if-absent. Verified against the pin, all three halves are false:
//!
//! - `prime` never touches `template/` — it is an *override* root, absent by
//!   default ("policy lives in config, not code", at litany's own constant).
//! - The override is a whole-file `fs::copy`, not a merge. A `workflow.yaml`
//!   carrying only `tool_control:` would delete `events:` — and with it every
//!   dispatch — from every workspace born after it.
//! - Authoring a *complete* override would need litany's embedded default, and
//!   the crate's `template` module is private. There is no lawful read of it.
//!
//! So the base is taken from where it is undeniably correct: the workspace's own
//! committed `workflow.yaml`, which is exactly what litany put there. And the
//! write goes through the one lawful writer of `config/*` — the scripted-editor
//! `litany config` drive (§9.3) — so yog still never writes inside a workspace.
//! This is *stronger* than the template route rather than a retreat from it: the
//! template only reached workspaces born after it, while this reaches every
//! workspace on its next start.
//!
//! Idempotence is by comparison, not by memory: [`authored`] is a fixed point,
//! so a tip that already carries the block computes to itself and nothing is
//! staged, nothing is spawned, and no commit is authored.
//!
//! **The same fixed point holds one other thing, and holds it empty: there is
//! no conversation budget** (bl-56af). litany's `workflow.yaml` may carry a
//! `budgets:` block — `max_total_tokens`, `max_wall_seconds`, `max_depth`
//! (litany ARCH §6) — and every axis of it is a **whole-tree** consumable, one
//! allowance a root and its whole descent spend together. lernie's pre-`0.0.11`
//! template shipped it *set*, so every workspace born before that release froze
//! `max_wall_seconds: 3600` and `max_depth: 4` into its `config/default` and
//! caps every agent forked off that lineage. litany has since retired the seed,
//! but a template only ever reaches workspaces born after it — this convergence
//! is the only thing that reaches the ones already standing, which is the same
//! argument that put the control here.
//!
//! So [`authored`] **strips** a top-level `budgets:` block and leaves one line
//! saying so. Unconditionally, not down to a smaller number: a whole-tree
//! ceiling ends a conversation that is still working, which is the expensive
//! failure DESIGN §3.5 already reasons about — and yog's own ceiling, the
//! `ui.json` `ceiling` key, is that reasoning's answer (dollars, absent by
//! default, gating a *birth* and never a live drone, spoken on the V4 board
//! ahead of the spawn it will bind). Two ceilings over one concern is the
//! second representation that drifts; the one yog authors is the one it can
//! remove.

use std::path::Path;

use crate::config_edit::branch::config_file;
use crate::config_edit::branch::edit::DraftFile;

/// The config lineage every workspace is born on and every fresh agent forks
/// off (litany ARCH §2.2).
pub const DEFAULT_CONFIG: &str = "default";
/// The refspec of `config/<name>` in the bare workspace repo.
fn config_ref(config: &str) -> String {
    format!("refs/heads/config/{config}")
}
/// The control file inside a config commit.
const WORKFLOW_YAML: &str = "workflow.yaml";
/// The block's key, as litany's workflow parser reads it.
const KEY: &str = "tool_control:";
/// litany's whole-tree spend ceiling (litany ARCH §6). The second top-level
/// block this fixed point holds, and the only one it holds **empty** — yog
/// authors no ceiling and removes the one a pre-`0.0.11` template seeded.
const BUDGETS: &str = "budgets:";
/// The prefix of the one comment line yog authors. Stripped by the same pass
/// that strips the block, so authoring stays a fixed point: a note that
/// survived its own block would accrete one copy per start.
const MARK: &str = "# yog authors this block";
/// The prefix of the note left where the ceiling was — same discipline as
/// [`MARK`]: stripped by the pass that re-authors it.
const BUDGETS_MARK: &str = "# yog holds this file's budgets";

/// One committed control file, as `config/<config>` carries it — the base every
/// authoring starts from. `None` when the workspace has no such config commit
/// yet or the file cannot be read: nothing to author onto, which is not an
/// error, only nothing to do. Shared with §3.7's `manifest.yaml` author: two
/// files, one read of the same lineage tip.
///
/// **The lineage is a parameter because the drone's is** (§8.7, bl-380f). It is
/// [`DEFAULT_CONFIG`] for every untagged start, and the ball's tag-selected
/// lineage otherwise — the control must land on the branch the agent will
/// actually fork off, or a tagged birth is the one birth nothing adjudicates.
pub fn committed(workspace: &Path, config: &str, file: &str) -> Option<String> {
    let bytes = config_file(workspace, &config_ref(config), file).ok()?;
    String::from_utf8(bytes).ok()
}

/// `base` with any existing top-level `tool_control:` block replaced by one
/// naming `shim`, and any top-level `budgets:` block removed. A **fixed
/// point**: authoring an authored file reproduces it byte for byte, which is
/// the whole convergence test.
///
/// Two blocks, one pass, because they are one file's fixed point — a second
/// transform over `workflow.yaml` would be a second answer to "what does this
/// file say when yog is done with it".
pub fn authored(base: &str, shim: &Path) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in base.lines() {
        // A top-level line ends the block; an indented one continues it.
        if inside && !line.starts_with([' ', '\t']) {
            inside = false;
        }
        if line.starts_with(KEY) || line.starts_with(BUDGETS) {
            inside = true;
        }
        if !inside && !line.starts_with(MARK) && !line.starts_with(BUDGETS_MARK) {
            out.push_str(line);
            out.push('\n');
        }
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    out.push_str(&unbounded());
    out.push_str(&block(shim));
    out
}

/// The note left where the ceiling was, so the absence of one is **stated**
/// and not merely true — a cap that binds a conversation must never again be
/// a number only the file knows.
fn unbounded() -> String {
    format!(
        "\n{BUDGETS_MARK} unbounded and strips the block at every start: a \
         whole-tree token/wall/depth ceiling ends a conversation that is still \
         working, and the ceiling yog offers instead is `ceiling` in ui.json — \
         dollars, absent by default, and it refuses a birth rather than killing \
         a drone.\n"
    )
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
/// converges this drift and §3.7's `manifest.yaml` drift in one `litany config`
/// pass — two files of one policy, one checkout, one commit, one ops row.
pub fn workflow_drift(workspace: &Path, config: &str, shim: &Path) -> Option<DraftFile> {
    let base = committed(workspace, config, WORKFLOW_YAML)?;
    let want = authored(&base, shim);
    (want != base).then(|| DraftFile {
        rel_path: WORKFLOW_YAML.to_owned(),
        bytes: want.into_bytes(),
    })
}

#[cfg(test)]
mod tests;
