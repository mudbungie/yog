+++
title = "the speaker label wears body ink on the body's own line — role hue never reaches the prefix"
created = 1787375806
updated = 1787375806
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator ruling 2026-08-21: the speaker label on a delivered message must be
visually distinct from the payload — its own line, in the role's hue — because
today the prefix and the body wear identical ink on one line and the eye cannot
tell speaker from output.

## Current state (verified 2026-08-21)

- `src/transcript/render.rs:164` — `paint(ui, row.tone, &row.prefix)`; a
  delivered/model text row is `Tone::Plain`, so `paint` (render.rs:234-250)
  falls to bare `ui.label` — the same default MOONLIT ink the body gets
  (render.rs:172-174, :196, :201). Same line: the whole row is one
  `ui.horizontal` (render.rs:151-174): stripe → toggle → prefix → inline preview.
- The role→hue mapping already exists and is spent only on the 3pt stripe:
  `src/theme/role.rs` — `Role{User,Model,Peer,Ended}` (:46), `role_badge`
  (:75-87: User=GATE, Model=SPECTRE, Peer=BRAZEN, Ended=BRAZEN_DIM),
  `role_stripe` (:90-101). `Row::role` is already `Some` exactly on speaking
  rows (`src/transcript/rows/project.rs` :64/:76/:88/:168/:239, `None` on
  machinery).

## The change

1. **Hue:** when `row.role` is `Some(role)`, paint the prefix with
   `theme::role_badge(role).0`; machinery rows (`Tone::Good/Bad/Live/InFlight`
   prefixes, role `None`) keep going through `paint()` untouched. No minted
   hue — the bl-3acb "no minted hue" half stays intact.
2. **Own line:** the speaker prefix moves to its own line above the payload for
   speaking rows. The stripe/toggle alignment contract (role.rs:90-101,
   render.rs:207-211) and the legible.rs:204-240 rule (any elided run must have
   its triangle on the same laid band) constrain the split — keep the toggle on
   the band of the line it folds.

## DESIGN.md must be amended in the same change — do not deviate silently

Two standing rulings contradict this and are superseded by this operator
ruling; amend both where they live:

- DESIGN.md ~6815-6825 "Density: one line per thing" — a speaking row is now
  two lines (speaker, payload); machinery rows stay one.
- DESIGN.md ~6993-7020 (bl-3acb): "ONE mechanism … a thin vertical stripe …
  no reordering, no relabeling" — the role hue now also inks the speaker label.
  Record this ball as the ruling.

## Tests

- Ink: extend `src/transcript/tests/render.rs` (e.g.
  `each_role_paints_its_stripe_and_machinery_paints_none`, :186-227) with
  label-ink assertions via `paint_probe::seen_of(...).ink` — a colored label is
  text ink, not a fill. Never assert `Galley::text()` (house rule,
  no-hand-rolled-paint-walk).
- Banding: `src/transcript/tests/legible.rs` (:204-240 triangle-band sweep,
  :247 alignment mark) will exercise the new layout.
- Prefix pins in `src/transcript/tests/rows/labels.rs` and parity.rs pair rows
  by prefix STRING — those stay valid if the prefix string is unchanged and
  only its paint seat moves.
- Acceptance `src/shell/acceptance/echo.rs:104-121` keys the pending echo on
  the user stripe FILL hue — unaffected unless the stripe changes; don't touch
  the stripe.

Line caps: render.rs sits at 250/300 — if the split projects it ≥300, pre-split
along a real seam and add the §12 row (design-time, not shaving).