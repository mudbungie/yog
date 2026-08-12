+++
title = "the Inbox tab's empty state is a bottom-anchored '(no deposits)' that names no paved path"
created = 1786163412
updated = 1786514125
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
QUALITY.md §1 criterion **H2** ('Absence is named. An empty region says what it is and names the paved path in full'). Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out. **This is the mildest violation in the audit** — filed for completeness, and it should be weighed against the cost of touching the surface.

SYMPTOM. With no mail, the Inbox tab paints a single parenthesised '(no deposits)' pinned to the BOTTOM of an otherwise empty pane — roughly 450px below the tab strip that produced it. It says what is absent but not how a deposit ever arrives.

WITNESS: `Q-S7-tab3-default.png`. Compare the sibling Work tab in the same inspector, `Q-S7-tab6-default.png`, which top-anchors a full sentence: *'nothing changed yet — the branch is there, and it carries no edits.'* followed by *'pick a file to read its changes'*. Two empty states in one tab strip, two different shapes and two different anchors.

REPRODUCTION: select a conversation with no inbox deposits (bare Down), then bare 3.

TRIAGE ONLY — filed by the first quality audit, not fixed by it.

---

GROUNDWORK (verified 2026-08-11, Ferrule, read-only — NOT claimed, NOT worked).

The surface is `src/inboxview/render.rs:18-35`. Both halves of the complaint have one cause:

    crate::tail::scroll(ui, true, |ui| {
        if entries.is_empty() {
            ui.label("(no deposits)");
            return;
        }
        ...

The bottom anchoring is not a layout accident — it is `tail::scroll`'s `anchored = true`. Per `src/tail.rs`'s own doc that argument is the surface's answer to 'is my bottom row my newest content?', and it takes BOTH halves of the tail idiom together (stick-to-bottom AND the top pad that pushes an underfull body onto the bottom edge). For a one-line empty state the honest answer to that question is no: there is no newest content. That is why '(no deposits)' sits ~450px below the tab strip — the pad is doing exactly what it was built to do, to content that should not be asking for it.

The sibling the ball compares against is `src/workdiff/render.rs:97`:

    ui.weak("nothing changed yet — the branch is there, and it carries no edits.");

top-anchored, a full sentence, and `render_patch` adds a second line naming the next gesture ('pick a file to read its changes', line 138).

THE VALUABLE PART: `src/inboxview/` is NOT coverage-excluded. tarpaulin excludes only `src/main.rs` and `src/shell/*` (`tarpaulin.toml`, AGENTS.md 'The local gate'). This surface already has real paint tests — `src/inboxview/tests.rs:140-141` assert `painted(&[], false).contains("(no deposits)")` in both raw and parsed modes. So this ball is directly testable in-crate with no acceptance-fixture work at all, and those two existing assertions are the ones to rewrite rather than a new harness.

Caution for the fixer: those two lines are exactly the vacuous-assertion shape — they pin the string but say nothing about WHERE it sits, which is the whole complaint. Assert the anchor too (`paint_probe::painted_settled` returns positions), or the fix can land while the test keeps passing on the unfixed layout.

The ball itself says this is the mildest violation in the audit and should be weighed against the cost of touching the surface. Given it is in-crate, already tested, and the fix is one argument plus a sentence, that cost is low.
