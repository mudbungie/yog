//! The populating reads' half of the line grammar (§8.5) — a seat with no
//! panes still has to be able to look. Split from [`super::parse`] at §12's
//! per-file budget (bl-6233) along the §4.8 taxonomy's own line, exactly as
//! the codec and the help table are already cut: actions mutate, queries
//! populate.
//!
//! A verb reaching here has already failed every mutating arm, so an unknown
//! one is unknown outright — this stays the whole grammar's one dead end.

use super::parse::ask;
use super::verbs::{max, work_file};
use super::{Context, args};
use crate::boundary::{Gesture, Query, help};

/// Read one populating verb's line, or refuse with the roster attached.
pub(super) fn queries(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    // The conversation-addressed family reads first, in its own table — the
    // seam `codec::query::inspector` already draws, and for the same reason: a
    // verb it does not claim falls through to this one unchanged.
    if let Some(gesture) = conversation(verb, tail, ctx) {
        return gesture;
    }
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
        // REMOTE §5's roster (bl-4e08): who is registered in the seat's own
        // workspace, who is live, and what each advertises. Scoped by the seat
        // exactly as `/providers` is.
        "clients" => {
            args::none(tail, verb)?;
            Ok(ask(Query::Clients {
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
        // REMOTE §3's follow-class read and the poll beside it (bl-024b). The
        // first names nothing: the queue it drains is the intake's own, so a
        // line seat typing it drains nothing and is told why at dispatch.
        "invocations" => args::none(tail, verb).map(|()| ask(Query::Invocations)),
        "capture" => match args::optional_word(tail, verb)? {
            Some(invocation) => Ok(ask(Query::Capture { invocation })),
            None => Err(format!("/{verb}: usage: /{verb} <invocation>")),
        },
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

/// The §11 inspector family's lines (bl-6233, REMOTE §9 step 1; extended
/// bl-13f9): the conversation's own reads. `None` when `verb` names none of
/// them — the signal [`queries`] chains on before its own table.
///
/// Every one is aimed by the seat — the workspace *and* the selected
/// conversation, exactly as `/message` is aimed — because a line states its
/// targets through the context, never twice. What each states is the one thing
/// no seat can supply.
fn conversation(verb: &str, tail: &str, ctx: &Context) -> Option<Result<Gesture, String>> {
    Some(match verb {
        "transcript" => bare(verb, tail, ctx, &|workspace, agent| Query::Transcript {
            workspace,
            agent,
        }),
        "steps" => bare(verb, tail, ctx, &|workspace, agent| Query::Steps {
            workspace,
            agent,
        }),
        "rail" => bare(verb, tail, ctx, &|workspace, agent| Query::Rail {
            workspace,
            agent,
        }),
        "inbox" => bare(verb, tail, ctx, &|workspace, agent| Query::Inbox {
            workspace,
            agent,
        }),
        "agent" => bare(verb, tail, ctx, &|workspace, agent| Query::Agent {
            workspace,
            agent,
        }),
        // A step is picked by its sequence name (`001`), as the Steps list
        // spells it. No default: "some step" is not a question.
        "step" => step(verb, tail, ctx),
        // Bare, the listing; with one word, that listed file's bytes — the
        // `/work-diff` shape, and for the same reason: a listing and one
        // entry's content are one question at two depths.
        "files" => files(verb, tail, ctx),
        // Config-frozen-at (VISION V1.2, bl-13f9): the `/files` shape at the
        // other tab whose subject is a commit's tree. Bare it is the
        // conversation's own tip, so a seat that has pinned nothing types the
        // one word the window paints unpinned.
        "governing" => governing(verb, tail, ctx),
        _ => return None,
    })
}

/// The five that take no argument at all: the address, and nothing else.
fn bare(
    verb: &str,
    tail: &str,
    ctx: &Context,
    build: &dyn Fn(String, String) -> Query,
) -> Result<Gesture, String> {
    args::none(tail, verb)?;
    let (workspace, agent) = at(verb, ctx)?;
    Ok(ask(build(workspace, agent)))
}

fn step(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    let Some(seq) = args::optional_word(tail, verb)? else {
        return Err(format!("/{verb}: usage: /step <seq>"));
    };
    let (workspace, agent) = at(verb, ctx)?;
    Ok(ask(Query::Step {
        workspace,
        agent,
        seq,
    }))
}

fn files(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    let (positional, flags) = args::split_flags(tail);
    args::only(&flags, &["at"], verb)?;
    let path = args::optional_word(&positional, verb)?;
    let (workspace, agent) = at(verb, ctx)?;
    Ok(ask(Query::Files {
        workspace,
        agent,
        path,
        at: args::flag(&flags, "at", verb)?,
    }))
}

fn governing(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    let (positional, flags) = args::split_flags(tail);
    args::none(&positional, verb)?;
    args::only(&flags, &["at"], verb)?;
    let (workspace, agent) = at(verb, ctx)?;
    Ok(ask(Query::Governing {
        workspace,
        agent,
        at: args::flag(&flags, "at", verb)?,
    }))
}

/// The §11 family's shared address: the seat's workspace and the conversation
/// selected in it, each refused by name when the seat has none — exactly as
/// `/message`'s address is, and for the same reason. A read that guessed a
/// conversation would answer about a different chat.
fn at(verb: &str, ctx: &Context) -> Result<(String, String), String> {
    Ok((args::workspace(ctx, verb)?, args::agent(ctx, verb)?))
}
