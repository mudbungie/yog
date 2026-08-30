//! One transcript entry → its rows (§11): the exhaustive per-variant match and
//! the labels each variant wears. The row *vocabulary* it builds with —
//! [`Row`], [`RowClass`], [`Tone`] — lives in [`super`], which is the only
//! caller; **how** a row is made of those parts is [`build`], split off at
//! §12's per-file budget on the seam this doc already drew. The one arm that
//! projects a *hole* rather than something somebody said is [`compacted`],
//! split off at that same budget on that same seam.
//!
//! **Only the tool-result row states its size** ([`build::with_size`], bl-1f75) —
//! it is the row whose payload the operator cannot guess: `✔ tool result — ok`
//! says nothing about whether the `▶` opens onto four characters or forty
//! thousand. The **live tail stays bare** and that is deliberate: it is
//! in-flight, so it is already expanded on screen (`super::expanded_for`),
//! and how much of the answer has landed is the in-flight strip's own line
//! (`nav::convs::flight` — `<who> · N chars streamed`, §5.1 #28a). A second,
//! per-frame spelling of a growing number in the row seat would say what the
//! screen already shows, twice.

use super::{Row, RowClass, Tone};
use crate::theme::{Role, message_role};
use crate::transcript::{Block, Entry, EntryKind, Transcript};

mod build;
mod compacted;

pub(crate) use build::key;
use build::{row, with_size};
use compacted::compacted_row;

/// Rows for one entry: one per model content block, else one for the entry.
pub(super) fn push_entry(
    transcript: &Transcript,
    entry: &Entry,
    speaker: &str,
    out: &mut Vec<Row>,
) {
    match &entry.kind {
        // A result message can assert an epitaph and no content (ARCH §2.6),
        // so its body is empty once the envelope is off — say so, exactly as
        // the no-content-blocks arm below does for its own empty case. The
        // ending itself rides the prefix seat ([`delivered_prefix`]), so the
        // pair never renders as a blank line from a stranger (bl-71e8).
        EntryKind::Delivered {
            sender,
            epitaph,
            body,
        } if body.is_empty() => out.push(row(
            key(&entry.name, 0),
            delivered_prefix(sender, epitaph.as_ref()),
            "(no message body)",
            RowClass::Response,
            Tone::Weak,
            Some(message_role(sender, epitaph.is_some())),
        )),
        EntryKind::Delivered {
            sender,
            epitaph,
            body,
        } => out.push(row(
            key(&entry.name, 0),
            delivered_prefix(sender, epitaph.as_ref()),
            body,
            RowClass::Response,
            Tone::Plain,
            Some(message_role(sender, epitaph.is_some())),
        )),
        EntryKind::Model {
            model_id, blocks, ..
        } if blocks.is_empty() => out.push(Row {
            hover: model_hover(model_id),
            ..row(
                key(&entry.name, 0),
                format!("{speaker}:"),
                "(no content blocks)",
                RowClass::Other,
                Tone::Weak,
                Some(Role::Model),
            )
        }),
        EntryKind::Model {
            model_id, blocks, ..
        } => {
            for (i, block) in blocks.iter().enumerate() {
                out.push(block_row(
                    transcript,
                    key(&entry.name, i),
                    speaker,
                    model_id,
                    block,
                ));
            }
        }
        EntryKind::ToolResult {
            content, is_error, ..
        } => {
            // §11 glyph doctrine: the outcome's words come from the one
            // mapping that owns the glyph ([`crate::theme::tool_result_badge`])
            // and are never invented here, and the prefix seat says them
            // outright — it is this row's always-visible identity slot, worded
            // for every other row class already. The hue arrives as a [`Tone`]
            // rather than the mapping's `Color32`: this projection is headless
            // and names no RGB, and the render paints `Good`/`Bad` with that
            // very same hydra/ichor pair. The size hint ([`with_size`]) takes
            // that same seat: it has to be legible **contracted**, which the
            // hover is not (it needs a pointer on the row) and the preview is
            // not (it is the payload's own first line) — and it trails the
            // outcome, so the row still leads with what it is.
            let (glyph, _, phrase) = crate::theme::tool_result_badge(*is_error);
            let tone = if *is_error { Tone::Bad } else { Tone::Good };
            out.push(with_size(row(
                key(&entry.name, 0),
                format!("{glyph} {phrase}"),
                content,
                RowClass::Other,
                tone,
                None,
            )));
        }
        EntryKind::Streaming { thinking, text } => {
            push_streaming(&entry.name, thinking, text, out);
        }
        EntryKind::Compacted {
            first,
            last,
            summary,
        } => out.push(compacted_row(&entry.name, *first, *last, summary)),
        EntryKind::Raw => out.push(row(
            key(&entry.name, 0),
            entry.name.clone(),
            &String::from_utf8_lossy(&entry.raw),
            RowClass::Other,
            Tone::Weak,
            None,
        )),
    }
}

