+++
title = "S5 brazen Apply beats are stale AGAIN: bl-5410 gave every provider row a second wrapped line, so the raw-config fold moved ~119px below the click bl-f8dc measured"
created = 1786600609
updated = 1786600715
claimant = "Rowel"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["drive"]
+++
Found while driving the full ladder for bl-1851 (a different subject — the
wall's ordering). **Not caused by bl-1851**, and proved so: the identical two
reds were reproduced on the SAME binary with the pre-bl-1851 scripts restored
(`git checkout HEAD~1 -- scripts/drive/`), minutes apart, on the same box.

## The two reds

    S5 brazen: Apply lands (the bracket for the negatives)  FAIL — marker not on disk
    S5-T4 hash-guard: reload then the same Apply lands      FAIL — marker not on disk

Same pair, same detail string, as bl-f8dc. That ball is closed and its repair is
correct in kind — it moved the beats inside the `raw config.toml` fold — but the
coordinate it landed has since been invalidated by a yog surface change.

## The cause, with both dates

`scripts/drive/beats_s5.sh` opens the fold with a click at **(330, 273)**,
measured against `s5-03b-raw-fold.png` by bl-f8dc, commit `2b2722e`,
**2026-08-11T22:28:18-07:00**.

`2229182` (bl-5410, **2026-08-12T18:02:24-07:00**) then changed
`src/shell/config_edit/brazen_pane.rs::provider_table` so each provider row's
`blocked` hint is *its own wrapped line* instead of a third element on the row:

    -            if let Some(why) = &row.blocked {
    -                ui.colored_label(theme::ASH, why);
    -            }
             });
    +        if let Some(why) = &row.blocked {
    +            // Its own **wrapped** line, not a third element on the row above
    +            // (bl-5410). ...
    +            ui.scope(|ui| {
    +                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    +                ui.colored_label(theme::ASH, why);
    +            });
    +        }

All seven shipped rows carry a `blocked` hint (`keyless — nothing to log in` for
the two `auth = none` rows, `api-key provider — set the key in Config` /
`bearer-token provider — set the token in Config` for the rest), so the table is
**14 lines where it was 7**. The evidence is the beat's own screenshot,
`/home/u/.cache/yog-drive/20260813T042347Z/run-s5s8/out/s5-03b-raw-fold.png`:
the `▶ raw config.toml — validated by bz before it lands` header sits at
**y≈392** and the mouse X is painted at (330, 273), on the `google` row. The
fold never opens, so there is no Apply to press, so the marker never lands —
which is exactly the failure mode the file's own comment predicted:

> A miss is loud, not silent — with the fold shut there is no Apply to press and
> the bracket beat below goes red, which is the beat that exists to say so.

The beat said so. Nobody had driven it since.

## The fix

Re-measure inside the opened fold, against a fresh `s5-03b-raw-fold.png`, at
yog's default 1150x760 window with the side panel at 260. Two coordinates in
`scripts/drive/beats_s5.sh`: the fold header (currently `330 273`) and the Apply
button inside it (currently `305 386`); `brazen_draft`'s text-box click at
`400 330` is in the same column and is owed the same re-measure. **Drive it
green before closing** — bl-f8dc was claimed and closed inside six minutes,
which is less than one `run-s5s8` takes, so its coordinate was never driven.

## The standing hazard, worth a sentence in the file

This is the second re-baseline of the same three coordinates in two days
(bl-2622 → bl-f8dc → here), and both invalidations came from *deliberate*
surface work with DESIGN amended to match. The beats are the last consumer of a
pixel column that four balls have now moved. Whether §9.1's Apply is really
unreachable by any named spelling is worth re-asking: `beats_s5.sh` argues at
length that it is (the §8.5 boundary keeps the draft's Apply on the pane, and
§11's focus floor cannot Tab off a code editor), and if that stands, this file
will keep paying this bill.