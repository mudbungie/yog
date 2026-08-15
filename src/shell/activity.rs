//! The activity accessory (§11): the demoted ops pane, a collapsed bottom
//! chip that expands on demand to the `ops.jsonl` tail — never inline rows in
//! conversation space. Coverage-excluded glue: the chip counts
//! ([`AppModel::activity`]) are a tested view-model; this file only wires the
//! collapsing header and the two verbs over the trail itself
//! ([`AppModel::ack_failures`] / [`AppModel::clear_trail`], bl-c417 — both
//! tested, both `ops.jsonl` writes rather than widget state).
//!
//! **The rows come over the wire** (REMOTE §1.2 and its read-path residual;
//! bl-adcb). The expanded
//! trail is a `Reply::Ops` that crossed loopback mTLS and was decoded by
//! `reply::decode`, exactly as the clients section beside it is (bl-ae05): the
//! window is a client of its own engine, so the tail it paints is the tail the
//! boundary answers, asked once per [`asker`](crate::wire::asker) pass rather
//! than derived per frame. There is nothing left to memoize — an answer *is*
//! the cached read — and there is no in-process accessor either, the model's
//! `ops_rows` having gone with the derivation.
//!
//! **The frame never waits**, so a trail opened on the frame that first asked
//! paints its rows one cadence period later; until then it is honestly empty,
//! and a refusal is painted rather than swallowed.

use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::reply::Reply;
use crate::opslog::{OPS_TAIL, OpRow};
use crate::theme;

/// The collapsed chip (`activity · N ops · M failed ⚠ · K drift`, ichor whenever
/// either count is non-zero — a drift is an alarm about yog's own watch layer,
/// §7.2) and,
/// expanded, the ops rows — each opening to the full entry (argv, cwd, exit,
/// stderr), because a trail that hides *why* is not a trail (§7.3). Default
/// collapsed; per-instance viewport state (§13.0) held by the caller in
/// `open`, so the header's click and the §11 `a` binding move one fact.
pub fn accessory(ui: &mut egui::Ui, model: &mut AppModel, open: &mut bool) {
    // What the derivation itself is doing (§7.2, bl-ee0a): how stale the
    // rendered snapshot is, and what grew since the last one. Both are `None`
    // in the ordinary case and render nothing; both are tested view-models
    // ([`AppModel::staleness`], [`AppModel::growth_note`]).
    for note in [model.staleness(), model.growth_note()]
        .into_iter()
        .flatten()
    {
        ui.colored_label(theme::ICHOR, note);
    }
    // **Asked only while the pane is open** (REMOTE §1.2): a question is keyed
    // by its own envelope, so a collapsed trail simply stops declaring one and
    // the asker drops it — no unsubscribe, and a chip nobody expanded costs the
    // wire nothing. The bound is the log's own ([`OPS_TAIL`]), never a second
    // number this seat picked.
    let trail = if *open {
        super::wire::ask(model, Query::Ops { max: OPS_TAIL }, |reply| match reply {
            Reply::Ops(rows) => Some(rows),
            _ => None,
        })
    } else {
        super::wire::Landed::default()
    };
    let rows: Vec<OpRow> = trail.value.unwrap_or_default();
    let refused = trail.refused;
    let summary = model.activity();
    let heading = if summary.errors > 0 || summary.drifts > 0 {
        egui::RichText::new(summary.chip()).color(theme::ICHOR)
    } else {
        egui::RichText::new(summary.chip()).weak()
    };
    let header = egui::CollapsingHeader::new(heading)
        .id_salt("activity-accessory")
        .open(Some(*open))
        .show(ui, |ui| {
            trail_controls(ui, model);
            // §11 tail idiom: `ops.jsonl` is chronological, so the newest op is
            // the bottom row and the trail opens on it — the newest line on the
            // bottom edge whether the trail is two ops or two hundred.
            //
            // The trail fills whatever height the operator dragged the pane to
            // (§4.1 `panels`) — never a fixed cap, which used to hold the ops
            // history to ~6 rows in a full-height window. The scroll body takes
            // the space actually left and no more, which is also what keeps a
            // long trail from ratcheting the panel taller (egui grows a panel to
            // fit content that overflows); an explicit `max_height` of that same
            // available height used to say so twice.
            crate::tail::scroll(ui, true, |ui| {
                // A refusal is painted, not swallowed: the wire is how this
                // pane reads, so what it was told is the trail's honest content.
                if let Some(said) = &refused {
                    ui.colored_label(theme::ICHOR, said);
                }
                // Outcomes are positional over the same rows (§6 retirement).
                let outcomes = crate::opslog::outcomes(&rows);
                for (i, (row, outcome)) in rows.iter().zip(outcomes).enumerate() {
                    ui.push_id(i, |ui| ops_row(ui, row, outcome));
                }
            });
        })
        .header_response
        .on_hover_text(
            "Open the ops trail: every command yog has run — lernie, bl, bz — with \
             its exit code and everything it printed. This is where a failure's \
             reason lives (a).",
        );
    if header.clicked() {
        *open = !*open;
    }
}

