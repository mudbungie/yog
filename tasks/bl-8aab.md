+++
title = "the control boundary (VISION §4.8): typed action/query surface, one chokepoint, headless entrypoint, parity tests"
created = 1785648911
updated = 1785649207
claimant = "Glamour"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
VERIFY docs/VISION.md §4.8 on main before editing — it carries the operator rulings this ball implements; where this body and VISION disagree, VISION wins.

A declared interface rewrite. The boundary carries ACTIONS and QUERIES; VIEWS never cross it. Classification rule (operator, verbatim): 'Focusing an input box doesn't go through it, but hitting enter does; switching tab doesn't, but changing a value does, and so too did the call that populated the contents of the tab.'

Deliverables:
1. The typed surface: action and query variants carrying their parameters — a datum, not a convention. The GUI's click-glue constructs variants; one chokepoint dispatches. A new gesture without a headless spelling must fail to COMPILE (exhaustive match), not fail review. Views stay exactly I6's closed RAM whitelist and gain no representation here.
2. The headless entrypoint: same binary, no window. Transport design decision inside this ball — the operator's instinct is deposit-based (a gesture as a create-only file into a yog-watched inbox: fits I4, audit = the deposit + ops.jsonl, I0 convergence free); an argv verb may exist as deposit-and-wait sugar. Do NOT let the GUI keep calling functions directly while headless goes through deposits — one surface, two serializations, never two implementations (VISION §8).
3. Parity/round-trip tests: every variant round-trips its serialization; queries return the same typed data both frontends render (the snapshot derivation run without a frame).
4. DESIGN.md amendment IN THIS DELIVERY: the §8 hatches become the full surface; the boundary taxonomy recorded (views=I6 whitelist, queries=I1 derivations, actions=ops rows). VISION §4.8 is the source.
Out of scope, follow-ons: V5 teleop rung proper; migrating scripts/drive off pixel-steering onto the boundary.