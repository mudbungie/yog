//! The `bl` family's line grammar (§8.5) — the ball an id-taking verb acts on
//! and the two authoring verbs with their payloads.
//!
//! Its own file on the seam every sibling family here is already cut on
//! (`fan`, `fork`, `tools`, `config`) and the one `codec/balls` draws one layer
//! down: [`super::verbs`] holds the litany-family and policy arms, this holds
//! the verbs that address *a ball in a project*.

use super::{Context, args};
use crate::actions::verbs::{Verb, edit};
use crate::boundary::{Action, Gesture};

fn act(action: Action) -> Gesture {
    Gesture::Act(action)
}

/// The ball an id-taking verb acts on: the typed word, else the focused ball.
pub(super) fn id(tail: &str, ctx: &Context, verb: &str) -> Result<String, String> {
    args::ball_id(args::optional_word(tail, verb)?, ctx, verb)
}

/// **The family's one door** (bl-92d3), the shape [`super::fan`]'s already has:
/// [`super::parse`]'s table names the family once and comes here for which
/// member, so the roster reads as a roster instead of spelling one grammar
/// three times inline. `verb` has already been matched there, so an unlisted
/// one cannot arrive — the same contract `codec::balls::decode` keeps.
pub(super) fn read(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    match verb {
        "create" => create(tail, ctx, verb),
        "update" => update(tail, ctx, verb),
        _ => identified(tail, ctx, verb),
    }
}

/// `/close [id]`, `/assign [id]`, `/release [id]` — the three that differ in
/// nothing but which act they name, read once. The fallthrough is `release`,
/// and [`read`]'s own contract is what keeps it honest.
fn identified(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let (project, id, name) = (
        args::project(ctx, verb)?,
        id(tail, ctx, verb)?,
        args::name(ctx, verb)?,
    );
    Ok(act(Action::Ball(match verb {
        "close" => Verb::Close { project, id, name },
        "assign" => Verb::Assign { project, id, name },
        _ => Verb::Release { project, id, name },
    })))
}

/// `/create <title…> [--body <text…>] [scheduling flags…]`.
fn create(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let (title, flags) = args::split_flags(tail);
    only(&flags, &["body"], verb)?;
    Ok(act(Action::Ball(Verb::Create {
        project: args::project(ctx, verb)?,
        name: args::name(ctx, verb)?,
        fields: edit::Create {
            title: args::required(&title, verb, "a title")?,
            body: args::flag(&flags, "body", verb)?,
            fields: fields(&flags, verb)?,
        },
    })))
}

/// `/update [id] [--title T] [--body B] [--note N] [scheduling flags…]` — at
/// least one field, or the line asked for nothing and the refusal says so.
fn update(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let (positional, flags) = args::split_flags(tail);
    only(&flags, &["title", "body", "note"], verb)?;
    let edits = edit::Update {
        title: args::flag(&flags, "title", verb)?,
        body: args::flag(&flags, "body", verb)?,
        note: args::flag(&flags, "note", verb)?,
        fields: fields(&flags, verb)?,
    };
    if edits == edit::Update::default() {
        return Err(format!("/{verb}: nothing to change; usage: {UPDATE_USAGE}"));
    }
    Ok(act(Action::Ball(Verb::Update {
        project: args::project(ctx, verb)?,
        id: id(&positional, ctx, verb)?,
        name: args::name(ctx, verb)?,
        fields: edits,
    })))
}

/// `/update`'s usage, said once — the refusal above and the help page read it.
pub const UPDATE_USAGE: &str = "/update [id] [--title T] [--body B] [--note N] [--priority N|--no-priority] \
     [--tag T|--no-tag T] [--parent ID|--no-parent] [--needs ID[:OP]|--no-needs ID]";

/// `/create`'s usage, beside it for the same reason.
pub const CREATE_USAGE: &str = "/create <title…> [--body <text…>] [--priority N] [--tag T] \
     [--parent ID] [--needs ID[:OP]]";

