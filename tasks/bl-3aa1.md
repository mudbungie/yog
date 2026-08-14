+++
title = "activity and inbox rows elide the informative tail of a path or agent id and keep the invariant prefix, so every row scans as the same string"
created = 1786163347
updated = 1786678328
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

---

## Unclaimed 2026-08-13 with COMMITTED work surviving — read before re-doing anything (Alkaloid)

Ferrule's session ended before the close landed. **The work is done, committed, and main is already merged in**; only delivery remains.

Worktree was clean (0 dirty files). Two commits sit on the machine-local branch `work/bl-3aa1`, ahead of main:

    2d64a16  Merge branch 'main' into work/bl-3aa1
    4d05c94  elision keeps the end that tells rows apart: argv cuts through the middle, a deposit's sender wears the ladder's floor [bl-3aa1]

Per `bl unclaim --skill`: *"Work you committed on the work/<id> branch survives on this machine … a same-machine claim re-attaches the surviving branch, so committed WIP is already in the new worktree."* **Claim it here and the work is already there.** Do not rewrite it. (From another clone you would get a fresh empty branch and this WIP would stay stranded.)

## The ball as filed was wrong, and the fix reflects that

The filed framing — *keep the distinguishing tail, drop the invariant prefix* — is right for the two witnesses but **wrong as a general rule**, and implementing it generally would have broken working code. A survey found **eight** head-keeping elisions in the tree and most are correct: previews, refusal reasons and ball titles are prose, and prose is written front-first, so its head IS its distinguishing end. Only machine strings — paths, `argv`, ancestry chains — are invariant at the front.

So the rule landed is **"cut where the information is not"**, and `src/elide.rs` deliberately covers machine strings only, with its module doc saying so — a module claiming to be the one home for every cut while eight prose sites kept their own would be a false claim of single-source-of-truth.

The two witnesses needed different mechanisms, which is why one shared function was never going to do it:
- **activity trail** — a character-count cut via `elide::middle`, keeping both ends.
- **inbox deposit** — not a cut at all, but a **floor to a whole terminal segment**, which already had one home in `nav::convs::id_floor` (bl-63a1). Reused rather than respelled.

## The finding worth keeping

An acceptance scan asserts *"no seat formats an agent id as a display name"* — and it was **silent** here, because it enumerates *field names* and knew only `agent_id`/`root_id`, while a deposit carries the same fact under `sender`. bl-63a1 had already recorded that exact lesson verbatim and it recurred anyway. Patched here (with the scan verified to bite by planting `ui.label(deposit.sender)`); the structural fix is **bl-45c7**.

Six assertions, each proved to bite by reverting the fix.

## Remaining

Merge current main, then close. **No separate `make check`** — `bl close` runs the same gate on the tree it delivers. Always `bl -C /home/u/dev/yog close bl-3aa1 --as <you>`. Another fleet is active on this repo, so expect ref-move races: a non-zero close exit after a green gate usually means re-merge and retry; `delivered, not sealed` means a bare re-run, no lock.
