//! The line writer (§8.5): a [`Gesture`] back to the line that spells it —
//! what a seat echoes after a click ("you just typed `/close bl-1f2a`"), what a
//! teleoperator logs, and, above all, **the compile gate**.
//!
//! The match below is exhaustive over [`Gesture`], so a variant added to the
//! boundary does not build until it can be typed. That is the same law the
//! codec applies to the envelope, and it is why the third serialization cannot
//! silently fall behind the other two.
//!
//! What a line elides, this writes elided: a message's workspace and agent are
//! the seat's selection, not the gesture's spelling. So the round trip holds
//! **modulo context** — `parse(spell(g), ctx_of(g)) == g` — which is exactly
//! the parity claim the line makes, and the tests read it that way.

use crate::boundary::{Action, Gesture, Query};
use crate::start::{BallSpec, Payload};

/// Spell one gesture as its line (§8.5). Total over the surface.
pub fn spell(gesture: &Gesture) -> String {
    match gesture {
        Gesture::Act(action) => spell_action(action),
        Gesture::Ask(query) => spell_query(query),
    }
}

fn spell_action(action: &Action) -> String {
    match action {
        Action::Message { content, .. } => format!("/message {content}"),
        // The tail is the text, verbatim, exactly as `/message`'s is: the same
        // deposit, with the stop ahead of it (bl-a33d).
        Action::Interrupt { content, .. } => format!("/interrupt {content}"),
        Action::Stop { children, .. } => match children {
            true => "/stop children".to_owned(),
            false => "/stop".to_owned(),
        },
        Action::Scan { .. } => "/scan".to_owned(),
        // The conversation is the seat's selection and the gesture carries no
        // payload, so the verb is the whole line.
        Action::Nudge { .. } => "/nudge".to_owned(),
        // The conversation is the seat's selection, as `/message`'s is, and the
        // lineage is the workspace's one default — so the verb is the line.
        Action::Retarget { .. } => "/retarget".to_owned(),
        Action::Close { id, .. } => format!("/close {id}"),
        Action::Assign { id, .. } => format!("/assign {id}"),
        Action::Release { id, .. } => format!("/release {id}"),
        Action::Create { fields, .. } => format!(
            "/create {}{}{}",
            fields.title,
            flag("body", fields.body.as_ref()),
            super::balls::spell_fields(&fields.fields)
        ),
        Action::Update { id, fields, .. } => format!(
            "/update {id}{}{}{}{}",
            flag("title", fields.title.as_ref()),
            flag("body", fields.body.as_ref()),
            flag("note", fields.note.as_ref()),
            super::balls::spell_fields(&fields.fields)
        ),
        Action::Prepare { payload, .. } => spell_payload(payload),
        Action::Prompt { goal, .. } => format!("/prompt {goal}"),
        // N is the whole line: the obligation and the prepared start are the
        // seat's, exactly as `/prompt`'s prepared is.
        Action::Fan(crate::fan::Verb::Spread { n, .. }) => format!("/fan {n}"),
        Action::Fan(crate::fan::Verb::Retire { handle, .. }) => format!("/retire {handle}"),
        Action::Fan(crate::fan::Verb::Deliver {
            handle, summary, ..
        }) => {
            format!("/deliver {handle} {summary}")
        }
        Action::DeleteWorkspace { typed, .. } => format!("/delete-workspace {typed}"),
        Action::DeleteAgent { typed, .. } if typed.is_empty() => "/delete-agent".to_owned(),
        Action::DeleteAgent { typed, .. } => format!("/delete-agent {typed}"),
        Action::Monitor(verb) => spell_monitor(verb),
        Action::Fleet(verb) => spell_fleet(verb),
        // The conversation is the seat's selection, exactly as `/seen`'s is;
        // the held id is derived, so the verdict is the whole line.
        Action::AnswerHold { ruling, .. } => format!("/answer {}", ruling.word()),
        // Same address, same elision; the direction is the verb.
        Action::Floor { raised, .. } => match raised {
            true => "/revoke".to_owned(),
            false => "/restore".to_owned(),
        },
        Action::ApplyConfig { file, text } => {
            format!("/config {} {text}", super::config::target_words(file))
        }
        Action::SetMarks { branch, .. } => format!("/marks {branch}"),
        Action::PickModel {
            role,
            provider,
            model,
            ..
        } => format!("/model {role} {provider} {model}"),
        Action::Fork { attempt, goal, .. } => super::fork::spell(attempt, goal),
        Action::Ack => "/ack".to_owned(),
        // The address is the seat's selection, exactly as `/message`'s is.
        Action::MarkSeen { .. } => "/seen".to_owned(),
        Action::ClearTrail => "/clear-trail".to_owned(),
        // The set is the whole line, in its one spelling (REMOTE §5, bl-4e08);
        // the client is the intake's and is never typed.
        Action::Advertise { tools } => {
            format!("/advertise {}", crate::registry::tools::encode(tools))
        }
        Action::Route(verb) => spell_route(verb),
    }
}

