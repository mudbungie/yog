+++
title = "raising a workspace opens an unrequested empty start-goal draft — and Send fires the empty goal onto the wire"
created = 1785646992
updated = 1785647374
claimant = "Whin"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Repro: 'w' → type a valid name → Create workspace. The sphere mints and focuses (correct), but the center-bottom now shows an empty 'Start goal → <new workspace>' draft box with Send (detached prompt)/Cancel that nobody asked for, stacked above the regular composer (bl-6ad8's two-composer problem, here with zero operator input behind it). Clicking Send fires a real detached lernie prompt whose payload is the identity preamble plus nothing ('You are <name>.\n\n' — verified in ops.jsonl) — wire spend on an empty goal. Fix: (1) find why the raise opens a start draft at all — the raise contract (§3.4) is focus + the composer, not a pending draft; (2) an empty goal never sends — disable Send/Enter until the payload is non-blank, everywhere a goal can fire (start draft, composer, ball draft). Acceptance: raise leaves exactly one composer; Send/Enter with blank payload is inert.