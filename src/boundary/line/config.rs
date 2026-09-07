//! The §9 config family's line grammar (§8.5, bl-3f46) — reader and writer in
//! one home, so a spelling cannot be typed and unwritten, or written and
//! unreadable.
//!
//! **The text is the whole tail, verbatim.** A config file's whitespace is
//! semantic — a YAML block key is its indentation — so the destination is
//! spelled as *leading words* and everything after them is the file, taken as
//! typed. That is the same rule §3.3's message content and prompt goal ride,
//! and for the same reason: these are the payloads a line must not mutate.
//!
//! Which is why the lineage modes are three sibling words rather than flags:
//! `--from base` after the text would be read as part of the file, and before
//! it would need a terminator. `branch` advances, `fork` names its source,
//! `orphan` starts fresh — one word each, and the grammar stays positional.

use super::{Context, args};
use crate::boundary::config::ConfigFile;
use crate::boundary::config::{Read, Write};
use crate::boundary::{Action, Gesture, Query};
use crate::config_edit::branch::edit::EditOrigin;
use crate::model_pick::{LEVELS, Tuning};
use crate::world::marks;

/// `/config <destination…> <text…>` — the destination's words, then the file;
/// `/config <destination…>` with nothing after them is the read (§8.5,
/// bl-0164), **a lineage included** since bl-dff8: `/config branch <lineage>
/// <path>` answers that file's bytes out of the lineage tip, which is the pane's
/// own Load and the very hint its button carries. It costs no case a write
/// already used — `ApplyConfig` refuses an empty lineage text — and it is what
/// makes an Apply on a lineage something other than a blind overwrite.
/// `/lineages` is the browse beside it: which lineages exist, and what each
/// holds.
pub(super) fn config(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let (target, rest) = args::first_word(tail);
    let (file, rest) = match target.as_str() {
        // The wall the file lives in is the seat's own workspace (bl-fcd5) —
        // read from the context like every other elided target, so `--ws`
        // states it headlessly and focus states it at the window.
        "brazen" => (
            ConfigFile::Brazen {
                workspace: args::workspace(ctx, verb)?,
            },
            rest,
        ),
        "models" => (ConfigFile::LitanyModels, rest),
        "cadence" => (ConfigFile::Cadence, rest),
        "workflow" => {
            let (name, rest) = word(&rest, verb, "a workflow name")?;
            (ConfigFile::LitanyWorkflow { name }, rest)
        }
        "branch" => lineage(&rest, ctx, verb, None)?,
        "orphan" => lineage(&rest, ctx, verb, Some(EditOrigin::Orphan))?,
        "fork" => {
            let (lineage_name, rest) = word(&rest, verb, "the lineage name")?;
            let (source, rest) = word(&rest, verb, "the lineage to fork from")?;
            let (path, rest) = word(&rest, verb, "the file's path in the lineage")?;
            (
                branch(ctx, verb, lineage_name, EditOrigin::Fork { source }, path)?,
                rest,
            )
        }
        other => Err(format!(
            "/{verb}: unknown destination {other:?}; usage: {USAGE}"
        ))?,
    };
    if rest.trim().is_empty() {
        return Ok(Gesture::Ask(Query::Config(Read::File { file })));
    }
    Ok(write(Write::Apply {
        file,
        text: args::required(&rest, verb, "the file's text")?,
    }))
}

/// `/marks <branch>` amends the §16.3 tracking branch of the seat's own
/// workspace; `/marks` bare reads it (§8.5, bl-0164) — a branch is always
/// required to write, so the empty tail cannot mean anything else.
///
/// **One word, and it is the branch itself.** The superseded grammar spelled
/// three modes (`shared | stealth | branch <name>`) because the knob was a
/// project's publish policy; the per-agent ruling makes it an agent's tracking
/// space, whose whole value is a branch name — so `balls/tasks` says "the
/// project's shared board" outright, and there is no mode word left to learn
/// or to disagree with the value beside it.
pub(super) fn marks(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let (branch, rest) = args::first_word(tail);
    if branch.is_empty() {
        return Ok(Gesture::Ask(Query::Config(Read::Marks {
            workspace: args::workspace(ctx, verb)?,
        })));
    }
    args::none(&rest, verb)?;
    if !marks::lawful(&branch) {
        return Err(format!("/{verb}: {}", marks::REFUSAL));
    }
    Ok(write(Write::Marks {
        workspace: args::workspace(ctx, verb)?,
        branch,
    }))
}

/// `/model <role> <provider> <model>` — the §9.4 pick on the focused workspace.
pub(super) fn model(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let [role, provider, model] = *tail.split_whitespace().collect::<Vec<_>>() else {
        return Err(format!(
            "/{verb}: usage: /model <role> <provider> <model-id>"
        ));
    };
    Ok(write(Write::Pick {
        workspace: args::workspace(ctx, verb)?,
        role: role.to_owned(),
        provider: provider.to_owned(),
        model: model.to_owned(),
    }))
}

/// **The §9.6 learning-loop pair** (bl-dd88), its own file at the budget on the
/// seam the pair itself draws: everything here writes a config file or reads
/// its bytes, and those two are about a *candidate* commit.
pub(super) mod proposals;

/// One §9 write, in the gesture that carries them all — the fold's one seam on
/// this side (bl-dd88), so no reader below states the carrier twice.
fn write(w: Write) -> Gesture {
    Gesture::Act(Action::Config(w))
}

