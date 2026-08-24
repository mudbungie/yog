+++
title = "the provider rung guards only the start pane: the docked composer and the empty-world bootstrap box fire the same doomed goal"
created = 1787549227
updated = 1787549227
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## What landed, and what it did not reach

bl-1fd0 put the §8.1 provider rung in front of the **start pane's** goal box:
on a wall whose brazen table carries no credential, the pane docks a band with
the reason and the §8.3 sign-in roster above the box, and Send refuses.

That is the seat the ruling named. It is not the seat the live incident most
likely used. Two other boxes fire a start with the identical outcome and neither
is guarded:

- **The docked message composer** (`shell::input_bar`). Its Enter on a workspace
  with no conversation selected is §3.4's **bare rung** — a full start, straight
  to `lernie prompt`. This is the ordinary way a first goal is typed.
- **The empty-world bootstrap box** (`shell::bootstrap`). The very first thing a
  stranger sees. Its Enter founds `home` and starts a conversation in it, on a
  wall that by construction has nothing signed in.

Both spend the operator's typed goal on a conversation that dies on no-models,
which is the whole of what the ruling was about.

## What it should cost

Nothing new is needed. `crate::start::StartGate` is the tested decision and
`shell::start_login` is the paint; the gate reads §3.4's `start_workspace`,
which is the sphere both of those boxes aim at as well. The open questions are
seating, not mechanism:

- The start pane's band is conditional on `state.start.pending`. Widening it to
  `goal` (the pane's existing `composer_open || pending.is_some()`) would cover
  the docked composer with no second band and no second rule.
- The bootstrap box is its own surface outside the pane's accessory stack, so it
  needs a seat of its own or a shared one.
- **Noise is the real design question.** The start pane's band appears once, at
  the moment a goal is about to be spent. A band over the docked composer stands
  for as long as the workspace is unsigned, beside an auth-failed banner that
  already says the same thing after the fact. Decide whether the composer's arm
  is the band, a one-line refusal on Send alone, or nothing.

## Boundaries

Do not touch `StartGate`'s predicate — that reasoning is settled in DESIGN §8.1
(a keyless row is not a credential; every other spelling readies the wall). This
ball is about which boxes ask it.