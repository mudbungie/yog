//! The line reader (§8.5): one slash command in, one [`Gesture`] out — the
//! same variant the GUI's click-glue would have constructed for that gesture,
//! so what follows is the one dispatch every seat shares.
//!
//! The verb table below **is** [`help::table`](crate::boundary::help::table)'s
//! other half; a test holds the two in step, so a spelling cannot be advertised
//! and unreadable, or readable and unadvertised. An unknown verb refuses with
//! the roster attached.
//!
//! **Help is read once, above the match** ([`asks_help`]): it is asked *about* a
//! verb, so it is not that verb's parameter and must not be eighteen arms. Four
//! spellings, one gesture — `/help`, `/help <verb>`, `<verb> --help|-h`, and a
//! bare `/`, which is the question with nothing named.

use super::verbs::{self, children, create, id, max, moved, payload, update, work_file};
use super::{Context, args, config};
use crate::boundary::{Action, Gesture, Query, help};

/// Read one line into the gesture it spells (§8.5). `ctx` supplies what the
/// line elides; every unresolvable parameter is a refusal naming it, never a
/// default — a gesture is an instruction, and a guessed target mutates the
/// wrong thing.
pub fn parse(input: &str, ctx: &Context) -> Result<Gesture, String> {
    let body = input.trim().strip_prefix('/').ok_or_else(|| {
        format!(
            "not a command — a line crossing the boundary starts with '/'\n{}",
            help::roster()
        )
    })?;
    let (verb, tail) = match body.split_once(char::is_whitespace) {
        Some((verb, tail)) => (verb, tail.trim_start()),
        None => (body, ""),
    };
    if let Some(query) = asks_help(verb, tail)? {
        return Ok(ask(query));
    }
    match verb {
        // The §8.2 lernie family: the composer's own three.
        "message" => Ok(act(Action::Message {
            workspace: args::workspace(ctx, verb)?,
            agent: args::agent(ctx, verb)?,
            // Verbatim (§3.3, bl-6920): the tail *is* the content, spacing and
            // newlines included — only the line's own ends are trimmed.
            content: args::required(tail, verb, "text to send")?,
        })),
        "stop" => Ok(act(Action::Stop {
            workspace: args::workspace(ctx, verb)?,
            agent: args::agent(ctx, verb)?,
            children: children(tail)?,
        })),
        "scan" => {
            args::none(tail, verb)?;
            Ok(act(Action::Scan {
                workspace: args::workspace(ctx, verb)?,
            }))
        }
        // Fire inference from where the conversation stands (bl-9bef). It takes
        // no words at all: what it says is *what is already there*, so a tail
        // would be a message, and a message is `/message`.
        "nudge" => {
            args::none(tail, verb)?;
            Ok(act(Action::Nudge {
                workspace: args::workspace(ctx, verb)?,
                agent: args::agent(ctx, verb)?,
            }))
        }
        // The §9.4 exit from the config freeze (bl-2d19): the conversation is
        // the seat's, the lineage is the workspace's one default, so the verb
        // is the whole line.
        "retarget" => verbs::retarget(tail, ctx, verb),
        // The §8.2 `bl` family: an id typed, or the focused ball's.
        "close" => Ok(act(Action::Close {
            project: args::project(ctx, verb)?,
            id: id(tail, ctx, verb)?,
            name: args::name(ctx, verb)?,
        })),
        "assign" => Ok(act(Action::Assign {
            project: args::project(ctx, verb)?,
            id: id(tail, ctx, verb)?,
            name: args::name(ctx, verb)?,
        })),
        "release" => Ok(act(Action::Release {
            project: args::project(ctx, verb)?,
            id: id(tail, ctx, verb)?,
            name: args::name(ctx, verb)?,
        })),
        "move" => moved(tail, ctx, verb),
        "create" => create(tail, ctx, verb),
        "update" => update(tail, ctx, verb),
        // The §8.1 start family, as its two real gestures.
        "prepare" => Ok(act(Action::Prepare {
            workspace: args::workspace(ctx, verb)?,
            payload: payload(tail, ctx, verb)?,
        })),
        "prompt" => Ok(act(Action::Prompt {
            prepared: ctx.prepared.clone().ok_or_else(|| {
                format!("/{verb}: nothing is prepared — /prepare first, then say the goal")
            })?,
            goal: args::required(tail, verb, "the goal")?,
        })),
        // The §3.6 unmaking: the typed name is the arming, and the gate is
        // dispatch's, not the reader's — fail-closed wherever it fires.
        "delete-workspace" => Ok(act(Action::DeleteWorkspace {
            workspace: args::workspace(ctx, verb)?,
            typed: args::required(tail, verb, "the workspace name, typed out")?,
        })),
        // The §3.6 class one conversation deep (bl-f17a): a bare line is the
        // leaf delete; the typed name is what arms `--children`. The gate is
        // dispatch's, not the reader's.
        "delete-agent" => Ok(act(Action::DeleteAgent {
            workspace: args::workspace(ctx, verb)?,
            agent: args::agent(ctx, verb)?,
            typed: tail.trim().to_owned(),
        })),
        // The VISION §4.9 monitor's three, read in `verbs` like every other tail.
        "arm" | "disarm" | "flag" => verbs::monitor(verb, tail, ctx),
        // The VISION §4.3 armed loop's two, read the same way.
        crate::boundary::codec::FLEET_ARM | crate::boundary::codec::FLEET_DISARM => {
            verbs::fleet(verb, tail, ctx)
        }
        // The VISION §4.11 capability answer: the conversation is the seat's,
        // as it is for `/message` and `/stop`, and the held `tool_use` id is
        // derived at fire time — so the whole line is one word, the verdict.
        "answer" => verbs::answer(tail, ctx, verb),
        // The VISION §4.9 fifth rung, written into that same fold: the floor
        // under which everything above a read waits for an answer. The
        // conversation is the seat's, so the verb is the whole line.
        "revoke" | "restore" => verbs::floor(verb, tail, ctx),
        // The §9 config family (bl-3f46): each destination's own words, then
        // the file's text verbatim — the grammar lives beside its writer.
        "config" => config::config(tail, ctx, verb),
        "marks" => config::marks(tail, ctx, verb),
        "model" => config::model(tail, ctx, verb),
        "fork" => super::fork::fork(tail, ctx, verb),
        "ack" => args::none(tail, verb).map(|()| act(Action::Ack)),
        "seen" => verbs::seen(tail, ctx, verb),
        "clear-trail" => args::none(tail, verb).map(|()| act(Action::ClearTrail)),
        // The queries (§8.5): populating reads, spellable exactly as actions
        // are — a seat with no panes still has to be able to look. Split out
        // at the §12 line budget; an unknown verb refuses there too, so this
        // stays the whole grammar's one dead end.
        other => queries(other, tail, ctx),
    }
}

