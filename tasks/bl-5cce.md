+++
title = "the drive harness's other eight pinned pixels: beats_s6/s7/s8/s3res still measure clicks against a screenshot, the class bl-b9f2 retired for §9.1"
created = 1786601146
updated = 1786601366
claimant = "Trestle"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
bl-b9f2 retired the §9.1 pixels by DERIVING them per run (`scripts/drive/locate.sh`
finds the `ui.separator()` that ends brazen's Config pane in the run's own
screenshot and folds every §9.1 point off it). STORIES.md now carries the rule
outright: **"A surviving coordinate is DERIVED, not pinned."**

Eight clicks in the harness still predate that rule and are pinned numbers
measured against some earlier day's screenshot:

    scripts/drive/beats_s3res.sh:158   click 37 703
    scripts/drive/beats_s3s4s6.sh:249  click 37 703
    scripts/drive/beats_s6.sh:80       click 1131 12
    scripts/drive/beats_s6.sh:81       click 1136 38
    scripts/drive/beats_s7.sh:177      click 324 158
    scripts/drive/beats_s7.sh:178      click 344 179
    scripts/drive/beats_s7.sh:184      click 275 158
    scripts/drive/beats_s8.sh:35       click 308 511

Each is the same standing regression source bl-2622 → bl-f8dc → bl-b9f2 paid for
three times in three weeks: a row inserted above the target silently retargets
it, and nobody learns until the beat has been red for a day.

**Not necessarily one fix per click.** Each should be re-asked in the order
STORIES.md's steering rule sets out before any of them is re-derived:

1. Does the gesture have a §11 key or a §8.5 line now? (bl-3f46 and bl-ec8f
   retired a dozen coordinates that way; some of these may already be dead.)
2. Is it a VIEW at all? If not it is a keyboard gap, not a pixel.
3. Only then, derive it — the anchor must be a structure that moves with the
   layout, the way `locate.sh` uses a pane's own separator, never a re-measure.

`locate.sh` is a `brazen` verb over one anchor rule; whether the other surfaces
share that anchor or want their own is part of the work, not assumed.