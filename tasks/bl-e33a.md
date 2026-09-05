+++
title = "a `grep -q` fed by a pipe answers FALSE when it matched: the leak self-test's flake, and one site where the gate drops a finding"
created = 1788582608
updated = 1788582608
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Found, measured and fixed downstream in lernie (its bl-7ad6). The scanner, the
rule table and the self-test harness are a deliberate near-byte-identical port,
so yog and brazen carry the same defect at the same sites; brazen took the same
port in its bl-39f1. Filed here rather than patched: this is yog's tree.

## The defect

`printf … | grep -q PATTERN` is a race under `set -o pipefail`, not a style
choice. `grep -q` exits the instant it matches and closes the read end of the
pipe; the writer is then killed by SIGPIPE part-way through its own write;
`pipefail` takes the pipeline's status from that DEAD WRITER rather than from
the reader that answered. **The pipeline reports failure exactly when the
pattern MATCHED**, and the rate rises with load.

`PIPESTATUS` captured at four independent false answers, instrumented in a
copy of the harness: `141 0` — writer killed by SIGPIPE, `grep` exited 0 having
matched. Re-running `scan_rule` inside the same failed iteration reproduced the
finding the check had just called missing, byte for byte.

Measured, one blob of three finding-lines, matching the first, four concurrent
workers on a loaded box:

    printf '%s\n' "$blob" | grep -qE ':2  \['    36 false answers / 16,000
    grep -qE ':2  \[' <<<"$blob"                  0 false answers / 16,000

After the equivalent fix, 1,200 concurrent runs of the whole self-test (twelve
workers) produced zero.

## Where it bites here

**`scripts/leak-selftest.sh`, three sites** — the `-qF` content check, the
`":$ln  ["` anchor check, and the `-qi` FIXTURE_MARKER check. All three are
`… || { report; fails=1; }`, so a false 141 **reports a live rule dead**. That
is the flake: one `make check` fails with `self-test: [<rule>] line N of
<fixture> was NOT flagged` and the next runs pass on the same tree, with
nothing in the fixture or the table touched. A self-test an agent learns to
re-run rather than read is exactly how a genuinely dead rule gets waved
through, and it reddens a close at random besides.

**`scripts/leak-scan.sh`, `scan_paths`** — one site, and this one is not a
flake. The shape is `… | grep -qE "$FORBIDDEN_PATH" && printf '  %s
[forbidden-path]'`: a false 141 means the `&&` never fires, so **a real
credential-shaped path is reported by nobody**. The gate misses a finding, in
silence, at a rate that rises with load. This is the reason the ball is p2 and
not cosmetic.

**`scripts/leak-scan.sh`, `scan_binary`** — the `BINARY_ALLOWED` arm, same
shape. It fails in the safe direction (an extra finding) and is vacuous today
because the allowlist is `^$`, but it is the same line and should move with
the others.

**`scripts/drive/` and the beat harness** — every beat of the form
`<gesture> | grep -q …` carries it in the direction that reddens a beat at
random. `predicates.sh`'s `row_ok`/`reply_exists` and `beats_s18.sh`'s piped
`grep -q` are instances.

## The fix taken downstream

A `grep -q` reads its subject from a **herestring**, never from a pipe:

    grep -qE PATTERN <<<"$subject"

There is no second process to die, under either setting of `pipefail`.
Semantics are byte-identical (a herestring appends the same trailing newline
`printf '%s\n'` did), and bash has had `<<<` since long before 3.2, so the
macOS leg is unaffected.

The ban is on the **shape**, not on the option, because a SOURCED file cannot
see whether its caller set `pipefail` — `leak-selftest.sh` does not set it and
inherits it from the scanner that sources it, which is how the defect reached
the one file whose whole job is to prove the gate is not lying.

Scope is where `pipefail` exists, which is bash. A `#!` naming any other
interpreter is out of it: a POSIX `/bin/sh` script has neither the option that
makes the shape wrong nor the herestring that would fix it, so a `sh` beat or
reconciler is correct as written and must not be flagged. A file with no `#!`
is a sourced bash fragment and is in scope.

## Where the guard belongs in yog

`scripts/beat-audit.sh` already refuses two mechanically checkable shapes of a
drive beat that proves nothing. This is a third, and it is the only one of the
three that can also make the GATE miss a real finding rather than merely
mis-score a beat. lernie had no beat-audit and put the check at the foot of
`self_test` instead, as one more arm of the same two-direction discipline: the
fixtures prove a rule still bites, and this proves that the answer a pipeline
reports is the answer its reader gave.

Whatever scans for it must be self-immune, or it flags its own text forever:
write the pipe as `[|]`, the same idiom `leak-rules.sh` already uses for
`Fil[e]`.

## The second half, and it is the half the ball asked for

`scan_rule` greps with `-I`, which reports NO HITS for a file grep judges
binary and says nothing about why. "This rule matched nothing" and "this file
could not be read as text here" then arrive as the same sentence, and only the
second is a fault of the box rather than of the gate. lernie's `fixture_lines`
now asks the readability question first, with `grep -qI '' "$fixture"`, and
answers it in its own sentence naming `LC_ALL` and `LANG`. It is cheap, it is
one `if`, and without it the infrastructure fault and the dead rule are
indistinguishable in the log an agent reads.