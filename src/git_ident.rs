//! **git has no identity here**, said once (DESIGN §8.1, bl-c28c) — the named
//! prerequisite a start's own dead words are read back to.
//!
//! Every durable thing yog makes is a git commit somebody else writes: `litany
//! new` authors the workspace's first `config/default` commit, `litany config`
//! advances a lineage, and `bl` seals a ball. All of them fail on a box where
//! git can name no author, and a stranger following the READMEs has no reason
//! to have set one. The sighting was the **first** act of the Settler story
//! (STORIES S0/S9) on a clean container: `lernie start home "hi"` answered a
//! twelve-line `litany new` capture with git's *"Please tell me who you are"*
//! text embedded as `\n`s inside an `error` string — which is exactly the shape
//! a control boundary exists to remove, and which named no prerequisite at all.
//!
//! # Why this is read off the failure rather than gated at the door
//!
//! The obvious answer is a rung at the `Prompt` door beside
//! [`signin`](crate::boundary::dispatch): a static fact, knowable before
//! anything is written. It is the wrong answer here, and the reason is a fact
//! about git rather than about yog: **an identity is a property of a
//! repository, not of a box.** `git config user.email` without `--global` is
//! ordinary and this repository's own suite depends on it — every fixture that
//! commits sets a local identity precisely so the suite passes on a box with no
//! global one. A door rung would therefore ask the *engine's* environment about
//! a commit some other repository will make, and answer confidently for boxes
//! where the two disagree in both directions.
//!
//! So the shape is [`dialect_decline`](crate::config_edit::brazen)'s, which
//! this tree already settled for the same family of question (bl-5252): gate
//! what is knowable, and read a dead act's **own words** back to the same named
//! fact, so the operator gets one sentence instead of a capture.
//!
//! # Why the world does not simply supply an identity of its own
//!
//! The nested world owns those repositories, so a world-supplied `GIT_AUTHOR_*`
//! looks like the elegant answer. It is not available. §16.2's override set is
//! handed to **every** child, and litany hands its environment on to every tool
//! subprocess, so those variables would reach an agent's own `git commit` in
//! the operator's project repository. They outrank `user.name`/`user.email`, so
//! they would *override* a configured identity rather than standing behind one
//! — stamping the operator's own work with a name yog chose — and git has no
//! low-precedence *name* variable to stand behind it with (`EMAIL` covers only
//! the address). A per-repository identity cannot be written either: `litany
//! new` creates the repository and commits in one act, with no window between
//! them.

/// **git's own banner for an unnameable author or committer**, and the whole of
/// the signal. git prints it under `Author identity unknown` and under
/// `Committer identity unknown`, ahead of every `fatal:` variant of the same
/// complaint (`unable to auto-detect email address`, `no email was given and
/// auto-detection is disabled`), and prints it for nothing else — so one exact
/// string answers for the family, the way brazen's dialect declines are keyed
/// on the dialect naming itself. Matching on `fatal:` lines instead would be a
/// list this file has to keep in step with git's.
const BANNER: &str = "*** Please tell me who you are.";

/// The one sentence. Fact first and remedy last, and the remedy is git's own
/// two commands spelled out, because an operator who has never set an identity
/// has no reason to know them.
pub(crate) const PREREQUISITE: &str = "git has no identity on this box, so the commit this act \
     makes cannot be authored: set one with `git config --global user.name \"Your Name\"` and \
     `git config --global user.email \"you@example.com\"`, then say it again";

/// [`PREREQUISITE`] when `capture` is a child's complaint that git could name
/// nobody, `None` when it is any other failure — which then rides back as
/// itself, since a capture yog cannot read is still the truest thing it has.
pub(crate) fn prerequisite(capture: &str) -> Option<String> {
    capture.contains(BANNER).then(|| PREREQUISITE.to_owned())
}

#[cfg(test)]
mod tests;
