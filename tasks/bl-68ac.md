+++
title = "every interactive control carries a hover explanation — operator had to ask what Scan means"
created = 1785645595
updated = 1785645603
claimant = "hover-fixer"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator asked 'what does the scan button mean?' (2026-08-02) — the answer (runs lernie scan: deposits died epitaphs for crashed drivers, flushes queued inbox deposits) lives only in DESIGN §8.2/§10. Make it an invariant, not a per-button fix: every interactive control (buttons, fields, checkboxes — Scan, Stop, +children, the dir rung field, ball actions Close/Release/Move, Flush, the composer target line, picker Set…) gets on_hover_text saying what pressing it DOES in operator terms. Sweep the whole shell surface; one test pattern proving each control's hover string is non-empty would hold the invariant (see bl-2d87's paint-layer assertion for the idiom). Wording derives from DESIGN; keep each to a sentence or two. Amend DESIGN §11 with the invariant.