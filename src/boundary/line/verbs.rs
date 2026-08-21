//! The line's **per-verb** argument builders (§8.5) — one small function per
//! grammar that is more than a bare tail, split out of [`super::parse`] at
//! §12's per-file budget. The seam is real: [`super::parse`] holds the verb
//! table and the higher-order help rule, this holds what each verb makes of
//! its words, and [`super::args`] the total helpers they all share.

use super::{Context, args};
use crate::boundary::{Action, Gesture};
use crate::monitor::Verb;
use crate::start::{BallSpec, Payload};
use std::path::PathBuf;

/// `/seen` — the §6 queue's answer. The seat's own selection is the item,
/// exactly as it is for `/message`: answering and acknowledging aim alike, so
/// the line names neither and the context supplies both.
pub(super) fn seen(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    args::none(tail, verb)?;
    Ok(act(Action::MarkSeen {
        workspace: args::workspace(ctx, verb)?,
        agent: args::agent(ctx, verb)?,
    }))
}

/// `/retarget` — the §9.4 exit from the config freeze (bl-2d19). It names
/// nothing at all: the conversation is the seat's, exactly as `/seen`'s is, and
/// the config it moves onto is the workspace's one lineage head (§9.3), which
/// is the only thing the drift beside it is measured against.
pub(super) fn retarget(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    args::none(tail, verb)?;
    Ok(act(Action::Retarget {
        workspace: args::workspace(ctx, verb)?,
        agent: args::agent(ctx, verb)?,
    }))
}

/// `/prompt <goal…>` — the §8.1 deferred fire, against whatever this seat has
/// prepared. **It carries no §3.3 seed** (bl-1747): a seed is the firing
/// seat's own prediction and this reader paints none, so the door draws off
/// the stamp — the window's line, which *does* have a preview above it, fills
/// the seed in after the read.
pub(super) fn prompt(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    Ok(act(Action::Prompt {
        prepared: ctx.prepared.clone().ok_or_else(|| {
            format!("/{verb}: nothing is prepared — /prepare first, then say the goal")
        })?,
        goal: args::required(tail, verb, "the goal")?,
        seed: None,
    }))
}

/// `/answer pass|hold|refuse` — the §4.11 capability answer. The conversation
/// is the seat's, like `/seen`'s; the held `tool_use` id is derived at fire
/// time, so the only word a line can carry is the verdict — and it is
/// required, because an answer with a default verdict would be yog deciding.
pub(super) fn answer(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    let word = args::required(tail, verb, "pass, hold or refuse")?;
    let ruling = crate::control::judge::Ruling::of(word.trim())
        .ok_or_else(|| format!("/{verb}: unknown verdict {word:?}; usage: {ANSWER_USAGE}"))?;
    Ok(act(Action::AnswerHold {
        workspace: args::workspace(ctx, verb)?,
        agent: args::agent(ctx, verb)?,
        ruling,
    }))
}

/// `/answer`'s usage, said once — the refusal above and the help page below
/// read this one string.
pub const ANSWER_USAGE: &str = "/answer pass | hold | refuse";

/// `/revoke` and `/restore` — VISION §4.9's fifth rung over the §4.11 fold.
/// Neither names anything but itself: the conversation is the seat's own, as
/// `/answer`'s and `/flag`'s are, and the direction is the verb rather than a
/// word after it, because raising and lowering are two instructions and a
/// gesture is never read out of an absence.
pub(super) fn floor(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    args::none(tail, verb)?;
    Ok(act(Action::Floor {
        workspace: args::workspace(ctx, verb)?,
        agent: args::agent(ctx, verb)?,
        raised: verb == "revoke",
    }))
}

/// `/ops` with no count: the tail an operator means by "what just happened".
const OPS_DEFAULT: usize = 50;

fn act(action: Action) -> Gesture {
    Gesture::Act(action)
}

/// `/stop [children]` — the cascade is a word, not a flag: it is the checkbox
/// beside the button, and a checkbox spells as the thing being checked.
pub(super) fn children(tail: &str) -> Result<bool, String> {
    match tail.trim() {
        "" => Ok(false),
        "children" => Ok(true),
        other => Err(format!(
            "/stop: unexpected {other:?}; usage: /stop [children]"
        )),
    }
}

/// The §3.4 payload rung, said outright: nothing is the bare rung, `dir <path>`
/// the path rung, `ball` the focused ball (or `--new <title…>` for one this
/// line mints). The rung is never inferred from what happens to be selected —
/// a start that silently picked a different rung is the one mistake this whole
/// grammar exists to make impossible.
pub(super) fn payload(tail: &str, ctx: &Context, verb: &str) -> Result<Payload, String> {
    let (rung, rest) = args::first_word(tail);
    match rung.as_str() {
        "" => Ok(Payload::Bare),
        "dir" => Ok(Payload::Path {
            dir: PathBuf::from(args::required(&rest, verb, "a work directory")?),
        }),
        "ball" => Ok(Payload::Ball {
            project: args::project(ctx, verb)?,
            ball: spec(&rest, ctx, verb)?,
        }),
        other => Err(format!(
            "/{verb}: unknown rung {other:?}; usage: /prepare | /prepare dir <path> | /prepare ball [--new <title…>]"
        )),
    }
}