/// One model content block as a row. A tool call still awaiting its result
/// says so in words beside the pulse (§11 glyph doctrine: the hue is never
/// the only carrier).
fn block_row(
    transcript: &Transcript,
    key: String,
    speaker: &str,
    model_id: &str,
    block: &Block,
) -> Row {
    match block {
        Block::Text(text) => Row {
            hover: model_hover(model_id),
            ..row(
                key,
                format!("{speaker}:"),
                text,
                RowClass::Response,
                Tone::Plain,
                Some(Role::Model),
            )
        },
        Block::Thinking(text) => row(
            key,
            "thinking:".to_string(),
            text,
            RowClass::Other,
            Tone::Weak,
            None,
        ),
        Block::ToolUse {
            id,
            name,
            input_summary,
        } => {
            let running = transcript.tool_in_progress(id);
            let prefix = if running {
                format!("⚙ {name} — running")
            } else {
                format!("⚙ {name}")
            };
            let tone = if running { Tone::InFlight } else { Tone::Plain };
            row(key, prefix, input_summary, RowClass::Other, tone, None)
        }
    }
}

/// The prefix seat of a delivered message: the sender, plus **how it ended**
/// when the envelope asserted an ending (§2.6). An `epitaph:` marks the
/// message as a *result deposit* — a child's terminal, arriving because this
/// agent dispatched it, not because someone chose to speak — and on a
/// `stopped` / `died` one it is the entire message. Saying it outright in the
/// always-visible seat is the §11 glyph doctrine's rule applied to words: the
/// wording comes from the one mapping that owns it
/// ([`crate::inboxview::Epitaph::label`]) and is never invented here. A
/// message with no epitaph — the operator's own words, a peer's — reads
/// exactly as it always has.
fn delivered_prefix(sender: &str, epitaph: Option<&crate::inboxview::Epitaph>) -> String {
    match epitaph {
        Some(epitaph) => format!("{sender} ended: {}", epitaph.label()),
        None => format!("{sender}:"),
    }
}

/// The live tail is up to **two** rows and they are the same two a committed
/// model turn has — reasoning, then the answer (§7.2 the thinking ruling).
/// Each keeps its committed counterpart's class, so the fold knobs mean one
/// thing on either side of the commit; what differs is the tone, and
/// `Tone::Live` is what auto-expands them while the step is happening
/// ([`super::expanded_for`]). An empty half is no row at all: a model that has
/// only thought so far shows one growing row, not one growing row and one
/// blank one.
fn push_streaming(name: &str, thinking: &str, text: &str, out: &mut Vec<Row>) {
    if !thinking.is_empty() {
        out.push(row(
            key(name, 0),
            "thinking:".to_string(),
            thinking,
            RowClass::Other,
            Tone::Live,
            None,
        ));
    }
    if !text.is_empty() {
        out.push(row(
            key(name, 1),
            "live:".to_string(),
            text,
            RowClass::Response,
            Tone::Live,
            Some(Role::Model),
        ));
    }
}

/// What a model turn's speaker label stands for: the model that ran it
/// (bl-2335). The model id is a **config** fact — which model the conversation's
/// governing commit assigned (§9.4) — not a speaker, so it rides the hover while
/// the label names the agent. One turn can name a different model than the
/// header's current assignment, and that is the truth of that turn: the id here
/// is read from the entry litany itself wrote.
fn model_hover(model_id: &str) -> String {
    format!("ran on {model_id} — the model is config, not the speaker")
}
