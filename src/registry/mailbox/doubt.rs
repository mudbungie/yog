//! **What the engine says when it cannot know whether the box ran it**
//! (REMOTE §5.6): the three sentences this leg writes into a refusal or into a
//! capture, and the one clause all of them share.
//!
//! Split out of [`super`] at §12's per-file budget, on the seam the file
//! already had: everything else there is a *shape* — what an invocation is,
//! what a capture is, how each is spelled on the wire — and these are the
//! engine's own prose, the only text on this leg a model or an operator reads
//! that no tool produced.
//!
//! **They say one fact and they say it once.** Delivery on the routed leg is
//! at-least-once, so a hand-off that went out and never came back may still
//! have run on the box; the count is the whole of what the engine knows, and
//! [`handed_to`] is the clause that states it. What differs between the three
//! is only what happened *around* the count — nothing came back, something
//! came back late, or the handle is gone — so the count has one home and the
//! endings are three.

use super::Capture;

/// The clause every sentence here opens with: who it went to and how many
/// times. `{client:?}` quotes the identity, which is a certificate CN and can
/// carry anything.
fn handed_to(client: &str, handed: u32) -> String {
    format!("this invocation was handed to {client:?} {handed} times")
}

/// What the count *means*, and what to do about it — the gesture lane's own
/// instruction, said the same way here: the engine cannot know what the box
/// did, so the recovery is a read and never a re-send.
const MAY_HAVE_RUN: &str = "each hand-off may have run it";
const READ_FIRST: &str = "read the world before acting again";

/// The one sentence an unheld handle earns, said the same way at both readers.
///
/// **It names three causes and the third is the restart**
/// (REMOTE §5.6, ruling 3). The mailbox is RAM and stays RAM: the durable home
/// of a tool result is litany's step record, reached only through the driver's
/// collect, so a capture nobody had collected when this process died is gone.
/// A driver that outlives the engine then polls a handle the new one never
/// minted, and until bl-8016 was told two of the three reasons that could be
/// true. The third carries the gesture lane's own instruction, because the
/// engine cannot know whether the box ran it.
pub(super) fn unknown(invocation: &str) -> String {
    format!(
        "no invocation {invocation:?} is in flight; it was answered already, it expired, or \
         this engine restarted since it was posted — the box may have run it; read the world \
         before acting again"
    )
}

/// **What an exhausted lease is answered with** (REMOTE §5.6, ruling 2): the
/// engine's own capture, written at the read that would hand a slot out for a
/// fourth time. Non-zero, because it is a failed tool result and the model
/// reads it as one; empty on stdout, because nothing ran *here*; and one
/// sentence naming the client, the count and the instruction the gesture lane
/// already gives — a box that dropped three hand-offs of one tool, with its
/// own redial series in between, is a box that tool is killing, and the
/// recovery is a read rather than a re-send.
pub(super) fn in_doubt(client: &str, handed: u32) -> Capture {
    Capture {
        stdout: String::new(),
        stderr: format!(
            "{} and never answered; {MAY_HAVE_RUN}; {READ_FIRST}",
            handed_to(client, handed)
        ),
        exit_code: 1,
    }
}

/// **The mark a redelivered invocation's capture carries** (REMOTE §5.6 as
/// amended by bl-0655): below the cap, the engine says the count *earlier* and
/// without giving up.
///
/// A foot restarted mid-flight ran one command twice on the box, and the
/// capture the model was handed — `Exit code: 0`, and the command's own output
/// — was indistinguishable from one clean run. The engine already held the
/// hand-off count and spent it only on the give-up case; at hand-off two,
/// which is the common case (one restart, one deploy, one blip), it said
/// nothing, so a doubled `apt install` or a doubled migration reported itself
/// as a single one.
///
/// **The mark rides stderr, not a field of its own.** The three facts a
/// capture carries are the whole of what reaches the model — litany renders
/// exit code, stdout and stderr into the `tool_result` envelope and nothing
/// else (its ARCH §3.3) — so a field would be a fact only a decoder could
/// read, on the one lane whose reader is a language model. It goes *first*,
/// because the bounded projection cuts what a long capture ends with.
///
/// **One hand-off is not a redelivery, and a re-post is not a second run.**
/// The count is the slot's, so this is silent at `handed == 1`; and Ruling 1's
/// held capture is posted on the next dial *before* that channel's first
/// follow-class read, so the slot is still at one when it lands — the case
/// where the box ran it once is the case that says nothing.
pub(super) fn redelivered(client: &str, handed: u32, capture: &Capture) -> Capture {
    if handed < 2 {
        return capture.clone();
    }
    let mark = format!(
        "{}; {MAY_HAVE_RUN}, so what this capture reports may be the effect of \
         more than one run; {READ_FIRST}",
        handed_to(client, handed)
    );
    Capture {
        stderr: match capture.stderr.as_str() {
            "" => mark,
            said => format!("{mark}\n{said}"),
        },
        ..capture.clone()
    }
}