/// `/effort <role> <low|medium|high|off>` — the §9.4 reasoning level on the
/// focused workspace (bl-23bd). Two words, fixed, the shape `/model`'s three
/// take; the level is validated **here**, against the one vocabulary
/// [`LEVELS`] states, so an unspellable value never reaches the grammar.
pub(super) fn effort(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let [role, level] = *tail.split_whitespace().collect::<Vec<_>>() else {
        return Err(usage(verb));
    };
    let level = match level {
        OFF => None,
        word => Some(crate::model_pick::Effort::parse(word).ok_or_else(|| usage(verb))?),
    };
    Ok(write(Write::Tune(Tuning::Effort {
        workspace: args::workspace(ctx, verb)?,
        role: role.to_owned(),
        level,
    })))
}

/// `/priority <role> <on|off>` — ask this role's provider for the priority
/// lane, or stop asking. The same two-word shape, over the one other closed
/// vocabulary the pair has.
pub(super) fn priority(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let [role, state] = *tail.split_whitespace().collect::<Vec<_>>() else {
        return Err(PRIORITY_USAGE.to_owned());
    };
    let on = match state {
        ON => true,
        OFF => false,
        _ => return Err(PRIORITY_USAGE.to_owned()),
    };
    Ok(write(Write::Tune(Tuning::Priority {
        workspace: args::workspace(ctx, verb)?,
        role: role.to_owned(),
        on,
    })))
}

/// The word that removes a tuning field — the same one for both knobs, because
/// they have the same off state and an operator should not have to remember two.
const OFF: &str = "off";
/// `/priority`'s other word.
const ON: &str = "on";

/// `/effort`'s refusal, one sentence for both ways to get it wrong (the wrong
/// number of words, and a level outside the vocabulary): the usage names the
/// vocabulary, so a reader who saw either has been told the whole rule.
fn usage(verb: &str) -> String {
    format!("/{verb}: usage: /effort <role> <{LEVELS}>")
}

/// `/priority`'s, said once for the same reason. A const rather than a
/// formatted sentence because its verb cannot vary.
pub const PRIORITY_USAGE: &str = "/priority: usage: /priority <role> <on|off>";

/// Spell a destination as the leading words `/config` reads it back from — the
/// writer half of the grammar above, and the reason [`spell`] can be exhaustive.
///
/// [`spell`]: super::spell
pub(super) fn target_words(file: &ConfigFile) -> String {
    match file {
        ConfigFile::Brazen { .. } => "brazen".to_owned(),
        ConfigFile::LitanyModels => "models".to_owned(),
        ConfigFile::Cadence => "cadence".to_owned(),
        ConfigFile::LitanyWorkflow { name } => format!("workflow {name}"),
        ConfigFile::Branch {
            lineage,
            origin,
            path,
            ..
        } => match origin {
            EditOrigin::Advance => format!("branch {lineage} {path}"),
            EditOrigin::Fork { source } => format!("fork {lineage} {source} {path}"),
            EditOrigin::Orphan => format!("orphan {lineage} {path}"),
        },
    }
}

/// The `/config` usage line, said once — refusals and [`help`] read this one
/// string rather than each restating seven destinations.
///
/// [`help`]: crate::boundary::help
pub const USAGE: &str = "/config brazen|models|cadence <text…> | /config workflow <name> <text…> | \
     /config branch|orphan <lineage> <path> <text…> | /config fork <lineage> <source> <path> <text…>";

/// The two lineage forms that take no source: `<lineage> <path>` then the text.
/// `Some(origin)` is the orphan; `None` advances.
fn lineage(
    rest: &str,
    ctx: &Context,
    verb: &str,
    origin: Option<EditOrigin>,
) -> Result<(ConfigFile, String), String> {
    let (name, rest) = word(rest, verb, "the lineage name")?;
    let (path, rest) = word(&rest, verb, "the file's path in the lineage")?;
    let origin = origin.unwrap_or(EditOrigin::Advance);
    Ok((branch(ctx, verb, name, origin, path)?, rest))
}

/// The lineage destination, on the seat's own workspace — the fact a line
/// elides everywhere else too.
fn branch(
    ctx: &Context,
    verb: &str,
    lineage: String,
    origin: EditOrigin,
    path: String,
) -> Result<ConfigFile, String> {
    Ok(ConfigFile::Branch {
        workspace: args::workspace(ctx, verb)?,
        lineage,
        origin,
        path,
    })
}

/// Peel one required word off the front, keeping the rest verbatim.
fn word(rest: &str, verb: &str, what: &str) -> Result<(String, String), String> {
    let (head, tail) = args::first_word(rest);
    if head.is_empty() {
        return Err(format!("/{verb}: {what} is required"));
    }
    Ok((head, tail))
}

#[cfg(test)]
mod tests;

/// Route one of the §9 family's verbs to its reader, or answer `None` for a
/// word that is not the family's (bl-dd88). **The match is the table**: an
/// allowlist beside it would be a second home for "which verbs are ours" and
/// would drift from the router the day one is added — and an unreachable
/// fallthrough arm is exactly the special case a missing reframe leaves behind.
pub(super) fn verb(verb: &str, tail: &str, ctx: &Context) -> Option<Result<Gesture, String>> {
    Some(match verb {
        "config" => config(tail, ctx, verb),
        "marks" => marks(tail, ctx, verb),
        "model" => model(tail, ctx, verb),
        // The §9.4 tuning pair (bl-23bd): the other two writers of the same
        // role assignment, each its own verb because a toggle must not make the
        // operator restate the pointer it is not changing.
        "effort" => effort(tail, ctx, verb),
        "priority" => priority(tail, ctx, verb),
        // The §9.6 learning-loop pair (bl-dd88): the listing (plural, a read)
        // and the settle (singular, an act). Two words rather than one verb
        // with a mode, because a read and a destructive act must not be one
        // typo apart.
        "proposals" => proposals::proposals(tail, ctx, verb),
        "proposal" => proposals::proposal(tail, ctx, verb),
        _ => return None,
    })
}
