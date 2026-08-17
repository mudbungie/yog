+++
title = "the fullness denominator is dark until hand-declared: if it should prefer the window brazen served, the shape is a read-time query over the model cache"
created = 1786937639
updated = 1786937639
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design", "context", "provider"]
+++
A consequence of bl-d9cb, filed so the tradeoff is tracked rather than rediscovered. Not a defect: the state below is the honest one, and this ball exists to decide whether a better one is worth its mechanism.

## What changed

The §9.4 picker used to write a `models.yaml` entry per pick, seeding `context_window` from the roster it had just read (bl-848f) and falling back to `DEFAULT_CONTEXT_WINDOW` (200_000) where the provider served none. bl-d9cb deleted that write — it landed in a table lernie retired — and did not relocate the seed. So no gesture creates a window declaration any more. The only writer is §9.2's Declare control, by hand.

**Consequence:** §5.1 #35's fullness figure is absent on a fresh world until the operator declares a window. `context::of_conversation` already answers `None` for an undeclared or zero window and DESIGN calls that the no-capability-theater rule, so the figure is *correct* — it is just usually not there. Two things were traded away and both were worth trading:

- the 200_000 default, which was right for Anthropic by coincidence, wrong for OpenAI, and off by ~64x for the Ollama turn bl-671d measured (server default 4095 against a declared 200000);
- the served seed, which only ever fired for Google — brazen publishes `Model.context_window` for the providers whose list GET serves it and `None` for Anthropic, OpenAI and Ollama.

## The shape if the figure should come back

A **query at read time**, never a field seeded at write time. The material is already on disk: `config_edit::brazen::model_cache_at(&BrazenPaths::in_wall(<wall>).models_cache_dir, provider, io)` reads the roster document `bz --list-models` wholesale-writes inside the workspace's own wall, and `serde_json` is already a dependency. The parse is three lines (it was `query::served_window`, deleted in bl-d9cb — recover it from that commit rather than rewriting it).

Why a query and not a field: the window is the provider's fact and moves without yog's involvement, so a copy in a config file is a snapshot that goes stale. DESIGN §9.4 already applies exactly this reasoning to the candidate roster — *"a stored candidate list would be a second representation of a fact the provider owns"*.

## What makes it non-trivial, and why it was not done in bl-d9cb

The reader is `app::derive::route::adopt_windows`, which reads ONE world-global file on the boot and the 15 s full sweep, and hands `Snapshot::windows` (wire-model-id -> tokens) to a pure filter. A served-window query is **per workspace and per provider row**, and the gauge keys on the wire model id off a `StepBill` — so it needs the step's workspace and its provider, which the bill does not carry today. Attack that first: it may fall out of the §16 walls the snapshot already enumerates, or it may want a column on the bill.

Then the precedence question, which is the real design call and has no obvious answer:

- **declaration wins** — the operator's correction is authoritative, and a served number only fills a gap. Costs: a stale hand-typed number silently beats a true one, which is bl-848f's defect back in a new place.
- **served wins** — the provider's fact is authoritative. Costs: the operator's only correction seat stops working for the providers that publish a window, and the declaration becomes a knob that does nothing on Google.
- **served, and no declaration at all** — deletes the block, the §9.2 editor over it, the §9.5 typed rows and this whole family (see the sibling ball on the dead columns). Costs: the figure is permanently dark for Anthropic, OpenAI and Ollama, which is nearly every turn.

Do not add a `bz` spawn to the derive loop to get there. `adopt_windows`'s own header rejects arming a fifth watch root over one file (*"§7.1's roots are the enumeration set, not a file list"*), and a subprocess on the sweep is strictly worse than that. The cache read is a file read and composes with the existing sweep; anything that needs a spawn is the wrong shape.

## The upstream that would dissolve part of this

brazen bl-f19d (filed by bl-671d): a first-class context declaration each dialect projects where it has a slot, or a config passthrough that merges into the typed `options` the encoder already built. That is about the number reaching the *request*, not the gauge — but if it lands, the honest denominator and the number actually in force become the same fact, and this ball should be re-read against it before anything is built.