# Interface parity — the seat/mobile drift gate (bl-80c1)

**Operator requirement (2026-09-01):** the desktop seat and the android client
must have interaction parity — NOT identical placement, but if something is
interactable in one client it must exist in the other, and drift between the
two surfaces must be caught mechanically going forward, not noticed by hand.

This document is that requirement made structural. It is the authority for the
parity contract: what is judged, against what, by which instrument, and how a
deliberate absence is recorded. The implementation lands by the balls named in
§9; when reality diverges from this doc, one of them is a bug — amend the doc,
never code around it.

## 1. The subject is operations, and that is §8.5's own line

The contract's subject is exactly what crosses the §8.5 control boundary:
actions and queries — the ops trail's rows and the I1 derivations that
populate. An interactable that crosses no wire is a **view** (focus, scroll,
selection, copy, collapse, theme), and §8.5 already rules views out: *"focusing
an input box does not cross it; switching tab does not."* The drift the
operator described is capability drift, and a capability that moves or reads
the world is an op by definition — a pure view moving nothing is an ergonomic
question, not a parity one, and it has no single authority to be judged
against; building one would be a hand-maintained index (§8 of this doc says
what catches view-class defects instead).

## 2. Each client against the roster — but the roster had to grow one fact

**Pairwise comparison is rejected.** Components meet only at the interface,
never pairwise (house architectural rule): a client-vs-client diff has no
authority when the two disagree, drifts with whichever client updated last,
and goes quadratic the moment a third surface exists (a TUI, a web seat). The
one authority for "things a client can do" already exists and is already
machine-consumed:

- the **enumeration** is the conformance corpus's request half —
  `corpus/request/<op>.json`, generated from the codec's own exhaustive
  surface walk (`boundary::corpus`, bl-32cb), committed, versioned per shape,
  and **vendored by every client today**. The help table is gated both ways
  against the codec (`boundary::help::tests`), so help rows, codec ops and
  corpus request shapes are one list seen three times.

But bare roster *coverage* — each client free to record any op as locally
absent — does **not** deliver the requirement. The failure it leaves open is
exactly the drift the operator described: one client surfaces `Floor`, the
other records "outside my slice", both stay green, and the surfaces have
diverged with no alarm. The sentence "if it is interactable in one it must
exist in the other" makes *which ops owe every seat a surface* a fact of the
**suite**, not of a client — and a suite fact gets one home, at the component
every client meets: yog's help table.

**So the roster grows one field, and no second list.** `HelpRow` gains a
classification — landed as `surface` (bl-8758), two values:

- **`control`** — every seat-class client owes this op a *discoverable
  interactable* (a tagged accessibility node, §4). Queries included: the owed
  interactable for a read is the affordance that reaches the view it
  populates.
- **`machine`** — spoken by programs, never owed a control: the foot's verbs
  and the routing leg's machine ends. Coherence is testable: every gesture
  `Grade::Foot` admits (`registry::peer`) must be classed `machine`.

The field is compile-enforced — a new verb cannot exist without stating its
class — and it publishes itself: the corpus's `reply/help.json` fixture is
generated from the table, so the classification ships inside the artifact
clients already vendor and replay. Two values only, deliberately; a third
class is added when a row honestly needs one, not speculatively.

**Raising the bar is how a client surfaces something new.** A client wanting a
control for an op classed `machine` — or an op not yet in the roster — changes
the classification (or the roster) at yog first. That one edit regenerates the
corpus, and on the next vendor refresh the *other* client's parity gate
reddens until it grows the control or records an exemption. The alarm fires at
the moment of divergence, from one home, with no client ever reading another
client's tree.

## 3. Why the wire's op token is the vocabulary, not the `Action` variant

