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

use crate::actions::verbs::Verb as BallVerb;
use crate::boundary::{Action, Gesture};

mod queries;
use crate::start::{BallSpec, Payload};
use queries::spell_query;

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
        Action::Ball(verb) => spell_ball(verb),
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
        Action::Tune(tuning) => spell_tuning(tuning),
        // The §8.3 sign-in: the wall is the seat's, as `/model`'s is, and the
        // row is the one word the context cannot supply.
        Action::Login { provider, .. } => {
            format!("/{} {provider}", crate::boundary::codec::LOGIN)
        }
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
        // REMOTE §1.4's enrollment (bl-f4e3). The workspace is the seat's, as
        // `/marks`' is; the name is stated because nothing on this side holds
        // it; and operator grade spells **bare**, because that is what
        // default-operator means at a keyboard (§4.2).
        Action::Enroll(request) => match request.grade {
            crate::registry::Grade::Operator => {
                format!("/{} {}", crate::boundary::codec::ENROLL, request.name)
            }
            grade @ crate::registry::Grade::Foot => format!(
                "/{} {} {}",
                crate::boundary::codec::ENROLL,
                request.name,
                grade.word()
            ),
        },
        Action::Route(verb) => spell_route(verb),
    }
}

/// The routing leg's two (bl-024b): each states what no seat can supply — the
/// addressee and the model's arguments, the handle and the bytes that came
/// back — and both end in their document, verbatim.
fn spell_route(verb: &crate::registry::mailbox::Verb) -> String {
    use crate::registry::mailbox::{Verb, capture_value};
    match verb {
        Verb::Invoke(call) => match &call.cwd {
            // The subject's location rides as a word between the tool and the
            // document (bl-77be) — the one line spelling, mirrored by the
            // reader. A cwd containing whitespace cannot ride the line face;
            // the JSON serializations carry it whole.
            Some(cwd) => format!(
                "/invoke {} {} --cwd {} {}",
                call.client, call.tool, cwd, call.input
            ),
            None => format!("/invoke {} {} {}", call.client, call.tool, call.input),
        },
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
/// The §8.2 `bl` family's five, beside [`spell_fleet`] and for its reason: the
/// roster names the family once and the members are spelled where they live
/// (bl-92d3). Each still spells as its own slash verb — the fold is in the
/// carrier, never in the surface.
fn spell_ball(verb: &BallVerb) -> String {
    match verb {
        BallVerb::Close { id, .. } => format!("/close {id}"),
        BallVerb::Assign { id, .. } => format!("/assign {id}"),
        BallVerb::Release { id, .. } => format!("/release {id}"),
        BallVerb::Create { fields, .. } => format!(
            "/create {}{}{}",
            fields.title,
            flag("body", fields.body.as_ref()),
            super::balls::spell_fields(&fields.fields)
        ),
        BallVerb::Update { id, fields, .. } => format!(
            "/update {id}{}{}{}{}",
            flag("title", fields.title.as_ref()),
            flag("body", fields.body.as_ref()),
            flag("note", fields.note.as_ref()),
            super::balls::spell_fields(&fields.fields)
        ),
    }
}

/// The §9.4 tuning pair (bl-23bd) — each member as its own line, `off` for
/// both absences, which is exactly the word the reader takes back.
fn spell_tuning(tuning: &crate::model_pick::Tuning) -> String {
    use crate::model_pick::Tuning;
    match tuning {
        Tuning::Effort { role, level, .. } => {
            let word = level.map_or_else(|| "off".to_owned(), |l| l.as_str());
            format!("/effort {role} {word}")
        }
        Tuning::Priority { role, on, .. } => {
            format!("/priority {role} {}", if *on { "on" } else { "off" })
        }
    }
}

fn spell_fleet(verb: &crate::fleet::Verb) -> String {
    use crate::fleet::Verb;
    match verb {
        Verb::Arm { cap, .. } => format!("/{} {cap}", crate::boundary::codec::FLEET_ARM),
        Verb::Disarm { .. } => format!("/{}", crate::boundary::codec::FLEET_DISARM),
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
