+++
title = "the ten brazen provider rows paint twice in the same frame: once in the Login pane and once in the Config pane's provider table"
created = 1786163411
updated = 1786684624
claimant = "Marlin"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
QUALITY.md §1 criterion **H1** ('One fact, one rendering. A fact paints once per surface'). Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out.

SYMPTOM. With config mode open and the side panel's Login section expanded, the same ten provider rows — same names, same auth state, same explanatory hint — render simultaneously about 350px apart, in one frame.

WITNESS: `probe-s5-login.png` and `Q-S5-login-default.png`, crop `crop-s5-login.png`. Left panel, Login section:
    <oauth-row>     auth oauth2 · <state>  [Login]
    local           auth none · no credential needed  keyless — noth…
    <api-key-row>   auth api_key · <state>  api-key p…
Centre pane, 'brazen config.toml' section, in the same frame:
    <oauth-row>     auth oauth2 · <state>
    local           auth none · no credential needed  keyless — nothing to log in
    <api-key-row>   auth api_key · <state>  api-key provider — set the key in Config
Ten rows, two renderings each. The duplicate is also the WORSE copy: the left-panel one is width-starved, so its third field is elided on every row while the centre one is complete.

The code already knows there are two lists — the centre pane's own hint reads *'built-in provider rows are compiled into bz and are not shown in this file (**the Login provider list shows them**)'*.

Each list has a job the other does not (Login carries the sign-in verb; Config carries the raw TOML editor), so the fix is unlikely to be 'delete one' — it is that the two should not render the same table.

REPRODUCTION: launch on a scratch world; CLICK '⚙ Config' in the side panel; CLICK 'Login' to expand the fold. Both are view selections, so a coordinate is lawful per the harness STEERING RULE.

TRIAGE ONLY — filed by the first quality audit, not fixed by it.

---

RULING (Marlin): **the roster has one seat — the §8.3 Login tab — and the §9.1 config pane references it.**

The premise had already moved: since bl-1ca2 Login is a §11 center tab, not a left-panel fold, so the two are no longer literally co-visible in one frame. The H1 defect survives the reseat anyway — one derivation (`brazen::row_views`) was painted whole by two surfaces, and this was two renderings of one fact whichever frame you catch them in.

WHY LOGIN OWNS IT. Every column the row carries is a Login fact: the credential words ('signed in' / 'no credential stored') come from the credential store, not from config.toml; the blocked sentence is 'what this row needs to sign in'; and the verb that acts on all of it is bz --login. The config pane's copy was strictly the same sentences minus the verb — the inert copy, and the one whose own text was incoherent in its seat: a row reading 'api-key provider — set the key in Config' was painted *inside* Config. The code had already conceded the seat in prose: BUILT_IN_ROWS_HINT read '... not shown in this file (the Login provider list shows them)'. The reverse seat is impossible — a sign-in verb cannot be referenced, it has to sit on its row — so Config is the surface that can be a reference and Login is the surface that must be the seat.

WHAT CONFIG PAINTS INSTEAD. The two facts the *file* owns and Login does not state: a count of the rows it ends up routing ('N provider rows are effective in this workspace — the Login tab names them and states each one's credential', counted from brazen's own answer, never pinned) and the standing built-ins hint, whose now-duplicate parenthetical was dropped. Plus one control that focuses the Login tab — the same shape §9.4's credential fault already takes (bl-91f1: name the remedy, carry the thing that goes there), spending the existing tab-focus gesture, hover carrying Ctrl+Shift+3. Subtraction that came with it: the pane no longer reads credential presence at all (its `creds` field and `BrazenEditor::credential_presence` are gone; the free `credential_presence` is the one home).

FOUND WHILE HERE — two QUALITY G1 defects the ten rows had been hiding below the fold of a 420x320 window, both now fixed in this ball: the raw fold's header ('raw config.toml — validated by bz before it lands') was laid 271pt into a 194pt pane and sliced mid-glyph with no ellipsis — §11 rule 1 cannot reach it, egui lays a CollapsingHeader's own text Extend whatever the style says — and, being the widest run on the surface, it set the ScrollArea's content width, so the lernie pane's models.yaml path elided at 287pt and was clipped again at 194. The header now names the file only; the clause it lost is on its hover.

DESIGN amended (the deviation is recorded, not silent): §5.1 row 22, a new §9.1 amendment, the §9.5 enumeration row, §8.3 rule 4, §9.4's remedy bullet, and a §12 module-map row for the new drive file.

DRIVE: src/shell/acceptance/one_rendering.rs — the H1 seat proof in both directions (the non-owner is asserted NOT to paint the row's own words, the owner is asserted to paint them), plus a pointer drive that the reference control focuses the Login tab.
