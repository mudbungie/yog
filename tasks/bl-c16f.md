+++
title = "the mark's thinking hue: gate violet is the dimmest in the palette and matches the wordmark beside it — sigil magenta"
created = 1785736866
updated = 1785736866
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ruling, 2026-08-02 (this session). The §11 live mark landed (bl-b768)
with `Doing::Thinking` on **gate violet**. Two things are wrong with it, one
perceptual and one semantic, and the perceptual one is decisive.

## Measured

Everything on the mark goes through `icon::deep` at the phosphor drive, so on a
~3px node circle only hue angle and brightness survive. Driven, against a void
of luminance 17:

| state | hue° | driven | luminance |
|---|---|---|---|
| idle | green 140 | `(32,255,108)` | 197 |
| waiting | orange 37 | `(255,167,25)` | 175 |
| inference | blue 203 | `(34,172,255)` | 149 |
| tools | red 354 | `(255,30,52)` | 79 |
| **thinking** | **violet 264** | `(123,32,255)` | **67** |

Gate violet is the **dimmest hue in the palette** — and it was carrying the
state the operator most wanted to see. Sigil magenta `(255,36,231)` is the same
perceptual family (still reads "purple"), +45% luminance, and its stated job in
the palette is already "never blends into a definite state's hue".

## The seat argument

The mark sits immediately left of the "yog" wordmark, which is painted in gate
violet. A thinking circle was therefore the same hue as the brand text two
pixels away — signal indistinguishable from decoration.

## Why tools keeps ichor red (the alternative, rejected)

With thinking on magenta, gate violet frees up, so tools-on-violet was the
obvious next question. Measured with CIE Lab ΔE over the whole five-set:

- **red set** (green/orange/blue/magenta/**red**): min ΔE **65**
- violet set (green/orange/blue/magenta/**violet**): min ΔE **49**

The tight pair differs, and that is the whole answer. Red↔orange are close in
hue but 79 vs 175 luminance — a 2.2x brightness gap separates them where hue
does not. Violet↔magenta are close in hue *and* in luminance (67 vs 97, 1.4x),
so nothing rescues them. The measure (ΔE76) overstates differences in the
blue/violet region, i.e. it is biased *toward* the rejected set, which loses
anyway.

And the degrading pair is the wrong one: in the violet set it is
**thinking↔tools** — the two states an operator most needs apart, one meaning
"pondering" and the other "touching the repo right now". In the red set it is
waiting↔tools, both "busy, be patient", where a mix-up costs nothing.

Also rejected: **idle→ash grey** to free hydra green for tools (which would
have made the mark agree with `flight_badge`'s green tools). Rest would stop
being the logo — a grey mark whenever nothing is running reads as broken, and
"the empty state *is* the shipped icon, byte-identical" is the best structural
property the design has.

## Change

One hue. `theme::doing_badge` `Doing::Thinking` gate → sigil; the borrowed-hue
note in that mapping and in DESIGN §11 re-stated (the borrows are now ichor for
tools and sigil for thinking); the mark tests' expected hue; README's word.

Final set: green nothing · orange waiting · **magenta** thinking · blue
inference · red tools.