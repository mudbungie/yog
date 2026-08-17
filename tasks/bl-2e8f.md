+++
title = "a fired start is not selected until its branch lands: the echo mints a row nothing focuses, so the operator's own new chat sits unhighlighted behind the birth placeholder"
created = 1786936785
updated = 1786936812
claimant = "Sift"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator report: "you start a new chat, start typing, and the new chat is not immediately selected."

## What happens

`new conversation` (button, or `n`) clears the agent selection: `shell/keys.rs:197
new_conversation` -> `focus::workspace`, so `focus.agent = None`. The operator
types and hits Enter. The fire mints a §3.3 name and holds the §3.4 claim by it:

    src/app/focus.rs:79  await_conversation(&mut self, ws, conversation, goal)
        self.started = Some(Echo::started(ws, conversation, goal, now));

That claim is spent ONLY once the detached driver has written `agents/<id>` and
a derivation has read it back:

    src/app/focus.rs:114 adopt_started
        if let Some(agent) = echo.resolved(&self.derived) {
            echo.target = Target::Agent(agent.clone());
            self.focus_agent(&echo.ws, &agent);
        }

Between Enter and that write, `focus.agent` stays `None`, so:

- the §11 list highlights nothing — `shell/conv_row.rs:59` takes `selected:
  model.focused_agent_id()` and `:118` compares it to `row.root_id`;
- the center is still the birth block and the `select a conversation`
  placeholder, which `src/shell/acceptance/started.rs` documents as "where Enter
  leaves the operator today".

The row itself is already there and is already addressable. `app/echo/rows.rs
with_echo` LEADS the list with a synthetic row whose `root_id` is the minted
name (`app/echo.rs pending_conversation` -> `agent_id: name`), and
`shell/convs.rs forest` folds it in ahead of every reader — the paint, the up/down
walk, the left page. So the one thing missing is the focus: nothing selects the
row §3.4 exists to mint.

## Shape of the fix (to be attacked before committing)

Focus the minted name at fire time — the pending row is a real `ConvRow` in the
echoed forest, so `nav::convs::selection` resolves it and the composer aims at
it by name, exactly as it does one ask later. `adopt_started` then moves the
focus from the name to the id when the branch lands; that is one value changing
identity, which is what the echo already models.

Open questions the claim must answer, not assume:

1. A second Enter while the claim is unresolved would be a §8.2 `message` aimed
   at a name lernie has no agent for. What gates it today, and what should?
   (`nav::convs::selection` / the composer send predicate.)
2. `Query::Agent` has no answer for the pending name, so the detail family (§9.4
   model row, §6 marks, Nudge) paints nothing for a frame or more. bl-48ae's
   rendering ruling says a fact that only paints may land late, so this is
   expected — confirm it degrades to blank rather than to a refusal.
3. `focus_agent` writes the §6 `seen` acknowledgement keyed by the agent id. For
   the minted name that records an ack against an id that will never exist. Is
   that a durable-garbage leak in `ui.json`, and does it want the name-keyed
   write skipped?

## Acceptance

`src/shell/acceptance/started.rs` asserts the post-adoption frame. The missing
beat is the frame BEFORE adoption: with the claim held and the branch not
written, the started conversation is the highlighted row and its (empty)
transcript is the center — not the placeholder.