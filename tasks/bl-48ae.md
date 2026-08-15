+++
title = "REMOTE read path: the seat's own Query::Agent — the last in-process read, and the rendering ruling it needs"
created = 1786768626
updated = 1786769175
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
`AppModel::focused_conversation` is the last §11 read still derived in process. bl-13f9 migrated the whole inspector *tab* family — transcript, steps, step detail, files (listing and preview in one ask), rail, inbox — and spelled config-frozen-at as `Query::Governing`, and it deliberately left this one, because it is not more of the same three moves.

`Query::Agent` exists and `answer::agent::agent` answers it. What is missing is a ruling about what a seat paints between the click and the answer, because this read's consumers are frame-synchronous in a way the tab reads are not:

- **The composer's target line** names the conversation off `seat.name`. A name that read "start a conversation" for one ask period after every selection would be a regression, not a cost.
- **The §11 focus walk** unfolds a selected member's ancestors off `seat.ancestors` (`shell::focus::ancestors`). An unfold that lags the selection by an ask period breaks the visible-selection invariant — the selection lands on a row nobody can see.
- **Two act gates read it at click time**: `stop_selected` refuses unless `seat.stoppable`, and the §3.6 danger row gates on `seat.root`. A gate whose input has not landed refuses a gesture the operator just made.
- Three of the seven callers hold `&AppModel` rather than `&mut`, so the migration is also a signature cascade through `shell::{focus,delete_agent,settings}`.

This is bl-f297's dissolved class 3 — *a consumer that reads it synchronously* — reappearing at a different noun. The shapes to weigh (none of them chosen here):

1. **A latch, as the marks pane's `Read current` got** (bl-f297) — the click throws it, the surface declares while it paints. Answers the gates; does not answer the composer's name or the ancestors' unfold, both of which are wanted on the frame the selection changes.
2. **The seat holds its last answer across the selection change**, painting the previous conversation's facts under the new one's name for a round trip. Refused on sight unless something says otherwise: a surface that lies for 500ms is worse than one that is blank.
3. **The selection's own facts are not a wire read at all** — the seat keeps the id and the workspace (which it already does, §13.1) and asks only for what it cannot know. That would mean splitting `AgentView` by *who needs it when*, which is a payload question like bl-44e9's, not a read-path one.

REMOTE §9.7 carries the residual entry and the reasoning above, verbatim, under bl-13f9's landing.

Note the ordering constraint: bl-7407 (name-keyed focus) touches the same seat and was filed first with its own ruling. Whichever runs second reconciles.