//! The **filename policy** (DESIGN §3.7 item 5): which files count as project
//! instructions, and in what order within one directory.
//!
//! The default lives in code and the override lives in the workspace's config
//! commit — `capability.yaml`'s exact shape one concern over (§8.6): **absence
//! is the shipped default, and that is the whole severability claim.** Deleting
//! `instructions.yaml` deletes the policy, not the mechanism, so removing a
//! default is a file removal and never a code edit.
//!
//! **The live tip, never the governing commit.** An agent's own structure
//! freezes where its branch forks (lernie ARCH §2.2), but this is the
//! *operator's* policy: a filename set that only bound conversations started
//! before the edit would not be a policy. So the read is
//! `config/default:instructions.yaml` at its head, at every fire.
//!
//! The grammar is one line shape — deliberately not a YAML subset with a parser
//! to trust, and deliberately no new dependency:
//!
//! ```yaml
//! - AGENTS.md
//! - CONTRIBUTING.md
//! ```
//!
//! Reading is **total**: a line that is not `- <bare filename>` is not a name,
//! exactly as a mangled `ops.jsonl` line is not a check. The file, when it
//! exists, is authoritative **including when it names nothing** — that is the
//! explicit opt-out, and it is why an existing file never falls back.

use crate::config_edit::branch::config_file;
use std::path::Path;

#[cfg(test)]
mod tests;

/// The shipped set. One name: this suite's own convention, which is what the
/// comparison asked for. Claude's filenames are deliberately absent — bl-e249
/// is evidence for a project-context mechanism, not a request to copy one.
const DEFAULT: &[&str] = &["AGENTS.md"];
/// The override's name, beside `capability.yaml` in the same config commit.
pub const INSTRUCTIONS_YAML: &str = "instructions.yaml";
/// The lineage the policy is read off — the one every workspace is born on.
const DEFAULT_REF: &str = "refs/heads/config/default";

/// `workspace`'s instruction filenames: its committed override, else the
/// shipped default. A workspace with no config commit, no such file, or bytes
/// git cannot hand back is the default — nothing to override with.
pub fn names(workspace: &Path) -> Vec<String> {
    match config_file(workspace, DEFAULT_REF, INSTRUCTIONS_YAML) {
        Ok(bytes) => parse(&String::from_utf8_lossy(&bytes)),
        Err(_) => DEFAULT.iter().map(|n| (*n).to_owned()).collect(),
    }
}

/// The names an override file declares, in its own order (pure).
fn parse(text: &str) -> Vec<String> {
    text.lines().filter_map(item).collect()
}

/// One `- <bare filename>` line's name, else `None`. The dash **and its
/// space** are the item marker, so a `---` document separator is not an item
/// named `--`. A name carrying a separator, or naming a directory hop, is not a
/// filename — the walk joins it onto each level and a hop would leave the level
/// it is meant to describe.
fn item(line: &str) -> Option<String> {
    let name = line.trim().strip_prefix("- ")?.trim();
    let bare = !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && name != "."
        && name != "..";
    bare.then(|| name.to_owned())
}
