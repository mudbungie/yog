+++
title = "V1 Historian: the step spine is a commit spine — history rail, notch-pinned inspector"
created = 1785719120
updated = 1785719120
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
VERIFY docs/VISION.md and docs/DESIGN.md on main before editing — ball bodies drift; where this body and VISION disagree, VISION wins. VISION §5 V1, sequencing tier 1 (no dependencies). Pure read of the workspace repo — refs, trees, commits; no new verb anywhere. Transcript grows a history rail (one notch per step = the step's read-state commit from meta.json); selecting a notch pins the whole inspector to that commit (transcript/files/config-frozen-at/budget as-of); descent edges render on the rail. Rail collapses to today's transcript for anyone who never clicks a notch. Graduates into STORIES.md as S10 with its tests.