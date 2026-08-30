//! **Freezing is not composing** (DESIGN §3.7 item 4): the one glob that makes
//! a frozen instruction document actually reach the model.
//!
//! Pinning a document commits it beside `goal.md`; whether it *composes* into
//! assembled context is the governing `manifest.yaml`'s question (litany ARCH
//! §5.2). The shipped worker role pins `goal.md`, `soul.md`, `descriptions/**`
//! and orders `summary/**`, `skills/**` — so a pin at `instructions/…` under
//! the stock manifest is a committed file no model ever sees. That premise,
//! taken on faith, is what would have shipped this feature broken.
//!
//! So `roles.worker.pinned` gains [`GLOB`], authored onto `config/default` by
//! the same fixed-point convergence §8.6 runs for `tool_control:` — and in the
//! *same* `litany config` drive, since the two files are one policy
//! ([`crate::start::execute_ensure_workspace`]). [`authored`] is the fixed
//! point: authoring an authored manifest reproduces it byte for byte, which is
//! the whole convergence test.
//!
//! **`pinned:` rather than `order:`** deliberately — pinned is included
//! regardless of budget, so instructions can never be silently shed; what
//! bounds their cost is the walk's own size caps, not an overflow policy.
//!
//! **A manifest with no `roles.worker.pinned` anchor is left alone.** That is
//! an operator's own manifest, and yog does not fight it: the transform is a
//! no-op, the drift is `None`, and nothing is staged.

use crate::config_edit::branch::edit::DraftFile;
use std::ops::Range;
use std::path::Path;

#[cfg(test)]
mod tests;

/// The control file this authors, inside a config commit.
pub const MANIFEST_YAML: &str = "manifest.yaml";
/// The glob that admits every frozen instruction document, at any rank.
pub const GLOB: &str = "instructions/**";
/// The role yog's drones run as (litany's shipped `worker`).
const ROLE: &str = "worker:";
/// The map key holding the roles.
const ROLES: &str = "roles:";
/// The always-included category (litany ARCH §5.2).
const PINNED: &str = "pinned:";

/// `workspace`'s `manifest.yaml` drift on `config/<config>`, or `None` when
/// that tip already composes `instructions/**` — the steady state, which stages
/// nothing and spawns nothing. The lineage is a parameter for §8.7's reason:
/// what composes into a drone's context must be authored where the drone forks.
pub fn drift(workspace: &Path, config: &str) -> Option<DraftFile> {
    let base = crate::control::author::committed(workspace, config, MANIFEST_YAML)?;
    let want = authored(&base);
    (want != base).then(|| DraftFile {
        rel_path: MANIFEST_YAML.to_owned(),
        bytes: want.into_bytes(),
    })
}

/// `base` with [`GLOB`] present in `roles.worker.pinned`. A **fixed point**:
/// a manifest that already carries it — or that has no anchor to carry it —
/// comes back byte for byte.
pub fn authored(base: &str) -> String {
    let mut lines: Vec<String> = base.lines().map(str::to_owned).collect();
    if let Some((at, pad)) = insertion(&lines) {
        lines.insert(at, format!("{pad}- {GLOB}"));
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Where the glob line goes and how far it is indented: the end of
/// `roles.worker.pinned`'s block, and the indent its existing items wear (else
/// one step past the key). `None` when the anchor is missing or the glob is
/// already there.
fn insertion(lines: &[String]) -> Option<(usize, String)> {
    let roles = block(lines, 0..lines.len(), 0, ROLES)?;
    let role_depth = indent(lines.get(roles.start)?);
    let worker = block(lines, roles, role_depth, ROLE)?;
    let key_depth = indent(lines.get(worker.start)?);
    let pinned = block(lines, worker, key_depth, PINNED)?;
    let want = format!("- {GLOB}");
    let item = |i: usize| lines.get(i).filter(|l| is_item(l));
    if pinned
        .clone()
        .any(|i| lines.get(i).is_some_and(|l| l.trim() == want))
    {
        return None;
    }
    let pad = pinned.clone().find_map(item).map_or_else(
        || " ".repeat(key_depth + 2),
        |line| " ".repeat(indent(line)),
    );
    Some((pinned.end, pad))
}

/// The lines strictly inside `key`'s block within `span`: `key` must sit at
/// exactly `depth` spaces of indent, and its block runs to the next non-blank
/// line indented no deeper. `None` when `span` holds no such key.
///
/// **A sequence item at the key's own indent still belongs to the key.** YAML
/// lets a block sequence sit level with the mapping key that owns it, and
/// litany's own template is the *other* spelling — so a rule that closed on
/// indent alone would read a level `- goal.md` as the end of `pinned:` and
/// author the glob outside the list it belongs to.
fn block(lines: &[String], span: Range<usize>, depth: usize, key: &str) -> Option<Range<usize>> {
    let end = span.end;
    let start = span.into_iter().find(|&i| {
        lines
            .get(i)
            .is_some_and(|line| indent(line) == depth && line.trim() == key)
    })?;
    let close = ((start + 1)..end)
        .find(|&i| {
            lines.get(i).is_some_and(|line| {
                let level_item = indent(line) == depth && is_item(line);
                !line.trim().is_empty() && indent(line) <= depth && !level_item
            })
        })
        .unwrap_or(end);
    Some((start + 1)..close)
}

/// Whether a line is a block-sequence item.
fn is_item(line: &str) -> bool {
    line.trim_start().starts_with("- ")
}

/// A line's leading-space count. Tabs are not indentation in YAML, so a tabbed
/// line simply never matches a depth and the transform leaves it alone.
fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}
