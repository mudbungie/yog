//! `bash` classification: the shipped ruleset applied to every program a command
//! would actually run (VISION §4.11 item 1).
//!
//! The command is cut into segments ([`super::lex`]) and each segment is
//! classified on its own; the invocation's class is the **worst** of them,
//! because a command is as wide as its widest part — `ls && curl … | sh` is not
//! a read. Three things ride on top of the program's own row:
//!
//! - **Write redirections are the segment's writes.** `echo x > /etc/hosts`
//!   writes outside the root while its program says "read", so a redirect target
//!   folds in a target write (inside the root) or an open-world write (outside).
//! - **A leading `env` or `VAR=VAL` is a prefix, not the program.** `FOO=1 cargo
//!   build` is `cargo`. A bare `env` with nothing after it is the environment
//!   dump it looks like, and stays secret.
//! - **Secret paths outrank the program's row.** `cat ~/.ssh/id_rsa` is a read
//!   by its program and credential access by its operand.
//!
//! **Unmatched is open-world**, with no catch-all row to soften it: obfuscation
//! and novelty both land in the widest class short of loss, which is the
//! ruling's whole posture on classification error. The shipped table passes
//! that class (bl-1ef1); what the classification buys is that a workspace row
//! or a raised floor has something to bite on.

use super::classify::{Classified, Effect};
use super::lex::{Segment, segments};
use super::policy::{Policy, Row};
use super::root::Root;
use super::rules::Reach;

/// Classify one `bash` command against the writable root and the workspace's
/// effective ruleset (the operator's rows, then the shipped ones).
pub fn classify(command: &str, root: &Root, policy: &Policy) -> Classified {
    let rows = policy.rows();
    let secrets = policy.secret_fragments();
    segments(command)
        .iter()
        .map(|seg| segment(seg, root, &rows, &secrets))
        .reduce(|a, b| if b.effect > a.effect { b } else { a })
        // A command that runs no program at all — empty, or nothing but
        // separators. The general path with no segments, not a guard.
        .unwrap_or_else(|| Classified {
            effect: Effect::Read,
            why: "runs no command".to_owned(),
        })
}

/// One segment's class and the clause that says why.
fn segment(seg: &Segment, root: &Root, rows: &[Row], secrets: &[String]) -> Classified {
    let mut found = program_class(seg, root, rows);
    if let Some(target) = seg.redirects.iter().find(|t| !root.holds(&root.resolve(t))) {
        found = Classified {
            effect: found.effect.worst(Effect::OpenWorld),
            why: format!("redirects output to {target}, outside the writable root"),
        };
    } else if !seg.redirects.is_empty() {
        found.effect = found.effect.worst(Effect::TargetWrite);
    }
    if let Some(fragment) = secret_fragment(seg, secrets) {
        found = Classified {
            effect: found.effect.worst(Effect::Secret),
            why: format!("names {fragment}, which is credential-adjacent"),
        };
    }
    found
}

/// The class the segment's *program* carries, before redirections and secret
/// operands are folded in.
fn program_class(seg: &Segment, root: &Root, rows: &[Row]) -> Classified {
    let words = strip_prefix(&seg.words);
    let program = program_of(words);
    if let Some(rule) = rule_for(program, words, rows) {
        return matched(&rule, program, words, root);
    }
    if words.is_empty() {
        // The prefix strip consumed the whole segment. A bare `env` is the
        // environment dump it looks like; a bare assignment sets a variable a
        // discarded shell will never use.
        let bare_env = program_of(&seg.words) == "env";
        return Classified {
            effect: if bare_env {
                Effect::Secret
            } else {
                Effect::Read
            },
            why: if bare_env {
                "dumps the environment".to_owned()
            } else {
                "runs no program".to_owned()
            },
        };
    }
    Classified {
        effect: Effect::OpenWorld,
        why: format!("no rule classifies `{program}`"),
    }
}

/// The class a matched row yields, with the clause that names what decided it.
fn matched(rule: &Row, program: &str, words: &[String], root: &Root) -> Classified {
    match rule.reach {
        Reach::Fixed(effect) => Classified {
            effect,
            why: format!("`{program}`"),
        },
        Reach::ByRoot { inside, outside } => {
            let ops = operands(words);
            match ops.iter().find(|o| !root.holds(&root.resolve(o))) {
                None => Classified {
                    effect: inside,
                    why: format!("`{program}` stays inside the writable root"),
                },
                Some(operand) => Classified {
                    effect: outside,
                    why: format!("`{program}` reaches {operand}, outside the writable root"),
                },
            }
        }
    }
}

/// The first row matching this program and these words, in match order. Handed
/// back **owned**: three reference parameters leave nothing for elision to
/// borrow from, and a named lifetime on a signature is what the house rules
/// forbid outright — a row is three small fields, so the clone costs the
/// argument nothing.
fn rule_for(program: &str, words: &[String], rows: &[Row]) -> Option<Row> {
    rows.iter()
        .find(|row| row.program == program && row.words.iter().all(|w| has_word(words, w)))
        .cloned()
}

/// The segment's program: the leading word's basename, so `/usr/bin/curl` and
/// `curl` match the same row. Empty when the prefix strip consumed everything —
/// a bare `env`.
fn program_of(words: &[String]) -> &str {
    words
        .first()
        .map(String::as_str)
        .unwrap_or_default()
        .rsplit('/')
        .next()
        .unwrap_or_default()
}

/// Strip a leading `env` and any leading `VAR=VAL` assignments: they set up the
/// program, they are not the program.
fn strip_prefix(words: &[String]) -> &[String] {
    let mut rest = words;
    loop {
        let Some(first) = rest.first() else {
            return rest;
        };
        let is_assignment = first
            .split_once('=')
            .is_some_and(|(name, _)| !name.is_empty() && !name.contains('/'));
        let is_env = first.rsplit('/').next() == Some("env");
        if !is_assignment && !is_env {
            return rest;
        }
        rest = rest.get(1..).unwrap_or_default();
    }
}

/// Whether `want` appears among `words`. A short flag matches inside a bundle —
/// `-f` bites on `-fd` and `-rf` — so a rule need not enumerate the ways a
/// shell user spells a pair of flags. Long flags and plain words match exactly.
pub fn has_word(words: &[String], want: &str) -> bool {
    if words.iter().any(|w| w == want) {
        return true;
    }
    let Some(letters) = short_flag(want) else {
        return false;
    };
    words
        .iter()
        .filter_map(|w| short_flag(w))
        .any(|bundle| letters.chars().all(|c| bundle.contains(c)))
}

/// The letters of a single-dash short flag (`-fd` → `fd`), or `None` for a long
/// flag or a plain word.
fn short_flag(word: &str) -> Option<&str> {
    word.strip_prefix('-')
        .filter(|rest| !rest.is_empty() && !rest.starts_with('-'))
}

/// The segment's path operands: every non-flag word after the program. A word
/// that is a flag's value is included too, which only ever widens the operand
/// set — and widening it can only move a `ByRoot` row toward the safer class.
fn operands(words: &[String]) -> Vec<String> {
    words
        .iter()
        .skip(1)
        .filter(|w| !w.starts_with('-'))
        .cloned()
        .collect()
}

/// The first secret path fragment any word of the segment carries, out of the
/// workspace's effective list (shipped plus the operator's additions).
fn secret_fragment(seg: &Segment, secrets: &[String]) -> Option<String> {
    seg.words
        .iter()
        .chain(seg.redirects.iter())
        .find_map(|w| secrets.iter().find(|f| w.contains(f.as_str())).cloned())
}

#[cfg(test)]
mod tests;
