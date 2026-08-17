+++
title = "drive harness decay: the S5 collapse and S8-T4 no-write beats still read ui.json for the collapsed set, which bl-8bbc moved to the per-seat pane document"
created = 1786936114
updated = 1786936115
claimant = "Dills"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
run-s5s8 fails two beats on current main while every neighbouring beat passes: 'S5 fixture: balls collapsed (ui.json) — no collapse record' and 'S8-T4 marks: no yog-owned file written — ui.json moved or absent'. Both are the harness, not yog (the memory pattern: a red beat after a cutover is usually the beat that drifted). REMOTE §7 as landed (bl-8bbc): the world document keeps <state-root>/ui.json (seen, pinned, identity_last_used, ceiling, prices — beats_s6's uses are correct), while pane-of-glass facts (panels, COLLAPSED, zoom) moved to <state-root>/clients/<client>/pane.json, the window's client being yog-window. So the S5 collapse lands in clients/yog-window/pane.json and ui.json is never created in this run at all — which also voids S8-T4's negative, whose non-vacuity rule (bl-f16e) requires the compared file to exist. Fix: point the collapse assertions at the pane document; make the S8-T4 negative's existing-file subject the pane doc and compare ui.json by md5of's absent-stable string beside it. Verify by rerunning the s5s8 verb.