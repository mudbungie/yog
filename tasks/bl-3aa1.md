+++
title = "activity and inbox rows elide the informative tail of a path or agent id and keep the invariant prefix, so every row scans as the same string"
created = 1786163347
updated = 1786163347
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
QUALITY.md §1 criterion **L4** ('Ids are tamed. A machine id (ancestry chain, sha, absolute path) never dominates a row a human scans: floor to the terminal segment or middle-elide, with the full value one gesture away' — precedent bl-63a1, bl-ac3d). Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out.

NOT A REGRESSION of bl-0bf9. That ball ('a prompt op's full goal text renders inline, breaking the one-row-per-op scan') asked for one line per row with middle/tail elision, and one-line-per-row HAS landed. What did not land is the elision DIRECTION: the rows keep the head and cut the tail, which is the opposite of what L4 asks for.

WITNESS 1 — the activity trail, `Q-S6-activity-default.png` / crop `crop-s6-activity-row.png`:
    · 2026-08-07 21:45:51Z lernie prompt --name growing /home/u/.cache/yog-drive/quality-20260807T214407Z/data/yog/workspac…
    · 2026-08-07 21:47:37Z lernie message /home/u/.cache/yog-drive/quality-20260807T214407Z/data/yog/workspaces/home 202608…
The prefix `/home/u/.cache/yog-drive/quality-20260807T214407Z/data/yog/` is identical on every row and consumes over half of it, while the terminal segment that actually distinguishes the rows — which workspace, which agent id — is exactly what gets cut.

WITNESS 2 — the inbox deposit row, `probe-s6-stirred.png` / crop `crop-s6-overlap.png`:
    ▶ ✉ <agent-id>-<agent-id> · 2026-08-07T22:03:25Z  mail nobody is driving
A full four-token descent chain, 52 characters, rendered unelided at the head of the row. L4 names 'ancestry chain' first among the things that must not dominate.

REPRODUCTION:
  1. scratch world, one conversation on the wire
  2. lay a child branch and an inbox deposit (`agents/<root>-<ts>-<short>` plus `inbox/<root>/<child>-001.md`), as `scripts/drive/beats_s7.sh`'s `lay_forensics` does
  3. bare a  -> activity trail (witness 1); bare 3 -> Inbox, or read the deposit row in the transcript (witness 2)

TRIAGE ONLY — filed by the first quality audit, not fixed by it.