+++
title = "the ten brazen provider rows paint twice in the same frame: once in the Login pane and once in the Config pane's provider table"
created = 1786163411
updated = 1786678327
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