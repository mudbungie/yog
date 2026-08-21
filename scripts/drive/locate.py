"""locate.py — the pixel half of `locate.sh`, which is the entry point and the
whole argument: read it first. This file is handed a verb, the shot's own size
and one grey plane of it, and answers with the points that surface's beat is
about to drive, or refuses.

Not an entry point of its own and never imported: `locate.sh` calls it with
`<verb> <w> <h> <plane>` and owns the temp file it names. Split out at the
300-line cap (bl-fc3f) on the seam the two halves already had — the shell
claims the plane, the reader reads it.
"""

import collections
import sys

verb, w, h, path = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
px = open(path, "rb").read()
# The window's own background is whatever colour most of it is — read, not
# assumed, so a theme change moves the threshold with it rather than past it.
bg = collections.Counter(px).most_common(1)[0][0]


def longest_run(y):
    """The longest flat horizontal run in row `y`, as (length, x, value)."""
    row = px[y * w : (y + 1) * w]
    best, x = (0, 0, 0), 0
    while x < w:
        v, x2 = row[x], x
        while x2 < w and abs(row[x2] - v) <= 2:
            x2 += 1
        if x2 - x > best[0]:
            best = (x2 - x, x, v)
        x = x2
    return best


def rules(span, inset):
    """The horizontal rules in the frame, top down, as (y, x0).

    A rule is a long flat run brighter than the background. `inset` is the whole
    of what tells the two FAMILIES apart: a section separator inside a column
    starts at that column's own left edge, while a window-wide panel's rule
    starts at x=0 and crosses the whole frame. Asking for one never returns the
    other, so neither family's count shifts when the other gains a member.
    """
    hits = []
    for y in range(h):
        n, x0, v = longest_run(y)
        if n >= span * w and v >= bg + 15 and (x0 > 0) == inset:
            hits.append((y, x0))
    # Coalesce, and keep only the THIN ones. A run of three or more adjacent hit
    # rows is a filled block (a text box, a selected row), not a rule.
    found, group = [], []
    for hit in hits:
        if group and hit[0] - group[-1][0] > 1:
            if len(group) <= 2:
                found.append(group[0])
            group = []
        group.append(hit)
    if group and len(group) <= 2:
        found.append(group[0])
    return found


def want(found, least, what):
    if len(found) < least:
        sys.exit(
            f"locate.sh {verb}: found {len(found)} {what} in the frame, want at "
            f"least {least} — is this the frame the surface describes?"
        )
    return found


# A column's own separators: a run over most of the window's width but starting
# inside it. The width test is also what keeps the roster column out — §11 caps
# it at half the window, so only the centre's rules can clear four tenths of one.
def column():
    return want(rules(0.4, True), 2, "centre rules")


# The window-wide panel rules: the top bar's under the frame's first row, the
# activity accessory's above its last.
def panels():
    return want(rules(0.9, False), 2, "panel edges")


def clusters(band, x0, gap):
    """Runs of ink in `band`, coalesced across gaps of `gap` px or less."""
    found, cur = [], None
    for x in range(x0, w):
        if max(abs(px[y * w + x] - bg) for y in band) > 4:
            if cur and x - cur[1] <= gap:
                cur[1] = x
            else:
                found.append(cur := [x, x])
    return found


# WHICH CENTRE TAB IS SELECTED, as the digit that focuses it — the fact every
# centre locator needs and no file holds. The strip is the band between the top
# bar's rule and the centre's first one; it is laid left to right in
# `CenterTab::all()` order and the one conditional tab (Search) is LAST, so a
# hidden tab never shifts the ones above it and the painted ordinal *is*
# `CenterTab::digit()` — the same digit `Ctrl+Shift+<n>` spells. The selected
# tab is a `selectable_label(true, …)`: a FILLED rect, so its band is mostly
# `selection.bg_fill` where an unselected label's is mostly background — read,
# never assumed, so a theme moves both together. GLYPH is the widest gap inside
# a label and TAB the strip's own item spacing (measured 13–14 px at every
# panel width in the evidence): clustering at the first coalesces a label,
# stopping at the second keeps the ⋯ overflow MENU out, which opens over this
# same band hundreds of pixels to the right.
def center_tab():
    band = range(panels()[0][0] + 3, column()[0][0] - 2)
    GLYPH, TAB = 6, 24
    tabs = []
    for run in clusters(band, column()[0][1], GLYPH):
        if tabs and run[0] - tabs[-1][1] > TAB:
            break
        tabs.append(run)
    lit = [
        i
        for i, run in enumerate(tabs)
        if collections.Counter(
            px[y * w + x] for y in band for x in range(run[0], run[1] + 1)
        ).most_common(1)[0][0]
        != bg
    ]
    if len(lit) != 1:
        sys.exit(
            f"locate.sh {verb}: the centre tab strip paints {len(lit)} selected "
            f"tabs of {len(tabs)} — nothing is standing where this reads."
        )
    return lit[0] + 1


# The guard, spent by both centre locators: this frame's rules belong to the
# surface the beat thinks it drove, or nothing is aimed at at all (bl-fc3f).
def on_tab(digit, name):
    got = center_tab()
    if got != digit:
        sys.exit(
            f"locate.sh {verb}: the centre is on tab {got}, not {name} "
            f"(Ctrl+Shift+{digit}) — the navigation that opens it never landed, "
            "so every point below would aim into another surface."
        )


