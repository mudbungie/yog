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
# A COORDINATE IS ONLY EVER RIGHT FOR ONE SURFACE, so every locator that aims
# at the centre column now says WHICH centre tab it is reading, and refuses any
# other (bl-fc3f). A missed navigation used to be converted here into a
# perfectly-formed click somewhere else: `Ctrl+Shift+2` did not land in one
# run of two, the centre stayed on Conversation, and `brazen` read the rule
# above the composer as if it ended the §9.1 pane — so S5's marker was typed
# into the composer and clicked into a `lernie prompt`, spending on the wire in
# the one run whose contract is that it spends nothing. The rules are the same
# family in both frames; only the tab strip says which surface they belong to.
#
# Usage: locate.sh <surface> <shot.png>   (any window size, any panel width)
#
#   centertab the §11 centre tab the strip paints as SELECTED, as the digit
#             that focuses it (`Ctrl+Shift+<n>`) — the guard the two centre
#             locators spend, and the only monotone predicate a beat has for a
#             tab focus, since focus is per-instance RAM (§13.1) and no file
#             can answer it.
#             → n
#   brazen    §9.1's four points, off the rule that ends brazen's Config pane —
#             the shot must be of the CONFIG tab with the raw fold still SHUT
#             (opening it is what the first point is for).
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
centertab | brazen | inspector | activity | tabbar) [ -f "$shot" ] || verb= ;;
*) verb= ;;
esac
[ -n "$verb" ] || {
  echo "usage: locate.sh centertab|brazen|inspector|activity|tabbar <shot.png>" >&2
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


# The reader is its own file (bl-fc3f, at the 300-line cap): this half claims
# the grey plane — one shot, one size, one temp — and `locate.py` reads
# geometry out of it. The seam is where the shell stops and the pixels start,
# and it is the same one the beats files were split on: nothing above knows a
# rule from a cluster, nothing below knows what ffmpeg is. Called, never
# `exec`ed — an exec would drop the trap above and leak the plane.
python3 "$(dirname "$0")/locate.py" "$verb" "${size%%,*}" "${size##*,}" "$gray"
