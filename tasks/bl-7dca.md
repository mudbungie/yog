+++
title = "DESIGN still describes a face it no longer has: ~200 sentences outside the retired §11"
created = 1788071155
updated = 1788414332
claimant = "Spellbind-J"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
bl-7942 rewrote §0, retired §11 behind a tombstone and re-derived §12's module
map over the shrunken tree, and amended the paragraphs that asserted a window
as a live face in this crate (§8.5's two-face pair, the one-engine-two-faces
ruling, the wire refusal, the stop mechanism). What it did not do is sweep the
remaining prose: roughly 200 sentences elsewhere in DESIGN still say *the
window*, *the frame*, *a paint*, *a click*.

They are not wrong about the FACT each states — a fact yog derives is still
derived — but they name a face this crate does not have, and a reader has to
apply a rule instead of reading a sentence.

The rule is at §11's tombstone and is the interim answer:

> Where this document says *the window*, *the frame*, *a paint*, *a click* or
> *a seat renders*, it is describing **a seat**, on the far side of the wire.
> yog states the fact; how it is shown is not yog's. […] Anything this document
> says yog **paints** is a fact it **answers**. If no §8.5 query or reply
> carries that fact, the fact is not reachable and that is a defect to file,
> not a face to add.

The sweep's real product is the last clause. Reading each sentence against
"which query or reply carries this?" is what finds the facts that had no
boundary spelling and were only ever reachable in process — the pin act
(bl-b986) is one such, found exactly that way while the tests were being
retargeted. Expect others.

Not urgent, and deliberately not folded into the severance: it is a reading
pass over a 7,500-line document, and doing it inside the ball that was
deleting 60,000 lines of code would have made both unreviewable.