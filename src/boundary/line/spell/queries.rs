//! The line writer's **query half** (§8.5) — a populating read back to the
//! line that spells it.
//!
//! Split from [`super`] at §12's cap on the boundary's own taxonomy, the seam
//! `action`/`query` are already cut on and the one the help table follows
//! (bl-719a): actions mutate the world and are spelled there, queries populate
//! and are spelled here. The match below is exhaustive over [`Query`], so a
//! read added to the boundary does not build until it can be typed — the same
//! compile gate the action half keeps.

use crate::boundary::Query;
use crate::boundary::config::Read;

/// The §9 config family's five, beside [`spell_ball`] and for its reason: the
/// roster names the family once and the members are spelled where they live
/// (bl-719a). Each still spells as its own slash verb — the fold is in the
/// carrier, never in the surface.
///
/// Four of the five elide their workspace, which rides the seat's own context
/// (`--ws`) exactly as `/marks` and `/conversations` do: a line states its
/// targets through the context flags, never twice. What a line does state is
/// the thing the context cannot supply — a destination, or a provider row.
fn spell_config(read: &Read) -> String {
    match read {
        Read::File { file } => format!(
            "/config {}",
            crate::boundary::line::config::target_words(file)
        ),
        Read::Marks { .. } => "/marks".to_owned(),
        Read::Providers { .. } => "/providers".to_owned(),
        Read::Roles { .. } => "/roles".to_owned(),
        Read::Lineages { .. } => "/lineages".to_owned(),
        Read::Models { provider, .. } => format!("/models {provider}"),
    }
}

pub(super) fn spell_query(query: &Query) -> String {
    match query {
        Query::Workspaces => "/workspaces".to_owned(),
        Query::Conversations { .. } => "/conversations".to_owned(),
        Query::Balls => "/balls".to_owned(),
        Query::WorkspaceBalls { .. } => "/workspace-balls".to_owned(),
        // The workspace is the seat's, as it is for every other workspace-
        // scoped line; the file, when one is asked for, is not.
        Query::WorkDiff { file, .. } => match file {
            Some(file) => match &file.handle {
                Some(handle) => format!("/work-diff {} {handle} {}", file.ball, file.path),
                None => format!("/work-diff {} {}", file.ball, file.path),
            },
            None => "/work-diff".to_owned(),
        },
        // The projection takes no parameter at all (§3.9): its subject is every
        // attempt the seat's own workspace holds.
        Query::Science { .. } => "/science".to_owned(),
        // The §11 inspector family (bl-6233): the workspace *and* the
        // conversation are the seat's selection, exactly as `/message`'s are —
        // so most of them are the verb alone, and the rest state only the
        // thing no seat can supply (which step, which file, which commit).
        Query::Transcript { .. } => "/transcript".to_owned(),
        Query::Follow { .. } => "/follow".to_owned(),
        Query::Steps { .. } => "/steps".to_owned(),
        Query::Step { seq, .. } => format!("/step {seq}"),
        Query::Files { path, at, .. } => {
            let pinned = at
                .as_ref()
                .map(|c| format!(" --at {c}"))
                .unwrap_or_default();
            match path {
                Some(path) => format!("/files {path}{pinned}"),
                None => format!("/files{pinned}"),
            }
        }
        // Config-frozen-at (bl-13f9): the `/files` elision, one flag wide —
        // the commit is the only thing the seat's selection cannot supply.
        Query::Governing { at, .. } => match at {
            Some(commit) => format!("/governing --at {commit}"),
            None => "/governing".to_owned(),
        },
        Query::Rail { .. } => "/rail".to_owned(),
        Query::Agent { .. } => "/agent".to_owned(),
        Query::Inbox { .. } => "/inbox".to_owned(),
        Query::Board => "/board".to_owned(),
        Query::Attention => "/attention".to_owned(),
        Query::Ops { max } => format!("/ops {max}"),
        Query::Search { text } => format!("/search {text}"),
        // Help spells as itself, never as the `--help` that also asks it: one
        // gesture has one canonical line, and the flag is the *other* way to
        // reach it (§8.5).
        Query::Help { verb } => match verb {
            Some(verb) => format!("/help {verb}"),
            None => "/help".to_owned(),
        },
        // The read half of the config family (§8.5, bl-0164): the same verb
        // as its write, spelled with nothing after the destination — the
        // grammar `config::config` reads that shape back as a read.
        Query::Config(read) => spell_config(read),
        // The workspace is the seat's, as `/providers`' is.
        Query::Clients { .. } => "/clients".to_owned(),
        // The follow-class read names nothing at all; its sibling names the
        // handle, which is the one thing a seat cannot hold.
        Query::Invocations => "/invocations".to_owned(),
        Query::Capture { invocation } => format!("/capture {invocation}"),
    }
}
