+++
title = "the conversation-name mint is seeded from a one-second wall clock, so two conversations started in the same second are minted the same name and every seat verb then addresses neither"
created = 1788675709
updated = 1788675709
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Routed from litany bl-8fe8 (round-1 usability campaign, fix lane L3). The repro and the cost are that ball's; the mechanism is here, and litany's own mint is not at fault.

MECHANISM

`src/boundary/dispatch/doors.rs`, the §8.1 fire door, the one place a conversation name is minted:

    &SplitMix64::from_seed(
        seed.unwrap_or_else(|| crate::ui_state::content_hash(ts.as_bytes())),
    ),

`ts` is `Clock::stamp()`, which `src/ui_state/clock.rs` defines as **unix seconds as a string** ("the wall-clock ops.jsonl timestamp (§4.2) — unix seconds as a string, the crate's timestamp convention"). So the seed has one value per second per box.

`litany::mint::mint` is a pure function of one RNG draw and the occupied set: same draw, same occupied set, same name. Two fires inside one second hand it the same draw — and the occupied set is equal too, because neither fire has landed a dispatch commit yet, so `answer::names_in` reads the same living names for both. Three `lernie start` calls in one shell line were all minted `ScarfPeach`; two admin tasks a second apart in an ordinary session both came back `MeadowGelato`.

WHAT IT COSTS

The name is the handle every seat verb takes — `lernie transcript <ws> <name>`, `follow`, `message`, `stop`, `agent`. Two live conversations under one name and the engine refuses, honestly and uselessly:

    {"error":"ambiguous conversation \"ScarfPeach\"","ok":false}

The escape is the raw agent id, which the start reply does not carry; it has to be recovered from `lernie conversations` by matching the goal preview.

WHY IT IS NOT LITANY'S

litany's own creation paths seed `SplitMix64::from_entropy` — nanoseconds XOR the pid, a grain finer than a creation — so `litany prompt`/`litany dispatch`/the `dispatch` tool are unaffected. yog does not use that path: it mints the name itself at the fire and passes `--name <minted>` to `litany prompt`, so the seed it chooses is the whole of the guarantee. litany bl-8fe8 landed the contract in words and a beat — ARCHITECTURE §2.3 and `src/workspace/agent_name/mint.rs` now state that a consumer injecting its own `Rng` owes it PER-CREATION entropy, and `two_generators_seeded_from_one_wall_clock_second_mint_one_name` pins the consequence — but litany cannot enforce it on a consumer that supplies the generator, and `require_available` cannot save it either: every racing creator scans before any of them commits a `name` blob, so they all see the name free.

EXPECTED

The default seed at the door is per-creation, not per-second. `seed: None` is a caller that made no prediction — a deposited line, the §4.3 loop — and this moment's stamp is standing in for a draw; it needs to be an actual draw. `SplitMix64::from_entropy()` is already linked and is exactly this (litany `mint.rs`: nanos XOR pid<<32), so the smallest honest change is to stop deriving a seed from `ts` at all on the `None` arm and let the entropy path do it. The `Some(seed)` arm is untouched: a firing seat that predicted a name still gets the name it predicted (bl-1747).

A test that fails on today's tree: two `prompt` fires against one workspace with the same `ts` and the same occupied set must mint two names.

RESIDUAL, worth a second look but not this ball's

Even with per-creation entropy the mint is a 1-in-292,140 collision per pair of simultaneous fires, because `answer::names_in` cannot see a conversation that has not landed its dispatch commit. That is litany's documented race and it is tiny; the per-second seed made it a certainty, which is the whole of what is being fixed here.