if verb == "centertab":
    print(center_tab())
elif verb == "brazen":
    on_tab(2, "Config")
    # The Config column's rules, top down: the one under the §11 centre tab
    # strip, then one per pane. brazen is the column's FIRST pane
    # (config_edit/mod.rs renders it before lernie, yog, marks and the branch
    # form), so the rule that ends it is the second — and the fold is the row
    # above that rule.
    #
    # The offsets, measured once against an OPENED fold (2026-08-13, and the
    # shot that carries them is any run's `s5-03b-raw-fold.png`). `FOLD_UP` is
    # the item spacing above a separator plus half a header row; the rest are
    # the fold's own body — egui's indent, a `desired_rows(6)` editor spanning
    # y+13..y+100, and the button row at y+105..y+121 whose three seats start at
    # the pane's x+18.
    sep_y, pane_x = column()[1]
    FOLD_UP, FOLD_IN = 16, 40
    BOX_DOWN, BOX_IN = 56, 60
    BTN_DOWN, APPLY_IN, RELOAD_IN = 113, 37, 87
    fold_y = sep_y - FOLD_UP
    print(
        pane_x + FOLD_IN,
        fold_y,
        pane_x + BOX_IN,
        fold_y + BOX_DOWN,
        pane_x + APPLY_IN,
        fold_y + BTN_DOWN,
        pane_x + RELOAD_IN,
        fold_y + BTN_DOWN,
    )
elif verb == "inspector":
    on_tab(1, "Conversation")
    # The centre's rules on the Conversation tab, top down: the one `center.rs`
    # ends its tab strip with, then the one `workspace.rs` paints between the
    # conversation's identity header and the altitude-2 strip. Everything the
    # header can grow — a replay line, one §6 mark per kind, the auth banner,
    # the wound banner, the ball rows — sits BETWEEN those two rules, so the
    # second one carries the whole inspector down with it and none of the
    # offsets below ever learn about it. That is the drift bl-1ca2 caused and
    # nothing noticed: a strip and a rule appeared above, and three clicks
    # measured before it went on aiming twenty-eight pixels high.
    #
    # Below the rule the inspector's own rows are fixed by `inspector/mod.rs`
    # and `inspector/controls.rs`, in this order and no other: the tab strip,
    # the Raw checkbox (bl-1ff1 put it on Steps too), then — Steps only — the
    # step selector and, once a step is picked, the record picker. `RECORD_IN`
    # clears `meta` to land on `request`, the record holding the malformed file.
    sep_y, pane_x = column()[1]
    RAW_DOWN, RAW_IN = 36, 40
    STEP_DOWN, STEP_IN = 56, 40
    RECORD_DOWN, RECORD_IN = 77, 60
    print(
        pane_x + RAW_IN,
        sep_y + RAW_DOWN,
        pane_x + STEP_IN,
        sep_y + STEP_DOWN,
        pane_x + RECORD_IN,
        sep_y + RECORD_DOWN,
    )
elif verb == "activity":
    # The trail is docked to the window's BOTTOM edge and the §11 tail idiom
    # seats its newest row on that edge (`crate::tail::scroll` sticks an
    # overfull body to the bottom and pads an underfull one down to it), so the
    # newest row is one intra-widget distance up from the frame's last pixel —
    # immune to the chip's own heading, to the Dismiss / Clear trail controls
    # and to the two §7.2 derivation notes that appear above them only when the
    # snapshot is stale. Nothing above the row can move it, which is the whole
    # test. `ROW_IN` is the panel's left margin plus the collapsing triangle.
    #
    # The panel rule is still read, as the GUARD the beat otherwise lacks: a
    # collapsed accessory is one row of chrome, so its edge sits within a row
    # height of the bottom, and a click into it would silently *toggle the very
    # thing the beat needs open* instead of opening a row. `a` failing to
    # expand the trail now stops the beat here rather than passing it vacuously.
    ROW_IN, ROW_UP, TRAIL_LEAST = 37, 12, 60
    edge_y, _ = panels()[-1]
    if h - edge_y < TRAIL_LEAST:
        sys.exit(
            f"locate.sh activity: the bottom panel is {h - edge_y} px tall — the "
            "activity trail is still collapsed, so there is no ops row to open."
        )
    print(ROW_IN, h - ROW_UP)
elif verb == "tabbar":
    # The §11 tab bar is laid right-to-left with the ⋯ overflow painted FIRST,
    # so it takes the window's right edge as soon as it is non-empty — and the
    # top bar is the frame's first panel, so nothing can be inserted above it.
    # Both coordinates therefore come off edges: the bar's own rule for y, the
    # window's right for x. The menu it opens is wider than the gap between the
    # button and that edge, so egui always clamps the popup INTO the frame and
    # the ★ — the last widget of the entry's row — lands one inset from the same
    # right edge. Which foreign workspace to pin is the pick; the ★ is the only
    # safe seat in the row, since the entry's own label focuses it instead.
    MORE_UP, MORE_RIGHT = 9, 19
    PIN_DOWN, PIN_RIGHT = 16, 14
    bar_y, _ = panels()[0]
    print(w - MORE_RIGHT, bar_y - MORE_UP, w - PIN_RIGHT, bar_y + PIN_DOWN)
