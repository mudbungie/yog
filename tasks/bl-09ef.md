+++
title = "the §6 desktop escalation derives on the engine and reaches nothing: src/alert has no caller and no boundary spelling"
created = 1788414430
updated = 1788414937
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["boundary"]
+++
DESIGN §6, "The strip escalates to the desktop when the window is buried (bl-e160)".

Found by the §11-rule sweep (bl-7dca): reading the passage against *which
§8.5 query or reply carries this?* answers **none**.

`src/alert/{mod,send}.rs` projects a `boundary::answer::queue::QueueRow` into
an `Alert` (summary + body) and spawns `notify-send`. Nothing in `src/` calls
either half outside its own tests: the fold ran on the frame, gated on the
window's focus and on the §4.1 `notify_unfocused` knob, and the frame is the
seat crate's since bl-7942. So the escalation is derived and dropped.

The fact has no carrier. `Query::Attention` answers the queue rows, but an
*arrival* — the set difference against what a seat last saw, which is the whole
of what an alert is — is baselined per seat in RAM that no longer exists here,
and nothing tells a seat "this row is new".

Three shapes to weigh, not a foregone answer:
- the arrival is the seat's to compute (it already asks `/attention` on a
  standing set; the baseline is one set difference) and `src/alert` + the
  §4.1 `notify_unfocused` knob move to the seat crate;
- a follow-class read (`Query::AttentionLane`, REMOTE §5) that frames only
  arrivals, so every seat gets the same sentence from one home
  (`AttentionKind::says`);
- yog keeps the spawn and the notification is the *engine box's* desktop,
  which is wrong wherever the seat is not on that box.

Whichever wins, DESIGN §6 and §4.1's `notify_unfocused` row must be amended to
match; today they describe a window.

---

Narrowed by bl-09aa, which landed the attention lane (REMOTE §14.1, `src/boundary/attend.rs`) while this ball was being written. `Query::Attention` is now answerable as a frame SEQUENCE — the answer as of connect, then a frame whenever the answer under that asker's scope changes, with row ages zeroed for the comparison so a clock reading never reads as news. That IS the arrival signal an alert is a difference against, so the second of this ball's three candidate shapes is already half-built and a seat's baseline is one set operation over frames it is handed rather than a poll it runs. What remains open is only the announcing: whose desktop hears it. If the answer is the seat's, `src/alert/` and the §4.1 `notify_unfocused` key move to the seat crate and this crate keeps `AttentionKind::says` as the one wording; if it is the engine box's, `alert::send` gains a caller and the knob gains a carrier (bl-f936 is the ball for the pane document that holds it). DESIGN §6 and §12's `src/alert` row now say exactly this.