/// The populating reads' half of the grammar (§8.5) — a seat with no panes
/// still has to be able to look. `verb` has already failed every mutating
/// arm, so an unknown one here is unknown outright.
fn queries(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    match verb {
        "workspaces" => args::none(tail, verb).map(|()| ask(Query::Workspaces)),
        "conversations" => {
            args::none(tail, verb)?;
            Ok(ask(Query::Conversations {
                workspace: args::workspace(ctx, verb)?,
            }))
        }
        "balls" => args::none(tail, verb).map(|()| ask(Query::Balls)),
        // Bare, the listing; with two words, one file's patch out of the named
        // ball's diff. A path alone would not say which attempt it belongs to
        // when the workspace holds more than one, so both words or neither.
        "work-diff" => Ok(ask(Query::WorkDiff {
            workspace: args::workspace(ctx, verb)?,
            file: work_file(tail, verb)?,
        })),
        "board" => args::none(tail, verb).map(|()| ask(Query::Board)),
        // brazen's provider table + credential presence (§8.5, bl-0164): the
        // §8.3 login pane's `↻`, spelled. Scoped to a workspace, not global
        // (bl-fcd5): sign-ins live inside a wall, so the seat's own `--ws` is
        // what makes the credential column mean anything.
        "providers" => {
            args::none(tail, verb)?;
            Ok(ask(Query::Providers {
                workspace: args::workspace(ctx, verb)?,
            }))
        }
        // The §9.3 browse (bl-dff8): the lineages of the seat's own workspace
        // and the files each tip holds — what `/config branch <lineage> <path>`
        // then reads a file out of. Scoped by the seat like `/providers`.
        "lineages" => {
            args::none(tail, verb)?;
            Ok(ask(Query::Lineages {
                workspace: args::workspace(ctx, verb)?,
            }))
        }
        // The §9.4 roster (bl-dff8): one word, and it is the provider row —
        // the picker asks per row, and a roster with no row named is not a
        // question. The wall it is asked in is the seat's, as `/providers`' is.
        "models" => Ok(ask(Query::Models {
            workspace: args::workspace(ctx, verb)?,
            provider: match args::optional_word(tail, verb)? {
                Some(provider) => provider,
                None => return Err(format!("/{verb}: usage: /models <provider>")),
            },
        })),
        "attention" => args::none(tail, verb).map(|()| ask(Query::Attention)),
        "ops" => Ok(ask(Query::Ops { max: max(tail)? })),
        // The whole tail is the needle — no flags, no bound. Search takes one
        // parameter and it is the text, so there is nothing for a grammar to
        // split off; how deep the answer goes is `search::MAX`, not a knob.
        // An empty tail is **not** a refusal: an empty query matches nothing
        // (the general path with no input), which is how a seat clears the last
        // answer without a second verb to do it with.
        "search" => Ok(ask(Query::Search {
            text: tail.trim().to_owned(),
        })),
        other => Err(format!("unknown command /{other}\n{}", help::roster())),
    }
}

/// The higher-order rule (§8.5): is this line asking what a command does?
///
/// `Some` when it is — `<verb> --help`, `/help`, `/help <verb>`, or the bare
/// `/` that names nothing. `None` when it is not, and the verb's own grammar
/// runs. The flag form is recognized **only when the tail is exactly the
/// flag**, which is what keeps the two verbatim payloads whole: `/message
/// --help` asks about `message`, and `/message run --help on it` is a message
/// that happens to mention one.
fn asks_help(verb: &str, tail: &str) -> Result<Option<Query>, String> {
    if matches!(tail.trim(), "--help" | "-h") && help::known(verb) {
        return Ok(Some(Query::Help {
            verb: Some(verb.to_owned()),
        }));
    }
    if !(verb.is_empty() || verb == "help") {
        return Ok(None);
    }
    match args::optional_word(tail, "help")? {
        None => Ok(Some(Query::Help { verb: None })),
        Some(about) if help::known(&about) => Ok(Some(Query::Help { verb: Some(about) })),
        Some(about) => Err(format!("unknown command /{about}\n{}", help::roster())),
    }
}

fn act(action: Action) -> Gesture {
    Gesture::Act(action)
}

fn ask(query: Query) -> Gesture {
    Gesture::Ask(query)
}
