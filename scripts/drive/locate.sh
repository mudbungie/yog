#!/bin/bash
# locate.sh — find a driven control in the window the run is ALREADY
# screenshotting, so a beat aims at a structure instead of a pinned number.
#
# WHY THIS EXISTS. stories.sh's STEERING RULE allows a coordinate for a VIEW —
# a fold, a focus, a pick with no address — because §8.5 gives views no boundary
# representation to prefer. What it does not say, and what four balls have now
# paid for (bl-2622 → bl-f8dc → bl-b9f2 → bl-5cce), is that a *measured*
# coordinate is a second representation of the layout, and two representations
# of one fact drift. Every one of those balls was the same failure: deliberate
# surface work moved a row, and a number measured against last week's screenshot
# went on pointing at where the row used to be. bl-5410 gave each of brazen's
# seven provider rows a second wrapped line and moved the §9.1 raw fold 119 px
# down; the beat below it kept clicking the old y and reported "marker not on
# disk" for a fold that simply never opened. bl-1ff1 put a Raw checkbox above
# the Steps tab's step selector and bl-1ca2 put a whole centre tab strip above
# the inspector, and S7's three clicks spent twelve days landing on blank panel
# while their beat went on printing PASS — a negative assertion ("the ops trail
# did not grow") is satisfied by a click that hits nothing at all.
#
# So the number is not measured here, it is DERIVED, once per run, from the
# frame the beat is about to drive. None of these controls has a name at the
# boundary or a key of its own — but each has a POSITION RELATIVE TO A STRUCTURE
# that moves with the layout, and that is what is written down:
#
#   * the §12 Config column and the centre both paint `ui.separator()` between
#     their sections, so a pane is addressable as "the Nth rule down";
#   * the top bar and the activity accessory are window-wide panels, so each
#     draws its own rule from edge to edge — a different family from the ones
#     above, told apart by exactly that;
#   * a panel docked to an edge is addressed FROM that edge, and the §11 tail
#     idiom (`crate::tail::scroll`) seats the newest row on the bottom one
#     whether the trail holds two ops or two hundred.
#
# Usage: locate.sh <surface> <shot.png>   (any window size, any panel width)
#
#   brazen    §9.1's four points, off the rule that ends brazen's Config pane —
#             the shot must have the raw fold still SHUT (opening it is what the
#             first point is for).
#             → fold_x fold_y box_x box_y apply_x apply_y reload_x reload_y
#   inspector §11 altitude-2's three controls, off the rule the centre paints
#             between a conversation's header and its tab strip. The shot must
#             be of the Conversation tab with the Steps tab open on an agent
#             that HAS steps (the step row is the third point's whole subject).
#             → raw_x raw_y step_x step_y record_x record_y
#   activity  the newest row of the EXPANDED activity trail, off the window's
#             own bottom edge. Refuses on a shot whose trail is still collapsed.
#             → row_x row_y
#   tabbar    the §11 top bar's ⋯ overflow button and the ★ in the menu it
#             opens, off the bar's own rule and the window's right edge.
#             → more_x more_y pin_x pin_y
#
# The measured leftovers are the OFFSETS in the table below, and they are a
# different kind of number: each is a distance INSIDE one widget — the gap from
# a fold's header to the editor egui indents under it, the height of a
# `desired_rows(6)` box, the checkbox above a step selector, a menu row's own
# inset — so only that widget's own contents can move them, never anything
# above, below or beside it. That is the whole trade: one run-time anchor per
# surface plus a handful of intra-widget constants, in place of eleven absolute
# pixels.
set -eu
verb=${1:-}
shot=${2:-}
case $verb in
brazen | inspector | activity | tabbar) [ -f "$shot" ] || verb= ;;
*) verb= ;;
esac
[ -n "$verb" ] || {
  echo "usage: locate.sh brazen|inspector|activity|tabbar <shot.png>" >&2
  sed -n 's/^#   \(\S\+\) \+\(§\|the \)/  \1/p' "$0" >&2
  exit 1
}

# ffprobe reports the shot's own size, so nothing here assumes the 1150x760
# launch window — a run at any geometry locates the same way.
size=$(ffprobe -v error -select_streams v -show_entries stream=width,height \
  -of csv=p=0 "$shot")
gray=$(mktemp -t yogdrive-locate.XXXXXX)
trap 'rm -f "$gray"' EXIT
# One plane, one byte per pixel: a separator is a brightness step, and colour
# would only be a third of the data to search for it in.
ffmpeg -v error -y -i "$shot" -f rawvideo -pix_fmt gray "$gray"

python3 - "$verb" "${size%%,*}" "${size##*,}" "$gray" <<'PY'
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


if verb == "brazen":
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
PY
