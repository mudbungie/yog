+++
title = "wire evaluation blocked: installed bz 0.0.4 vs lernie linked brazen 0.0.3; the W5 gate is blind to it"
created = 1784955879
updated = 1784959173
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found by the bl-8e07 real-substrate evaluation (docs/drive-logs/2026-07-24-s0-s1-wire-blocked.md).

WHAT IS MISSING
The second half of the STORIES done-bar ("the flow works against the real
one") cannot be evaluated on this machine right now. Every conversation dies
before the model call:

    $ LERNIE_HOME=<scratch>/lernie XDG_STATE_HOME=<scratch>/state \
        lernie prompt <ws> "Respond with exactly this text and nothing else: Manual wire OK."
    lernie prompt: bz version "0.0.4" does not match the linked brazen crate
    "0.0.3" (§4.4 — install the pinned binary: cargo install brazen --version =0.0.3)

Reproduced OUTSIDE yog (no yog in the loop), so it is a host-tuple condition,
not a yog defect. Installed bz mtime 2026-07-24 17:39 (replaced by concurrent
brazen work mid-run); installed lernie mtime 2026-07-22 19:09.

TWO PIECES OF WORK
1. Re-pin the tool tuple (install the bz the installed lernie links, or
   reinstall lernie against 0.0.4) and RE-RUN
   `scripts/drive/stories.sh run` — the payoff beats
   ("S0 payoff: wire reply on disk", "S1 message-to-agent") are unproven until
   then. The three dispatch-only S0 beats and all three S1 structural beats
   passed in this run, so only the model-reply half is outstanding.
2. DESIGN §16.6 W5 note: this skew class is exactly what the capability gate
   exists for ("a stale installed lernie passed the presence gate, then every
   Start died"), and the amended gate still cannot see it — `lernie prompt
   --help` exits 0 while `lernie prompt` refuses. lernie states the mismatch
   itself in one clean line, so a cheap read-only probe may exist (a verb that
   reports its linked-crate versions); if none does, that is an upstream ask,
   and the honest interim is the bl-7f2e rendering (surface the dead step with
   its cause). Decide which, then amend W5 in DESIGN — do not leave the gate
   documented as covering a class it demonstrably does not.

Related: bl-7f2e (no cause rendered for a dead conversation), bl-20f4 (drive
harness coordinate drift).