//! The **line** (§8.5): the boundary's third and last serialization — a slash
//! command, the spelling a human types in the composer and a TUI or a
//! teleoperator types anywhere there is a keyboard and no pointer.
//!
//! The other two are already here: the GUI's serialization is the in-RAM
//! [`Gesture`] variant its click-glue constructs, and the headless one is the
//! [`codec`](super::codec) JSON envelope. Neither is typable — one needs a
//! mouse, the other needs a JSON writer — so an operator driving yog through a
//! terminal, a chat window, or an agent's tool call had no spelling of their
//! own. This module is that spelling, and it is a *serialization*, never a
//! second implementation (VISION §8): [`parse`] builds the same variants the
//! click-glue builds, and the same [`dispatch`](super::dispatch::dispatch) /
//! [`answer`](super::answer::answer) chokepoints run them.
//!
//! **The line is terse and context-bearing; the envelope is total and
//! context-free.** `/message ship it` says what the operator means and nothing
//! about *where* — the workspace and the agent come from the seat's own
//! selection, carried in as a [`Context`], exactly as the composer's Enter
//! reads them off the focus today. A parameter the line omits and the context
//! cannot supply is a **refusal naming what is missing**, never a guess. The
//! envelope stays the spelling for a seat that holds no selection.
//!
//! **Help threads through it, once** (§8.5): `/help`, `/help <verb>`, a bare
//! `/`, and `<verb> --help` are all one gesture — [`Query::Help`](super::Query::Help),
//! read by a rule above the verb match rather than by an arm inside each verb.
//! Help is asked *about* a command, so it cannot be a parameter of one.
//!
//! **A new gesture without a line spelling fails to compile**: [`spell`] is
//! exhaustive over [`Gesture`], so a variant added to the boundary breaks this
//! module until it can be typed — the same compile gate the codec applies to
//! the headless envelope.
//!
//! **What the line does not mutate.** The two payloads that must reach a model
//! unmutated (§3.3, bl-6920) — a message's content and a prompt's goal — are
//! the whole tail, taken verbatim, and admit no flags. Everywhere else a value
//! is whitespace-normalized: a line is a line.

use crate::start::{BallSpec, Prepared};
use std::path::PathBuf;

mod args;
mod config;
mod fan;
mod fork;
mod parse;
mod queries;
mod spell;
#[cfg(test)]
mod tests;
mod verbs;

pub use config::USAGE as CONFIG_USAGE;
pub use parse::parse;
pub use spell::spell;
pub use verbs::ANSWER_USAGE;

/// The selection facts a seat holds (§8.5): what the line elides. Every field
/// is the same fact the GUI's click-glue resolves off the focus before it
/// constructs a variant — the composer's workspace and selected agent, the
/// balls fold's project, the §3.2 workspace name a `bl` verb stamps, the
/// focused ball, and the [`Prepare`](super::Action::Prepare) reply a deferred
/// [`Prompt`](super::Action::Prompt) fires with. A seat with no selection
/// (argv, a fresh TUI) hands [`Context::default`] and spells its targets out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Context {
    /// The focused workspace (§3.1) — the lernie family's target.
    pub workspace: Option<PathBuf>,
    /// The selected conversation's agent id (§11) — message/stop's target.
    pub agent: Option<String>,
    /// The focused project (§3.5) — the `bl` family's repo.
    pub project: Option<PathBuf>,
    /// The `--as` stamp every `bl` verb carries (§3.2): the ball's bound
    /// workspace name, never the operator's `$USER`.
    pub name: Option<String>,
    /// The focused ball (§3.5), whole: an id the ball verbs default to, and
    /// the spec `/prepare ball` starts from.
    pub ball: Option<BallSpec>,
    /// A prepared start (§8.1) awaiting its goal — what `/prompt` fires.
    pub prepared: Option<Prepared>,
}

/// Whether a draft is a command rather than something to say. A leading `/` is
/// the marker; a leading `//` is the **escape**, so a message that genuinely
/// starts with a slash can still be sent (see [`unescape`]). Read on the draft
/// as typed: a line that starts with a space is text, not a command.
pub fn is_command(draft: &str) -> bool {
    draft.starts_with('/') && !draft.starts_with("//")
}

/// The text a non-command draft actually says: `//…` sheds one slash, and
/// everything else is itself. One function, so the escape is a tested rule and
/// not a convention each seat re-implements.
pub fn unescape(draft: &str) -> String {
    match draft.strip_prefix("//") {
        Some(rest) => format!("/{rest}"),
        None => draft.to_owned(),
    }
}