/// The ball a `/prepare ball` starts from: the focused one, or a new one this
/// line names. An **existing** ball carries roster facts (title, body, its §3.5
/// join) that no line can state, so it comes from the seat's selection or not
/// at all — a seat holding no roster spells this gesture as an envelope.
fn spec(rest: &str, ctx: &Context, verb: &str) -> Result<BallSpec, String> {
    let (positional, flags) = args::split_flags(rest);
    args::only(&flags, &["new", "body"], verb)?;
    args::none(&positional, verb)?;
    match args::flag(&flags, "new", verb)? {
        Some(title) => Ok(BallSpec::New {
            title,
            body: args::flag(&flags, "body", verb)?.unwrap_or_default(),
        }),
        None => ctx.ball.clone().ok_or_else(|| {
            format!("/{verb}: no ball is selected — select one, or say --new <title…>")
        }),
    }
}

/// `/work-diff [<ball> [<handle>] <path>]` — the file a patch read names, or
/// nothing at all for the listing. Three words name a fan candidate's file
/// (bl-c2bd): its cohort's candidates all carry the obligation's ball, so only
/// the handle says whose diff the path belongs to.
pub(super) fn work_file(
    tail: &str,
    verb: &str,
) -> Result<Option<crate::workdiff::WorkFile>, String> {
    match tail.split_whitespace().collect::<Vec<_>>().as_slice() {
        [] => Ok(None),
        [ball, path] => Ok(Some(crate::workdiff::WorkFile {
            ball: (*ball).to_owned(),
            handle: None,
            path: (*path).to_owned(),
        })),
        [ball, handle, path] => Ok(Some(crate::workdiff::WorkFile {
            ball: (*ball).to_owned(),
            handle: Some((*handle).to_owned()),
            path: (*path).to_owned(),
        })),
        _ => Err(format!(
            "/{verb}: usage: /work-diff [<ball> [<handle>] <path>]"
        )),
    }
}

/// `/ops [n]` — how deep to read the trail.
pub(super) fn max(tail: &str) -> Result<usize, String> {
    match args::optional_word(tail, "ops")? {
        None => Ok(OPS_DEFAULT),
        Some(word) => word
            .parse()
            .map_err(|_| format!("/ops: {word:?} is not a row count")),
    }
}

/// The VISION §4.9 monitor's three, read as one family (they are one boundary
/// variant). `/arm <model>` — the model pin is arming's whole parameter, with
/// no default: it decides what every check costs, and guessing it would spend
/// the operator's money on yog's opinion. `/disarm` names nothing at all — its
/// own verb rather than an arm with no pin, because it is its own instruction.
/// `/flag <why…>` requires its reason: an attention item with nothing behind it
/// is a light the operator reading the trail tomorrow pays for.
pub(super) fn monitor(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    let workspace = args::workspace(ctx, verb)?;
    let gesture = match verb {
        "arm" => Verb::Arm {
            workspace,
            model: args::required(tail, verb, "the model to check with")?,
        },
        "disarm" => {
            args::none(tail, verb)?;
            Verb::Disarm { workspace }
        }
        _ => Verb::Flag {
            workspace,
            agent: args::agent(ctx, verb)?,
            reason: args::required(tail, verb, "why")?,
        },
    };
    Ok(act(Action::Monitor(gesture)))
}

/// The VISION §4.3 loop's two, read as one family (they are one boundary
/// variant). `/fleet <cap>` — the cap is arming's whole typed parameter, with
/// no default: it decides how many drones this workspace may run at once, and
/// guessing it would spend the operator's money on yog's opinion. The project
/// it takes work from is the seat's own, like every other `bl`-family target.
/// `/disband` names nothing at all — its own verb rather than a `/fleet 0`,
/// because a zero cap is an armed loop that still reaps and is a different
/// instruction.
pub(super) fn fleet(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    let workspace = args::workspace(ctx, verb)?;
    let gesture = if verb == crate::boundary::codec::FLEET_ARM {
        let word = args::required(tail, verb, "the cap — how many balls at once")?;
        crate::fleet::Verb::Arm {
            workspace,
            project: args::project(ctx, verb)?,
            cap: word
                .parse()
                .map_err(|_| format!("/{verb}: {word:?} is not a cap"))?,
        }
    } else {
        args::none(tail, verb)?;
        crate::fleet::Verb::Disarm { workspace }
    };
    Ok(act(Action::Fleet(gesture)))
}
