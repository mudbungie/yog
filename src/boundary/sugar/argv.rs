//! `yog gesture`'s own argv (§8.5): the one payload — a JSON envelope **or** a
//! slash line — and the context flags a line reads its elided targets from.
//!
//! The terminal is a seat like any other, and it holds no selection: nothing is
//! focused, so a line typed here states its targets outright. That is what the
//! flags are, and they are the same four facts the window reads off its focus
//! ([`AppModel::line_context`](crate::AppModel::line_context)) — never a fifth
//! way to name a target.
//!
//! **`--help` is not a flag of this verb** (§8.5): it rewrites the invocation
//! into the help gesture — `--help` is `/help`, `--help close` is `/help
//! close` — so the terminal asks the same question the composer's `/help`
//! asks, and one answer serves both. That is what threading a higher-order
//! operation through an interface looks like: a rewrite, not an arm.
//!
//! **`--prepared` is how the start flow composes across invocations** (bl-44d8).
//! Every `yog gesture` is its own process, so the [`Prepared`] a `/prepare`
//! returns cannot survive into the `/prompt` that fires it the way a window's
//! composer holds it — which made the two terminal steps the help advertises
//! impossible to actually run. It is a seat fact like the four above, and the
//! seat that has no composer states it: the `prepare` reply's own `prepared`
//! object, handed straight back (`--prepared "$(… | jq -c .prepared)"`). It is
//! read by the very codec that wrote it, so this is the deposited envelope's
//! spelling and not a second one.
//!
//! The focused **ball** is deliberately absent: an existing ball's spec carries
//! roster facts (its title, body and §3.5 join state) that no flag can state, so
//! a seat with no roster spells that one gesture as an envelope, which carries
//! them in full. Everything else is typable here.

use crate::boundary::Gesture;
use crate::boundary::codec::{self, prepared_from_value};
use crate::boundary::line::{self, Context};
use serde_json::Value;

/// One invocation: what to do, and the seat it is typed at.
struct Invocation {
    context: Context,
    /// The envelope or the line, verbatim.
    payload: String,
}

/// What one argv-seat invocation means: the gesture, and the envelope that
/// carries it. **Two argv seats share this one reader** — `yog gesture`, which
/// deposits into the world's inbox, and `yog seat`, which sends over the wire
/// (REMOTE §3's two intakes, bl-b6fa) — so the flags, the refusals and the
/// `--help` rewrite are one implementation and cannot drift apart. `verb` is
/// the seat's own name, which is all a usage line differs by.
pub(crate) fn read_gesture(verb: &str, args: &[String]) -> Result<(Gesture, Value), String> {
    read(args)
        .and_then(|invocation| envelope(&invocation))
        .map_err(|why| format!("{why}; {}", usage(verb)))
}

/// The deposit envelope this invocation means: a line read at the seat its
/// flags describe, or the JSON envelope validated as written. Either way what
/// is carried is the codec's own encoding — the line is a serialization of
/// the boundary, never a second inbox format.
fn envelope(invocation: &Invocation) -> Result<(Gesture, Value), String> {
    if line::is_command(&invocation.payload) {
        let gesture = line::parse(&invocation.payload, &invocation.context)?;
        let value = codec::encode(&gesture);
        return Ok((gesture, value));
    }
    let value: Value =
        serde_json::from_str(&invocation.payload).map_err(|e| format!("not JSON: {e}"))?;
    // The envelope is carried **as written**, not as re-encoded: the audit
    // keeps the operator's own bytes. Decoding is the validation and the read
    // the help short-circuit above needs.
    Ok((codec::decode(&value)?, value))
}

/// Read the multiplexed tail. Refuses — naming the offender — on an unknown
/// flag, a flag with no value, or anything other than exactly one payload.
/// The **usage line is not appended here** (bl-e66f): [`read_gesture`] carries
/// it onto every refusal this seat hands back, including the ones the line
/// parser raised, so there is one rule and one place rather than a per-site
/// decision that had already been made two ways.
fn read(args: &[String]) -> Result<Invocation, String> {
    let mut context = Context::default();
    let mut payload: Option<String> = None;
    let mut asked_help = false;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--help" || arg == "-h" {
            asked_help = true;
            continue;
        }
        let Some(flag) = arg.strip_prefix("--") else {
            if payload.replace(arg.clone()).is_some() {
                return Err(format!("expected one gesture, got another: {arg:?}"));
            }
            continue;
        };
        let value = rest
            .next()
            .ok_or_else(|| format!("--{flag} needs a value"))?
            .clone();
        match flag {
            "ws" => context.workspace = Some(value),
            "agent" => context.agent = Some(value),
            "project" => context.project = Some(value),
            "as" => context.name = Some(value),
            "prepared" => {
                let v = serde_json::from_str(&value)
                    .map_err(|e| format!("--prepared: not JSON: {e}"))?;
                context.prepared = Some(prepared_from_value(&v)?);
            }
            other => return Err(format!("unknown flag --{other}")),
        }
    }
    // The rewrite: whatever else was typed, help is what was asked. A word
    // beside it names the command asked about, with or without its slash.
    if asked_help {
        let about = payload.unwrap_or_default();
        let about = about.trim().trim_start_matches('/');
        return Ok(Invocation {
            context,
            payload: format!("/help {about}").trim_end().to_owned(),
        });
    }
    let payload = payload.ok_or_else(|| "nothing to do".to_owned())?;
    Ok(Invocation { context, payload })
}

/// The one usage line, so a refusal always says how to be right. It differs
/// between the two argv seats by the verb alone, which is the whole reason
/// this is a function rather than a second const.
///
/// `pub(crate)` since bl-e66f: the seat also prints it above the help answer,
/// because the flags had lived **only** in refusals — so the way to learn how
/// to aim a gesture was to type one wrong, and the refusal for a *missing
/// target* was the one refusal that did not carry it.
pub(crate) fn usage(verb: &str) -> String {
    format!(
        "usage: yog {verb} [--ws NAME] [--agent ID|NAME] [--project NAME] [--as NAME] \
         [--prepared JSON] '<json>' | '/line'\n       yog {verb} --help [command] — every \
         gesture, or one command's page"
    )
}
