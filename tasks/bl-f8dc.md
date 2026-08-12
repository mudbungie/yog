+++
title = "S5 brazen Apply beats are stale: §9.1's Apply/Reload moved inside the collapsed 'raw config.toml' fold (bl-2622), so (290,197) clicks nothing"
created = 1786511802
updated = 1786512126
claimant = "Plumb"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Measured on the bl-00ee drive (release of work/bl-00ee, `make drive DRIVE_RUNS=run-s5s8`, scratch world). With the §9.2 birth gate retired the run goes from **10 red to 2 red**; these two are the residue, and they are a *different* cause from the gate.

    S5 brazen: Apply lands (the bracket for the negatives)  FAIL — marker not on disk
    S5-T4 hash-guard: reload then the same Apply lands      FAIL — marker not on disk

`scripts/drive/beats_s5.sh:183` and `:204-207` click the §9.1 brazen editor's Apply at **(290,197)** and Reload at **(340,197)**, and type the draft after a click at (400,140). The file already flags them: *"Re-measure warning (bl-1ca2) … The numbers here are from before that reseat and are owed a re-baseline."*

The reseat is bigger than a y-offset. bl-2622 turned §9.1 into §9.5 controls: `src/shell/config_edit/mod.rs:224` `brazen_editor` now paints the effective provider table read-only, then folds the raw TOML draft **and its Apply / Reload / Effective buttons** inside a collapsed `egui::CollapsingHeader` (`mod.rs:232`, "raw config.toml — validated by bz before it lands"). So at y=197 there is a provider *row* (`openai-responses` in the screenshot), not a button — the click lands on nothing, the retype goes to whatever holds focus, and the run drifts off the Config tab entirely (evidence: `s5-03-config.png` shows the Config tab with the fold shut; `s5-04-applied.png` shows the Conversation tab with the marker text in the composer).

The repair is a fixture repair, not a code one: expand the raw fold first (it is a `CollapsingHeader` — one click on its own row, or reach it by Tab), then re-measure Apply/Reload/text-box inside it. The three S5-T3/T4 assertions themselves are still the right assertions.

Sibling stale-beat balls, same class: bl-afa7 (S4-T1 `w`), bl-2d45 (S6 stop), bl-52c7 (S7-T5 tree coordinates).