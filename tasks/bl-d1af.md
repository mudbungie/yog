+++
title = "logskel names the wrong binary: `command -v yog` resolves the operator's installed yog, not the build under drive"
created = 1786163532
updated = 1786163532
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
scripts/drive/logskel.sh:43 is `bin=$(command -v yog 2>/dev/null || echo "not on PATH")`. Inside `drive.sh` the PATH is arranged so that resolves the driven build; run standalone via `make drive-log` — which docs/QUALITY.md §3 step 6 now REQUIRES an audit to start from ("Start it generated, not blank") — it resolves `~/.local/bin/yog`, the operator's live installed binary, which is a different sha.

The emitted skeleton then names a binary that was never driven, in the one field a drive log exists to pin (QUALITY.md §4: "a verdict is a claim about the sha it names, and about nothing newer"). It fails silently and plausibly: the path looks right.

Found by Shotwright during the first quality audit (bl-65f7, log docs/drive-logs/2026-08-07-quality-audit.md), who caught it and corrected the binary by hand rather than shipping the wrong one. Landed in bl-56d5 (4b0e75c).

Fix: resolve the binary the same way the drive did rather than re-asking PATH — take it from the run (the verdict rows / the drive's own resolved binary) or require it as an argument, and make an unresolvable binary loud instead of a plausible default. A `not on PATH` fallback string in a field that must be exact is the same silent-default smell yogdrive.sh:58 already refuses for the seat.