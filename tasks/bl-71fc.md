+++
title = "the Inbox tab's empty state is a bottom-anchored '(no deposits)' that names no paved path"
created = 1786163412
updated = 1786163412
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
QUALITY.md §1 criterion **H2** ('Absence is named. An empty region says what it is and names the paved path in full'). Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out. **This is the mildest violation in the audit** — filed for completeness, and it should be weighed against the cost of touching the surface.

SYMPTOM. With no mail, the Inbox tab paints a single parenthesised '(no deposits)' pinned to the BOTTOM of an otherwise empty pane — roughly 450px below the tab strip that produced it. It says what is absent but not how a deposit ever arrives.

WITNESS: `Q-S7-tab3-default.png`. Compare the sibling Work tab in the same inspector, `Q-S7-tab6-default.png`, which top-anchors a full sentence: *'nothing changed yet — the branch is there, and it carries no edits.'* followed by *'pick a file to read its changes'*. Two empty states in one tab strip, two different shapes and two different anchors.

REPRODUCTION: select a conversation with no inbox deposits (bare Down), then bare 3.

TRIAGE ONLY — filed by the first quality audit, not fixed by it.