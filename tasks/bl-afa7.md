+++
title = "S4-T1 drive beat is stale: `w` opens the name-the-sphere modal, so `await verb_ge new 2` can never land"
created = 1786162675
updated = 1786162675
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["drive"]
+++
Found by the 2026-08-07 re-baseline drive (bl-c63e), docs/drive-logs/2026-08-07-ladder-rebaseline.md. Reproduced identically on two runs of `scripts/drive/stories.sh run-s3s4s6` against release `aafa438`.

Beat: `S4 new workspace: second lernie new` (STORIES S4-T1), `scripts/drive/beats_s3s4s6.sh:178`.

    "$drive" bare "$wid" w
    await verb_ge new 2 \
      && pass "S4 new workspace: second lernie new" \

Verdict lines (both runs):

    S4 new workspace: second lernie new            FAIL — no mint
    S4 new workspace: two named spheres            FAIL — not two
    S4 mint focus: the bottom composer fired       FAIL — no prompt row

## The beat is stale; yog is right

`w` fires correctly — it opens a modal. Screenshot `s4-11-second-workspace.png`:
title `new workspace`, prompt `name this sphere — a client, an employer, personal vs. work:`, an empty name box, and a `Create workspace` button.

- `src/shell/new_ws.rs:56` `egui::Window::new("new workspace")`
- `src/shell/new_ws.rs:74` `ui.label("name this sphere — a client, an employer, personal vs. work:");`
- `src/shell/new_ws.rs:78` `.hint_text("ops")` — the `ops` seen on screen is HINT text; the field opens empty
- `src/shell/new_ws.rs:116` `.add_enabled(verdict.is_ok(), egui::Button::new("Create workspace"))` — empty fails validation, so the button is disabled

So `bare w` alone can NEVER fire `lernie new`, and no `await` window changes that. The beat neither types a name nor confirms.

This is deliberate design and both authorities already say so:

- docs/DESIGN.md §11 binding table: "| `w` | Ctrl+Shift+N | new workspace: the deliberate sphere-wall raise, **name typed by the operator** (§3.1, §3.4) |"
- docs/STORIES.md S4-T1: "the deliberate raise fires a second `lernie new` with the operator's **typed, validated** name (DESIGN §3.1 as amended at bl-df65)"

Introduced by `ae80e99` (2026-07-29) "workspace names are operator-chosen: typed+validated at New workspace, fixed `home` at bootstrap; the workspace mint dies [bl-d942]". The beat comment ("`w` is §11 s deliberate sphere-wall mint") was authored 2026-07-26 in `ce5fc5c` (bl-30e3), three days earlier.

## Repair (no new machinery)

Return already confirms the form — `src/shell/new_ws.rs:88`:

    let entered = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

So the beat becomes: `bare w` -> type a valid name -> `Return`, then `await verb_ge new 2`. Covered in-crate by `src/shell/acceptance/modal.rs:91` `return_submits_a_valid_name_and_a_refused_one_keeps_the_form`.

## Also fix: two beats that PASS vacuously

`beats_s3s4s6.sh:203` sets `minted=$(find … ! -path "$first_ws")`, which is the EMPTY STRING when the mint never happened. The next assertion is then

    grep '"lernie","prompt"' "$ops" | tail -1 | grep -q "\"$minted\""

i.e. `grep -q '""'`, which matches any row. `S4 mint focus: Enter prompts the MINTED sphere` and `S4 mint focus: focused => no re-mint` both reported PASS while asserting nothing. Guard on a non-empty `$minted` — a beat that cannot fail is worse than a red.