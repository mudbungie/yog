//! The Steps tab's **drill-in tier**: the record picker and the jsonview trees
//! behind it (§11). Split from the list tier for the same reason
//! [`super::detail`] is split from [`super::build`] — the cheap list should not
//! carry the heavy read's rendering, and neither file then rides the §12 cap.
//!
//! Every record renders through [`crate::jsonview`] — or, under the §11 Raw
//! toggle, as the record file's own bytes — so the whole tab keeps its
//! promise: every byte inspectable. The record names are the on-disk file names
//! (ARCH §2.3), which say nothing to an operator who has never read the spec —
//! so each carries its meaning on hover, the `Workspaces:` label's idiom
//! (bl-2d87, bl-3ffc).
//!
//! **What the picker offers is [`super::records`]'s answer**, not this file's
//! (bl-83d6): the five JSON records always, and each capture log the step has
//! bytes in. A log renders as bounded bytes rather than a tree, through the one
//! painter the Files tab's preview already uses — nothing parsed a log, so Raw
//! has nothing to escape from and paints it identically.

use std::collections::HashSet;

use super::{Doc, StepDetail, StepTab, ToolIo, records};
use crate::files_view::Preview;
use crate::theme;

/// What a tool call's opaque id is for.
const TOOL_ID_HINT: &str = "The provider's id for this call — it is what ties this \
     input and output to the matching tool event in the response record.";

/// What a record with no bytes says — one wording, whether the reader is
/// looking at the parsed view or the Raw one, because it is one fact.
const ABSENT: &str = "(absent)";

/// The record picker, then the selected record's trees. `collapsed` is the
/// caller-owned jsonview collapse state (§5.3), threaded into every tree;
/// `raw` is the §11 Raw toggle, which replaces every tree here with the
/// record file's own bytes.
pub(super) fn render_detail(
    ui: &mut egui::Ui,
    detail: &StepDetail,
    tab: StepTab,
    collapsed: &mut HashSet<String>,
    raw: bool,
) {
    ui.horizontal(|ui| {
        ui.label("Records:").on_hover_text(
            "The files behind this step. Pick one to read it in full — nothing \
             here is summarized away. A `.log` seat appears when something was \
             written to it.",
        );
        for (which, label, hint) in records::seats(Some(detail)) {
            let text = egui::RichText::new(label);
            ui.label(if which == tab { text.strong() } else { text })
                .on_hover_text(hint);
        }
    });
    match tab {
        StepTab::Meta => render_doc(
            ui,
            &detail.meta,
            &format!("{}/meta", detail.seq),
            collapsed,
            raw,
        ),
        StepTab::Request => render_doc(
            ui,
            &detail.request,
            &format!("{}/request", detail.seq),
            collapsed,
            raw,
        ),
        StepTab::Staging => render_doc(
            ui,
            &detail.staging,
            &format!("{}/staging", detail.seq),
            collapsed,
            raw,
        ),
        StepTab::Response => render_response(ui, &detail.response, &detail.seq, collapsed, raw),
        StepTab::Tools => render_tools(ui, &detail.tools, &detail.seq, collapsed, raw),
        StepTab::Stderr => render_log(ui, detail.stderr.as_ref()),
        StepTab::Driver => render_log(ui, detail.driver.as_ref()),
    }
}

/// A capture log's bytes, bounded (bl-83d6). The painter is the Files tab's own
/// ([`crate::files_view::preview_body`]) — one wording for "this is all of it",
/// "truncated at 64 KiB of N bytes" and "binary file", because a second
/// spelling of a bound is how two surfaces come to disagree about it.
///
/// `None` is a log with no bytes, which is also a log the picker did not offer
/// a seat for: reachable only by holding a selection across a step that has one
/// to a step that does not, and it says [`ABSENT`] — the same word every other
/// empty record says, rather than a blank the reader has to interpret.
fn render_log(ui: &mut egui::Ui, log: Option<&Preview>) {
    match log {
        Some(preview) => crate::files_view::preview_body(ui, preview),
        None => {
            ui.weak(ABSENT);
        }
    }
}

/// A single record: under Raw, [`render_raw`]'s verbatim bytes; otherwise a
/// jsonview tree; "(absent)" for a file with no bytes; or — for bytes that are
/// not JSON — the §11 **error row** ([`super::UNPARSED`], in ichor, the same
/// grammar as the §7.3 wound sentence) above the bytes kept verbatim. Both
/// halves of the promise: nothing is summarized away, *and* the reader is told
/// the file is broken rather than left to mistake it for prose.
fn render_doc(
    ui: &mut egui::Ui,
    doc: &Doc,
    root: &str,
    collapsed: &mut HashSet<String>,
    raw: bool,
) {
    if raw {
        render_raw(ui, doc);
        return;
    }
    match doc {
        Doc::Json { value, .. } => crate::jsonview::render(ui, value, root, collapsed),
        Doc::Absent => {
            ui.weak(ABSENT);
        }
        Doc::Unparsed(bytes) => {
            ui.colored_label(theme::ICHOR, super::UNPARSED);
            ui.monospace(String::from_utf8_lossy(bytes));
        }
    }
}

/// The record's bytes, unaltered (§11 Raw). No framing rides along — the
/// [`super::UNPARSED`] row is the parsed view's *word about* the file, and Raw
/// is the file. A record with no bytes has none to show and says so, rather
/// than painting a blank the reader has to interpret.
fn render_raw(ui: &mut egui::Ui, doc: &Doc) {
    let bytes = doc.raw();
    if bytes.is_empty() {
        ui.weak(ABSENT);
        return;
    }
    ui.monospace(String::from_utf8_lossy(bytes));
}

fn render_response(
    ui: &mut egui::Ui,
    events: &[Doc],
    seq: &str,
    collapsed: &mut HashSet<String>,
    raw: bool,
) {
    if events.is_empty() {
        ui.label("(no events)");
    }
    for (i, event) in events.iter().enumerate() {
        render_doc(ui, event, &format!("{seq}/resp/{i}"), collapsed, raw);
    }
}

fn render_tools(
    ui: &mut egui::Ui,
    tools: &[ToolIo],
    seq: &str,
    collapsed: &mut HashSet<String>,
    raw: bool,
) {
    if tools.is_empty() {
        ui.label("(no tool calls)");
    }
    for tool in tools {
        ui.horizontal(|ui| {
            // §11 glyph doctrine: the ok-vs-error mapping is the transcript
            // result row's own ([`theme::tool_result_badge`] — one home for
            // glyph, hue and words), and this seat heads a tool's input/output
            // trees on a line of its own, so it has the room to say the
            // outcome outright rather than hide it behind a hover.
            let (glyph, color, phrase) = theme::tool_result_badge(tool.is_error);
            ui.colored_label(color, format!("{glyph} {phrase}"));
            ui.label("call");
            ui.monospace(&tool.tool_id).on_hover_text(TOOL_ID_HINT);
        });
        ui.label("input");
        render_doc(
            ui,
            &tool.input,
            &format!("{seq}/{}/in", tool.tool_id),
            collapsed,
            raw,
        );
        ui.label("output");
        render_doc(
            ui,
            &tool.output,
            &format!("{seq}/{}/out", tool.tool_id),
            collapsed,
            raw,
        );
    }
}