/// The routing leg's two (bl-024b): each states what no seat can supply — the
/// addressee and the model's arguments, the handle and the bytes that came
/// back — and both end in their document, verbatim.
fn spell_route(verb: &crate::registry::mailbox::Verb) -> String {
    use crate::registry::mailbox::{Verb, capture_value};
    match verb {
        Verb::Invoke(call) => format!("/invoke {} {} {}", call.client, call.tool, call.input),
        Verb::Complete(done) => format!(
            "/complete {} {}",
            done.invocation,
            capture_value(&done.capture)
        ),
    }
}

/// The monitor family's three (VISION §4.9). Each spells as its own verb: the
/// family is one variant at the boundary, never one word at the keyboard.
fn spell_monitor(verb: &crate::monitor::Verb) -> String {
    use crate::monitor::Verb;
    match verb {
        Verb::Arm { model, .. } => format!("/arm {model}"),
        Verb::Disarm { .. } => "/disarm".to_owned(),
        Verb::Flag { reason, .. } => format!("/flag {reason}"),
    }
}

/// The armed loop's two (VISION §4.3). The project and the workspace are the
/// seat's selection, exactly as they are for every `bl` verb, so the cap is the
/// whole line.
fn spell_fleet(verb: &crate::fleet::Verb) -> String {
    use crate::fleet::Verb;
    match verb {
        Verb::Arm { cap, .. } => format!("/{} {cap}", crate::boundary::codec::FLEET_ARM),
        Verb::Disarm { .. } => format!("/{}", crate::boundary::codec::FLEET_DISARM),
    }
}

fn spell_query(query: &Query) -> String {
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
        Query::ReadConfig { file } => format!("/config {}", super::config::target_words(file)),
        Query::Marks { .. } => "/marks".to_owned(),
        // The workspace rides the seat's own context (`--ws`), exactly as
        // `/marks` and `/conversations` spell theirs — a line states its
        // targets through the context flags, never twice.
        Query::Providers { .. } => "/providers".to_owned(),
        // Same elision, same reason: the workspace is the seat's, and what the
        // line states is the one thing it cannot supply — the provider row.
        Query::Lineages { .. } => "/lineages".to_owned(),
        Query::Models { provider, .. } => format!("/models {provider}"),
        // The workspace is the seat's, as `/providers`' is.
        Query::Clients { .. } => "/clients".to_owned(),
        // The follow-class read names nothing at all; its sibling names the
        // handle, which is the one thing a seat cannot hold.
        Query::Invocations => "/invocations".to_owned(),
        Query::Capture { invocation } => format!("/capture {invocation}"),
    }
}

/// The §3.4 rung, said outright — the reader never infers it, so neither does
/// the writer.
fn spell_payload(payload: &Payload) -> String {
    match payload {
        Payload::Bare => "/prepare".to_owned(),
        Payload::Path { dir } => format!("/prepare dir {}", dir.display()),
        Payload::Ball { ball, .. } => match ball {
            // An existing ball is the seat's selection (its title, body and
            // §3.5 join are roster facts no line states), so it spells as the
            // rung alone; a new one is named by the line that mints it.
            BallSpec::Existing { .. } => "/prepare ball".to_owned(),
            BallSpec::New { title, body } => format!(
                "/prepare ball --new {title}{}",
                flag("body", Some(body).filter(|b| !b.is_empty()))
            ),
        },
    }
}

/// An optional field as its flag, or nothing at all.
fn flag(key: &str, value: Option<&String>) -> String {
    match value {
        Some(text) => format!(" --{key} {text}"),
        None => String::new(),
    }
}