/// The eight flags the four §8.2 scheduling facts are said with (bl-dbde). Both
/// authoring verbs take all eight: a clearing form at create is a no-op rather
/// than a refusal, because a new ball's fields start empty and that is the
/// general path at zero input, not a case of its own.
const FIELD_FLAGS: [&str; 8] = [
    "priority",
    "no-priority",
    "tag",
    "no-tag",
    "parent",
    "no-parent",
    "needs",
    "no-needs",
];

/// [`args::only`] over the verb's own flags **plus** the scheduling eight.
fn only(flags: &[(String, String)], own: &[&str], verb: &str) -> Result<(), String> {
    let mut allowed = own.to_vec();
    allowed.extend(FIELD_FLAGS);
    args::only(flags, &allowed, verb)
}

/// The scheduling facts, **in the order they were typed** — the fold to argv
/// applies them in order, so the reader may not sort or dedupe them, and a
/// repeated `--tag` is two applications rather than a collision.
fn fields(flags: &[(String, String)], verb: &str) -> Result<Vec<edit::Field>, String> {
    flags
        .iter()
        .filter_map(|(key, value)| field(key, value, verb))
        .collect()
}

/// One flag read as a [`edit::Field`], or `None` if it is not one of the eight
/// (the verb's own `--title`/`--body`/`--note`, already read by name).
fn field(key: &str, value: &str, verb: &str) -> Option<Result<edit::Field, String>> {
    Some(match key {
        "priority" => valued(value, key, verb).and_then(|n| priority(&n, key, verb)),
        "no-priority" => bare(value, key, verb).map(|()| edit::Field::Priority(None)),
        "parent" => valued(value, key, verb).map(|id| edit::Field::Parent(Some(id))),
        "no-parent" => bare(value, key, verb).map(|()| edit::Field::Parent(None)),
        "tag" | "no-tag" => valued(value, key, verb).map(|tag| edit::Field::Tag {
            tag,
            on: key == "tag",
        }),
        "needs" | "no-needs" => valued(value, key, verb).map(|edge| edit::Field::Needs {
            edge,
            on: key == "needs",
        }),
        _ => return None,
    })
}

/// A priority is a number and yog says so — the one thing here it can judge
/// without holding an opinion balls already holds.
fn priority(word: &str, key: &str, verb: &str) -> Result<edit::Field, String> {
    word.parse()
        .map(|n| edit::Field::Priority(Some(n)))
        .map_err(|_| format!("/{verb}: --{key} takes a number, got {word:?}"))
}

/// A flag that must carry text — a bare one is a refusal, not an empty value,
/// exactly as [`args::flag`] rules it.
fn valued(value: &str, key: &str, verb: &str) -> Result<String, String> {
    if value.is_empty() {
        Err(format!("/{verb}: --{key} needs a value"))
    } else {
        Ok(value.to_owned())
    }
}

/// A clearing flag, which takes nothing: `--no-priority 3` is a typo the
/// operator must see rather than a priority silently dropped.
fn bare(value: &str, key: &str, verb: &str) -> Result<(), String> {
    if value.is_empty() {
        Ok(())
    } else {
        Err(format!("/{verb}: --{key} takes no value, got {value:?}"))
    }
}

/// The §8.5 writer's half of [`fields`]: the same list, in the same order, so
/// the line round trip gives back the gesture it spelled.
pub(super) fn spell_fields(fields: &[edit::Field]) -> String {
    fields.iter().map(spell_field).collect()
}

fn spell_field(field: &edit::Field) -> String {
    match field {
        edit::Field::Priority(Some(n)) => format!(" --priority {n}"),
        edit::Field::Priority(None) => " --no-priority".to_owned(),
        edit::Field::Parent(Some(id)) => format!(" --parent {id}"),
        edit::Field::Parent(None) => " --no-parent".to_owned(),
        edit::Field::Tag { tag, on } => format!(" --{}tag {tag}", negation(*on)),
        edit::Field::Needs { edge, on } => format!(" --{}needs {edge}", negation(*on)),
    }
}

fn negation(on: bool) -> &'static str {
    if on { "" } else { "no-" }
}