yog's `Action` enum folds families (`Ball`, `Monitor`, `Fan`, `Route`, `Tune`)
to one variant over a family `Verb`; the wire spells each member as its own
`op` (`close`, `arm`, `disband`, `capture`, `effort`, …). Clients mirror op
strings, not Rust variants — the corpus buckets by op, the help table keys by
op, and an op is what a control fires. So the parity ledger's unit is the **op
token** (`HelpRow::verb` == the envelope's `op`), and a folded family is
tagged per member, never per variant.

## 4. The tag: `act:<op>`

The control that fires op `V` carries the literal token **`act:V`** in its
accessibility node — AccessKit metadata on the desktop seat, the exported
accessibility tree's node text on android (§6). The visible label stays a
human word; the tag is machine metadata, and the op token is the one name that
already exists everywhere (help row, envelope, corpus filename), so no
translation table is born.

The `act:` namespace is reserved and judged in both directions: every
`control`-classed op must appear as a tag somewhere in the client's walked
inventory, and every `act:` tag found must name a corpus op — a typo'd or
stale tag fails, exactly as a leak-fixture line that stops matching fails.

## 5. The meta-test, per client

Each client repo runs **one standing assertion**, in its own gate, consuming
its own inventory instrument and its own vendored corpus — nothing crosses
repos at test time:

    roster     = vendored corpus reply/help.json rows classed `control`
    inventory  = the set of act: tags found across the harness's
                 named-screen walk
    exemptions = the client's parity exemption file (§7)

    assert roster − exemptions ⊆ inventory      (coverage)
    assert tags(inventory) ⊆ ops(corpus)        (no unknown tag)
    assert ∀e ∈ exemptions: e ∈ roster          (no rotted exemption)
    assert ∀e ∈ exemptions: e ∉ inventory       (no stale exemption)

The inventory instrument is the sibling harness's, never a third:

- **lernie** — the kittest/AccessKit harness (lernie bl-dc07): the walk over
  the seat's screens collects tags off the real AccessKit tree.
- **yog-android** — the emulator accessibility dumps beside the screencaps
  (yog-android bl-243b), once the egui tree is exported to the platform
  accessibility layer (§6).

**Presence is the claim; depth is the harness's.** Parity asserts a tagged
node exists in the walked tree. Reachability in bounded gestures, nothing
clipped off-screen, no blank regions — those are the sibling harnesses' own
standing assertions and stay theirs. And **unproven is red**: a control that
exists only on a screen the walk never visits fails honestly — extend the
walk or move the control; the walk's screen set is part of the instrument.

## 6. The android instrument gap, named

As of this design the android seat renders into one wgpu surface and eframe is
built without its accesskit feature, so a uiautomator dump of a seat screen
yields **one opaque node** — bl-243b's dump instrument sees the outer chrome
and nothing of the egui tree. The parity test cannot stand until the tree is
exported. Two honest routes, decided in the android implementation ball:

- **preferred:** enable eframe's `accesskit` feature so egui's accessibility
  tree reaches the platform layer and the dump instrument reads it — a
  feature toggle on an existing dependency, but it grows the lockfile, so it
  needs the standing dependency ruling; it also buys the app real
  accessibility, which is worth having on its own.
- **fallback:** a debug-gated in-process inventory — the shell serializes the
  act-tags it painted this frame to a file the harness reads. No new
  dependency; weaker, because it is self-reported rather than observed.

Either way the assertion in §5 is unchanged — only where the inventory bytes
come from differs.

## 7. Exemptions: config, cited, loud, two-directional

A deliberate absence lives in a committed config file in the client's own repo
(working name `parity.toml`): one line per op, each carrying a reason that
**cites a ruling or a ball id**. Two kinds occur in practice:

- *unbuilt* — the surface is intended and not yet built; the citation is the
  ball that will build it. This is a loud TODO ledger, machine-checked.
- *never on this platform* — an operator ruling that the thing should not
  exist here; the citation is the ruling's ball.

Severability is the house test: deleting the line re-reddens the gate, and no
code changes. The test prints the exemption roster on every run — an absence
is never silent. And §5's last two assertions keep the file honest: an
exemption for an op that is now surfaced fails (stale), as does one naming an
op the roster no longer carries (rotted).

**The initial state will be mostly exemptions, and that is the deliverable.**
At adoption the desktop seat surfaces a handful of ops as controls and the
android client a similar handful; the rest of the `control` class lands in
each client's exemption file with its citation. The parity gate's first
service is making today's divergence enumerable and loud; the android repo's
hand-kept parity ledger (its DESIGN §13.4) folds into its exemption file —
one home, machine-checked, instead of prose.

## 8. Out of contract, each with its reason

- **Views** — §1. What catches view-class defects is the sibling harnesses'
  own quality assertions (reachability, clipping, blank regions), per client.
- **Placement, keybindings, gesture count** — the ruling's own words: parity
  of existence, never of placement.
- **Client sugar** — a button composing several roster ops is convenience
  over capabilities the other client already owes one by one; sugar earns no
  ledger row.
- **A wire-crossing client-only feature** — impossible by construction: the
  wire refuses an unknown op, so anything a control fires is already in the
  roster.
- **Behavior** — parity claims presence, not function; a tag on a dead button
  passes it. The rung above (drive the tagged node against the fixture
  endpoint and assert the emitted envelope's `op` equals the tag) is cheap in
  the kittest harness and is filed as a later rung inside the client ball,
  not a gate at adoption.

## 9. Drift cadence, end to end

1. A verb lands in yog. The classification field is compile-required; the
   corpus gate (`store::check`, an ordinary test) refuses an unregenerated
   corpus — a new op cannot land unclassified or unpublished.
2. A client refreshes its vendored corpus when it moves its yog pin — and it
   cannot ride a newer yog without refreshing, because the PROTOCOL handshake
   refuses a mismatch fail-closed.
3. On refresh, the client's parity assertion reddens for every new
   `control`-classed op until a decision is recorded: a tagged control, or a
   cited exemption line. The client's conformance decision table (the
   codec-level gate the android client already runs) reddens beside it for
   the spelling; the two gates answer different questions and neither
   substitutes for the other.

## 10. What lands where

- **yog** — the classification field on `HelpRow`, every row classed, the
  coherence test against `Grade::admits`, corpus regenerated. **Landed
  (bl-8758)**: `HelpRow::surface`, every row classed, with exactly **five
  `machine`** rows and everything else `control` — the roster's own length is
  a count nobody should restate, and this paragraph carried two that went stale
  the next time a verb landed (bl-c285's `/login` and `/login-tail`). Ask
  `reply/help.json`, which is generated. The five machine rows are the routing
  leg's own ends — `advertise`, `complete` and
  `invocations` (the three `Grade::Foot` admits, coherence-tested) plus
  `invoke` and `capture`, the asking program's fire and poll. Three calls the
  roster made and their reasons, so a later reader is not re-litigating them:
  **`deliver` is `control`** — accepting one of `n` fanned candidates is an
  operator judgement and the sibling of `/fan` and `/retire`, which no reading
  makes machine; §2's `machine` is the foot's verbs and the routing leg's
  machine ends, and `deliver` is neither. **`enroll` is `control`** — §1.4's
  mint is an act at a seat, pairing a device that is not yet a peer. And
  **yog's argv table is outside the ledger** — `yog env`, `yog exec`, `yog bl`
  and the rest share the row type for its text but cross no §8.5 boundary, so
  no seat can owe them a control and the generated roster never sees them.
  Wire-visible: `reply/help` rows gained a field, so `PROTOCOL` is **7**
  (REMOTE §9.14).
- **lernie** — `act:` tags on the seat's verb controls, the §5 assertion in
  the bl-dc07 kittest harness's gate, the exemption file (bl-38d4 in the
  lernie store).
- **yog-android** — the accessibility-tree export (§6, with its dependency
  ruling), `act:` tags, the §5 assertion over the bl-243b dumps, the
  exemption file folding DESIGN §13.4's ledger (bl-fe4c in the android
  store).
