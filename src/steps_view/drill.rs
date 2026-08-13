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

use std::collections::HashSet;

use super::{Doc, StepDetail, StepTab, ToolIo};
use crate::theme;

/// The five records a step leaves behind — the picker's word, and what that
/// word means for someone reading it cold. `pub(crate)`: the §11
/// discoverability invariant (bl-68ac) makes the shell's own step-tab control
/// carry the same explanation, and two spellings of one fact drift — this is
/// its one home, the same argument the column table makes for a heading.
pub(crate) const RECORDS: [(StepTab, &str, &str); 5] = [
    (
        StepTab::Meta,
        "meta",
        "The step's own note about itself: the commit it started from and the \
         times it began and ended.",
    ),
    (
        StepTab::Request,
        "request",
        "Exactly what was sent to the model to open this step — the prompt, the \
         history and the settings, as they went over the wire.",
    ),
    (
        StepTab::Staging,
        "staging",
        "The conversation entry being assembled out of this step's reply, caught \
         mid-write before it became part of the transcript.",
    ),
    (
        StepTab::Response,
        "response",
        "The model's reply as it streamed back, one event per line — text, tool \
         calls, usage and the end of each attempt.",
    ),
    (
        StepTab::Tools,
        "tools",
        "Every tool this step called, each with the arguments it was handed and \
         what it gave back.",
    ),
];

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
            "The five files this step wrote. Pick one to read it in full — \
             nothing here is summarized away.",
        );
        for (which, label, hint) in RECORDS {
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
