//! **Project instructions freeze at the binding** (DESIGN §3.7, from bl-e249's
//! Claude Code comparison): the deterministic walk that finds a project's
//! instruction files and the `--pin` specs that freeze them into the agent's
//! dispatch commit before the first inference.
//!
//! The input is the §3.3 typed work target and nothing else — no rung table:
//! the ball rung's claim-derived `work/<id>` worktree, the path rung's
//! directory, and for the bare rung (and a not-yet-created ball) no binding at
//! all, which discovers nothing. The general path with empty inputs.
//!
//! **yog reads no instruction bytes.** It stats candidates and names paths;
//! litany's caller-supplied pinned documents (ARCH §2.5, released 0.0.4 as
//! bl-fb5c, in the `=0.0.8` pin) load, validate, write and commit them — in the
//! CLI layer, before any branch, ref or inference exists. There is no yog-side
//! copy of anything, which is the whole "no opaque automatic memory" clause.
//!
//! The two halves that make the freeze *visible* live beside this walk:
//! [`names`] is the severable filename policy, [`manifest`] the glob that makes
//! a frozen document actually compose into assembled context (§3.7 item 4 —
//! pinning is not composing).

use std::path::{Path, PathBuf};

pub mod manifest;
pub mod names;
#[cfg(test)]
mod tests;

/// The largest instruction document that rides. A bigger one is **skipped
/// whole, never truncated**: half a rule reads exactly like a whole rule, so a
/// truncated instruction is worse than a missing one (§3.7 item 1).
const MAX_BYTES: u64 = 128 * 1024;
/// The most documents one freeze carries — the bound on a chain of deeply
/// nested directories each declaring its own, and what keeps the rank two
/// digits wide.
const MAX_DOCS: usize = 16;
/// The pin destination's first segment. None of litany's reserved harness names
/// (`goal.md`, `soul.md`, `name`, the control files, `descriptions/`,
/// `messages/`, `summary/`), so the pin is accepted.
const DEST_ROOT: &str = "instructions";
/// What marks a git checkout root: a directory in an ordinary clone, a **file**
/// in a `work/<id>` worktree (the gitdir pointer). Either is the authority root.
const GIT_MARK: &str = ".git";
/// `--pin`'s own separator, split at the *first* occurrence — so a destination
/// may not contain one (a source may).
const SEP: char = '=';

/// The `--pin <dest>=<src>` arguments for every instruction document the
/// `binding`'s project declares, in precedence order — outermost directory
/// first, `names`' declared order within each.
///
/// This is the module's whole interface: the fire appends these to its one
/// argv (§3.3's "built once and spawned *and* logged from it"), so what the
/// agent froze and what `ops.jsonl` records can never disagree.
pub fn specs(binding: &Path, names: &[String]) -> Vec<String> {
    discover(binding, names)
        .into_iter()
        .map(|p| format!("{}{SEP}{}", p.dest, p.src.display()))
        .collect()
}

/// One document to freeze: where it lands on the dispatch commit, and the file
/// it is read from.
struct Pin {
    dest: String,
    src: PathBuf,
}

/// The §3.7 walk: authority root → binding, each configured name at each level,
/// ranked in discovery order.
fn discover(binding: &Path, names: &[String]) -> Vec<Pin> {
    let root = authority_root(binding);
    let mut out: Vec<Pin> = Vec::new();
    for dir in chain(&root, binding) {
        for name in names {
            if out.len() >= MAX_DOCS {
                return out;
            }
            let src = dir.join(name);
            let Some(rel) = admissible(&src, &root) else {
                continue;
            };
            out.push(Pin {
                dest: format!("{DEST_ROOT}/{:02}/{rel}", out.len()),
                src,
            });
        }
    }
    out
}

/// The nearest ancestor of `binding` — itself included — holding a [`GIT_MARK`],
/// else `binding`. **The walk never ascends above it**, which is the whole
/// answer to untrusted parent instructions (§3.7 item 1): a `$HOME/AGENTS.md`
/// is not skipped by a check, it is unreachable by construction.
fn authority_root(binding: &Path) -> PathBuf {
    let mut cur = binding;
    loop {
        if std::fs::symlink_metadata(cur.join(GIT_MARK)).is_ok() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => return binding.to_path_buf(),
        }
    }
}

/// `root` → `binding` inclusive, outermost first — the precedence order, so the
/// most specific instructions arrive last. `root` is an ancestor-or-self of
/// `binding` by construction ([`authority_root`] only ascends), so the ascent
/// always terminates on it.
fn chain(root: &Path, binding: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut cur = Some(binding);
    while let Some(dir) = cur {
        out.push(dir.to_path_buf());
        if dir == root {
            break;
        }
        cur = dir.parent();
    }
    out.reverse();
    out
}

/// `src`'s authority-root-relative path when it may be frozen, else `None`:
/// a **regular file** by `symlink_metadata` (a symlink is skipped — the freeze
/// is byte-exact and a link can point out of the root), within [`MAX_BYTES`],
/// and spellable as a destination (UTF-8, and no [`SEP`], which `--pin` splits
/// on). A candidate that only fails the *read* later is litany's loud pre-fork
/// refusal, not a silent partial (§3.7 item 2).
fn admissible(src: &Path, root: &Path) -> Option<String> {
    let meta = std::fs::symlink_metadata(src).ok()?;
    let rel = src.strip_prefix(root).ok()?.to_str()?;
    (meta.is_file() && meta.len() <= MAX_BYTES && !rel.contains(SEP)).then(|| rel.to_owned())
}
