+++
title = "a button's whole label elides to a bare '…' — including the Login button on the one provider that is not signed in"
created = 1786163340
updated = 1786163340
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
QUALITY.md §1 criteria **G1** ('deliberate elision shows an ellipsis and the full value is reachable') and **H2** ('never a bare blank or an unlabeled box'). Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out.

SYMPTOM. Side-panel rows lay a label and a trailing button on one line and give the button whatever width is left. When the label is long the button's text is elided to nothing — the control renders as a bare '…' that says neither what it is nor what it does. Both witnesses are at the DEFAULT 1150x760 window, not a small one.

WITNESS 1 — the balls board's assign badge, `Q-S13-default.png` / crop `crop-s13-board.png`:
    ready    ▶ bl-7cdd: tune the transcript scroll anchor    [ … ]
    gated    ▶ bl-b5ce: epic: the inspector rework           [ assig… ]
Same button (`assign → <ws>`, `src/shell/start_rows.rs:61`), rendered as 'assig…' on one row and as a bare '…' on the row whose title is two characters longer.

WITNESS 2 — the Login pane, and this is the damaging one, `Q-S5-login-default.png` / crop `crop-s5-login.png`:
    openai-chatgpt         auth oauth2 · <state>       [ Login ]
    claude-session-direct  auth oauth2 · <state>   [ … ]
The provider that IS signed in gets a legible 'Login'; the provider that is NOT signed in — the one row where the operator actually needs the verb — gets a bare '…', because 'claude-session-direct' is the longest name in the table.

REPRODUCTION:
  1. launch on a seeded project carrying a ball whose title is ~40 chars (witness 1)
  2. CLICK 'Login' in the side panel to expand the pane (fold selection is a view; no §11 binding exists) (witness 2)
Both at the default window size; widening the panel restores the labels, which is what makes it an elision defect rather than a missing control.

TRIAGE ONLY — filed by the first quality audit, not fixed by it.