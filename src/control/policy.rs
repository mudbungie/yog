//! The **standing capability policy** (VISION §4.11 item 4, DESIGN §8.6): the
//! per-workspace override of the shipped table, ruleset and secret list, read at
//! the config lineage's **live tip**.
//!
//! **Absence is the shipped defaults, and that is the whole severability
//! claim** — the `cadence.yaml` pattern one layer down. The defaults live in
//! code ([`super::judge::Table`], [`super::rules::DEFAULT`],
//! [`super::rules::SECRET_FRAGMENTS`]); the file is an *override*, so deleting
//! it deletes policy rather than the gate. Nothing seeds it: a shipped ruleset
//! materialized into config would make `rm capability.yaml` mean "no rules",
//! which is precisely the inversion the ruling forbids.
//!
//! **The live tip, never the governing commit.** An agent's `workflow.yaml` is
//! frozen where its branch forked (litany ARCH §2.2), and that is right for the
//! agent's own structure — but this is the *operator's* policy, and revocation
//! that only bound conversations started afterwards would not be revocation. So
//! the read is `config/default:capability.yaml` at its head, on every consult.
//!
//! The grammar is four keys, flat and line-oriented — deliberately not a YAML
//! subset with a parser to trust, and deliberately no new dependency:
//!
//! ```yaml
//! confinement: required        # refuse to fire drones with no OS layer
//! table:
//!   open-world: hold           # class → verdict, overriding the shipped row
//!                              # (this one row is the parked default, back)
//! rules:
//!   python: open-world         # program [qualifying words…] → effect class
//!   git push: target-write
//! secrets:
//!   - .kube                    # extra credential-adjacent path fragments
//! ```
//!
//! Reading is **total**: a line that names no class this control knows is not a
//! row, exactly as a mangled `ops.jsonl` line is not a check. What stops that
//! being silent is that the effective policy is itself readable — the operator
//! sees the rows that took, beside the file (§9.5's own answer to a blind
//! editor).

use std::path::Path;

use super::classify::Effect;
use super::judge::Ruling;
use super::rules::{DEFAULT, Reach, SECRET_FRAGMENTS};

/// The lineage the control reads its policy off — the one every workspace is
/// born on and every fresh agent forks from.
const DEFAULT_REF: &str = "refs/heads/config/default";

/// The policy file's name, beside `workflow.yaml` in the same config commit.
pub const CAPABILITY_YAML: &str = "capability.yaml";

/// The word `confinement:` takes when a workspace demands an OS layer.
const REQUIRED: &str = "required";

/// One classification row as the matcher reads it — owned, because an operator
/// row is read off a file and the shipped rows are `&'static`. The two are one
/// list by the time [`Policy::rows`] hands them over, operator rows first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The leading word's basename this row matches.
    pub program: String,
    /// Words that must all appear in the segment for the row to bite.
    pub words: Vec<String>,
    /// The class the row yields.
    pub reach: Reach,
}

/// A workspace's standing policy: the operator's overrides, and nothing else.
/// [`Policy::default()`] is the shipped state — every accessor then answers out
/// of the code consts, so absence and an empty file are one behaviour.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Policy {
    /// `confinement: required` — this workspace refuses to fire drones on a
    /// platform with no confinement layer (VISION §4.11 item 8).
    pub confinement_required: bool,
    table: Vec<(Effect, Ruling)>,
    rules: Vec<Row>,
    secrets: Vec<String>,
}

impl Policy {
    /// The workspace's policy at the live config tip; the shipped defaults when
    /// the file, the commit or the workspace is not there. An unreadable policy
    /// is *not* an error — it is a workspace that states no override.
    pub fn read(workspace: &Path) -> Policy {
        let bytes =
            crate::config_edit::branch::config_file(workspace, DEFAULT_REF, CAPABILITY_YAML);
        let text = bytes
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default();
        Policy::parse(&text)
    }

    /// Read one policy file. Total: every line the grammar does not recognise
    /// contributes nothing.
    pub fn parse(text: &str) -> Policy {
        let mut policy = Policy::default();
        let mut section = "";
        for raw in text.lines() {
            let line = strip_comment(raw);
            if line.trim().is_empty() {
                continue;
            }
            if !line.starts_with([' ', '\t', '-']) {
                section = policy.top_level(line);
                continue;
            }
            policy.item(section, line.trim_start().trim_start_matches('-').trim());
        }
        policy
    }

    /// One top-level line: either the `confinement:` scalar or the name of the
    /// block whose indented items follow. The section name is returned rather
    /// than stored, so the parse keeps no state a caller could observe.
    fn top_level(&mut self, line: &str) -> &'static str {
        let (key, value) = split_pair(line);
        match key {
            "confinement" => {
                self.confinement_required = value == REQUIRED;
                ""
            }
            "table" => "table",
            "rules" => "rules",
            "secrets" => "secrets",
            _ => "",
        }
    }

    /// One indented item, read under the block that opened it.
    fn item(&mut self, section: &str, item: &str) {
        match section {
            "table" => {
                let (class, verdict) = split_pair(item);
                if let (Some(effect), Some(ruling)) = (Effect::of(class), Ruling::of(verdict)) {
                    self.table.push((effect, ruling));
                }
            }
            "rules" => {
                let (key, class) = split_pair(item);
                let mut words = key.split_whitespace().map(str::to_owned);
                if let (Some(program), Some(effect)) = (words.next(), Effect::of(class)) {
                    self.rules.push(Row {
                        program,
                        words: words.collect(),
                        reach: Reach::Fixed(effect),
                    });
                }
            }
            "secrets" => self.secrets.push(item.to_owned()),
            _ => {}
        }
    }

    /// This class's ruling: the operator's row when they wrote one, else the
    /// shipped table. **Last override wins** — the file is read top to bottom
    /// and a later line is a later statement.
    pub fn ruling(&self, effect: Effect) -> Ruling {
        self.table
            .iter()
            .rev()
            .find(|(class, _)| *class == effect)
            .map_or_else(|| super::judge::Table::ruling(effect), |(_, r)| *r)
    }

    /// The effective ruleset in match order: the operator's rows, then the
    /// shipped ones. Operator rows lead because an override that could not
    /// reclassify `curl` would not be an override.
    pub fn rows(&self) -> Vec<Row> {
        self.rules
            .iter()
            .cloned()
            .chain(DEFAULT.iter().map(|(program, words, reach)| Row {
                program: (*program).to_owned(),
                words: words.iter().map(|w| (*w).to_owned()).collect(),
                reach: *reach,
            }))
            .collect()
    }

    /// The effective secret-path fragments: the shipped ones plus the
    /// operator's. Additive only — a workspace may widen what counts as
    /// credential-adjacent, never narrow it, because narrowing it is what an
    /// exfiltrating rule would want.
    pub fn secret_fragments(&self) -> Vec<String> {
        SECRET_FRAGMENTS
            .iter()
            .map(|f| (*f).to_owned())
            .chain(self.secrets.iter().cloned())
            .collect()
    }
}

/// `key: value`, trimmed. A line with no colon is all key and no value, which
/// every caller reads as "names nothing".
fn split_pair(line: &str) -> (&str, &str) {
    match line.split_once(':') {
        Some((key, value)) => (key.trim(), value.trim()),
        None => (line.trim(), ""),
    }
}

/// The line without its trailing `#` comment.
fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(head, _)| head)
}

#[cfg(test)]
mod tests;
