//! **The enrolment grammar** (REMOTE §1.4 as amended, §4.2, §8.4) — `/enroll`'s
//! two positional words and its one flag, in its own file beside the other
//! per-family grammars ([`fan`](super::fan), [`fork`](super::fork),
//! [`tools`](super::tools)) rather than in [`verbs`](super::verbs), which took
//! it past §12's cap when the flag landed (bl-fec6). The seam is the same one
//! those three are cut on: a verb whose grammar is more than words is its own
//! file.

use super::{Context, args};
use crate::boundary::{Action, Gesture};

/// The one flag `/enroll` reads — the address the device will dial (bl-fec6),
/// spelled once so the parser, the speller and the page cannot disagree.
const AT: &str = "at";

/// `/enroll <common-name> [foot|thrall] [--at <host:port>]` — REMOTE §1.4's
/// enrollment, typed
/// (bl-f4e3; brands read since bl-427b).
///
/// The common name is the one fact no seat's context holds: it names a machine
/// that has never connected, so nothing on this side can have it selected. The
/// workspace it seats the new client in is the seat's own, exactly as
/// `/marks`' and `/clients`' are.
///
/// **Bare is operator grade** (§4.2's default-operator, made typable): there is
/// one word to add and adding it is the demotion, so no spelling can promote a
/// foot by accident. **Two vocabularies read, one spells** (bl-427b): the
/// registry's own words (`foot`, and `operator` said outright) and the brands
/// an operator says out loud (`thrall`, `lernie`) both parse — `foot` without
/// context confused exactly the operator this verb serves — while `spell`
/// emits only the registry's word, so a spelled line round-trips in one
/// vocabulary. Anything else refuses naming the token rather than rounding to
/// either grade.
pub(super) fn enroll(tail: &str, ctx: &Context, verb: &str) -> Result<Gesture, String> {
    // **`--at <host:port>` is the address the DEVICE will dial** (bl-fec6),
    // read off the flag half so the two positional words are unchanged. It is
    // a flag rather than a third word because it is optional beside another
    // optional, and `--at` reads as the preposition it is.
    let (words, flags) = args::split_flags(tail);
    args::only(&flags, &[AT], verb)?;
    let address = args::flag(&flags, AT, verb)?;
    let (name, rest) = args::first_word(&words);
    if name.is_empty() {
        return Err(format!(
            "/{verb}: the common name the device's certificate will carry is required — \
             /{verb} <common-name> [{}] — bare enrolls a Lernie (operator grade: the seat, \
             which reads and steers and hosts its device's tools); add {0} (or thrall) for \
             a device that should ONLY offer tools",
            crate::registry::peer::FOOT
        ));
    }
    let grade = match args::optional_word(&rest, verb)?.as_deref() {
        None | Some("lernie") => crate::registry::Grade::default(),
        Some("thrall") => crate::registry::Grade::Foot,
        Some(word) => crate::boundary::codec::grade_of(word).map_err(|e| {
            format!(
                "/{verb}: {e} — say thrall or foot for a tools-only device, lernie or \
                     operator (the bare default) for a seat"
            )
        })?,
    };
    Ok(Gesture::Act(Action::Enroll(
        crate::registry::enroll::Request {
            workspace: args::workspace(ctx, verb)?,
            name,
            grade,
            address,
        },
    )))
}