/// The expanded pane's two verbs over the trail itself (bl-c417): **Dismiss**,
/// offered only while something is actually alarming ([`AppModel::has_alarms`] —
/// a control that would write a line and change nothing should not be there),
/// and **Clear trail**.
///
/// Both live *inside* the expansion, never on the collapsed chip. That is the
/// whole guard on the destructive one: reaching Clear costs opening the pane
/// first, so no single misclick on a chip that sits at the bottom of every
/// frame can end a trail. Each explains itself on hover (bl-68ac), and the
/// words come from `opslog::operator` so the banner's Dismiss and this one are
/// spelled once.
fn trail_controls(ui: &mut egui::Ui, model: &mut AppModel) {
    use crate::opslog::operator;
    ui.horizontal(|ui| {
        if model.has_alarms()
            && ui
                .small_button(operator::ACK_LABEL)
                .on_hover_text(operator::ACK_HOVER)
                .clicked()
        {
            model.ack_failures();
        }
        if ui
            .small_button(operator::CLEAR_LABEL)
            .on_hover_text(operator::CLEAR_HOVER)
            .clicked()
        {
            model.clear_trail();
        }
    });
}

/// One expandable ops row (§7.3): the summary heading — ichor red on a **live**
/// failure — opens to the full `ops.jsonl` entry. A *retired* failure (§6: a
/// later clean run of the same verb superseded it) keeps its ⚠ and its record
/// but goes ash: the fact stays, the prominence retires.
///
/// §11 glyph doctrine: glyph, hue and words all come from `theme::op_badge`, and
/// this seat — a dense repeating row whose argv is already long — says the
/// outcome on **hover** rather than inline, the doctrine's stated minimum. An
/// inline "ran clean" on every row would bury the one row that failed.
///
/// The leading column is [`OpRow::when`] (bl-61db), not the raw `ts` — the
/// row used to lead with an unreadable epoch (`1785630266`). The epoch is a
/// lossless round-trip of the same instant `when()` renders, so it earns no
/// seat in the expanded detail either.
///
/// The collapsed heading reads [`OpRow::when`] and [`OpRow::summary`] — a
/// prompt op's `argv` carries an arbitrary-length, multi-line goal, and this
/// row is a scan surface, one line per op (bl-0bf9). `summary` is the only
/// place that elides; the expansion below always shows the byte-exact `argv`.
fn ops_row(ui: &mut egui::Ui, row: &crate::opslog::OpRow, outcome: crate::opslog::OpOutcome) {
    let (glyph, hue, phrase) = theme::op_badge(outcome);
    // A "⋯" hint marks a row worth expanding — one whose captured streams carried
    // bytes ([`OpRow::has_output`]); a bare row expands only to cwd/exit.
    let more = if row.has_output() { " ⋯" } else { "" };
    let summary = format!("{glyph} {} {}{more}", row.when(), row.summary());
    let heading = egui::RichText::new(summary).color(hue);
    ui.collapsing(heading, |ui| {
        ui.label(format!("cwd: {}", row.cwd));
        ui.label(format!("argv: {}", row.argv));
        // The exit **said in words**, never the bare field (bl-afa9): three of
        // its values are §4.2 sentinels, and a negative integer in an exit
        // column reads as a signal death. `OpRow::exit_label` is the one home
        // of that wording.
        ui.label(row.exit_label());
        if !row.stdout.is_empty() {
            ui.label("stdout:");
            ui.code(&row.stdout);
        }
        if !row.stderr.is_empty() {
            ui.colored_label(theme::ICHOR, "stderr:");
            ui.code(&row.stderr);
        }
    })
    .header_response
    .on_hover_text(format!(
        "{phrase} — open the row for the command's directory, its whole argv, how it \
         exited and everything it printed. No key of its own: Tab reaches it, Space \
         presses it."
    ));
}
