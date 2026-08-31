# yog — drive quality

**Scope.** `STORIES.md` owns *does it work*: the acceptance ladder and its
two-half done-bar. This document owned *does it feel good* — the rubric every
rendered surface was judged against, and the shot-sheet that sampled those
surfaces. **yog renders nothing** (bl-7942): the window is the `lernie` seat
crate's, and so is the rubric. What is left here is the half that was never
about pixels — the protocol a real-substrate drive follows, and the rule that
says when any verdict has expired. When this document and the harness disagree,
one of them is amended deliberately, never coded around.

---

## §1 The rubric (retired — the seat's)

Geometry (G1–G4), Honesty (H1–H4), Hand-feel (F1–F6) and Language (L1–L4) are
**retired to git history** (bl-7942). Every one of them was a claim about a
rendered surface, judged from two witnesses this crate no longer has: the
captured pixels and the in-crate paint layer. A rubric enforced by nobody is
worse than none, and the seat crate is where the surfaces are.

Two of the rubric's claims were never really about pixels and are **kept as
invariants elsewhere**, which is why they are named here rather than simply
dropped:

- **H2, "absence is named"** — a fact yog cannot derive is answered as absent
  and never as a zero. That is a property of the §8.5 replies and is tested at
  the boundary (DESIGN §8.5; every `Option` field on a reply row).
- **F4, "the frame never blocks"** — no substrate read, spawn or wait on the
  paint path. Its functional twin is STORIES INV-1 (idle is pure), and the
  structural half is now unconditional: derivation is the worker's, every seat
  is a separate process, and nothing in this one has a frame at all.

The heading stays so citations (`QUALITY §1`, `G1`, `F1`, …) keep resolving.

---

## §2 The shot-sheet (retired — the seat's)

Fourteen sampled surfaces at three window sizes, captured per run. Retired with
§1 and for its reason: there is no surface in this repository to sample. The
heading stays so citations keep resolving.

---

## §3 The drive protocol

What a dispatched model does, start to finish:

0. **Preflight.** `make drive-preflight` — it names *every* missing host
   requirement at once instead of dying ten seconds in: the tools (python3,
   git, the `yog` under drive), the two world-seed files, and the **wall** — whether a workspace born in a scratch
   world will carry the provider rows its birth template names, asked of the
   binary under drive through an empty wall, plus a host credential to seed
   into it (DESIGN §16.2; bl-49c6). Both are **advisory** since bl-00ee retired
   the §9.2 birth gate: a workspace is born whatever its template names, so a
   missing row or sign-in costs the wire beats and nothing else. An audit that
   starts here does not discover its host one binary — or one fixture — per
   attempt.
1. **Build and isolate.** `make release`; drive the release binary in a
   scratch world (`XDG_DATA_HOME=<scratch>`). Never the live world — the
   `make drive` family refuses a scratch root that overlaps `$XDG_DATA_HOME`
   in either direction.
2. **Populate.** Seed with the drive harness's own verbs — `make drive-seed`
   prints a laid scratch world's path (`stories.sh seed` underneath). A
   wire-dead world still answers most of the boundary; name any beat skipped
   and why.
3. **Drive.** `make drive`. Every beat is a §8.5 gesture, and each leaves a
   `verdicts.jsonl` row beside `gestures.jsonl` (DESIGN §12.2): one row per
   beat, so what the run *established* is machine-readable rather than re-read
   out of the scroll.
4. **Score.** Every beat gets pass / fail with its own reply and the world it
   left behind as witness. Judge what the engine answered, not the intent.
5. **File.** One ball per failure: title states the symptom, body cites the
   beat, the reply and the reproduction gesture. Search
   `bl list <needle> --all` first — a recurrence of a closed ball is filed as
   a regression naming it.
6. **Record.** One log per run, **beside the run it is about**: `drive.sh`
   writes `<run>/drive-log.md` under the evidence root `$DRIVE_ROOT` (default
   `$XDG_CACHE_HOME/yog-drive`), with the `gestures.jsonl` and
   `verdicts.jsonl` it quotes. It carries the build sha, host tuple, sheet
   coverage (including named skips), the beat table and the filed balls.
   **Start it generated, not blank** — `make drive-log` emits the sha, the host
   tuple, the load, the binary the run actually drove (read off its own verdict
   rows, not re-resolved from the PATH of whoever is generating the log — that
   PATH answers with the *installed* yog, bl-d1af) and any beat table the run
   produced; the scorecard and every judgement are written over that skeleton by
   hand. The house style is evidence quoted, not summarized — exactly the half
   no generator can supply.

   **A run that produced no beats still gets a log** (bl-d0a0). A red run is
   exactly when a report is wanted, and the reddest run of all is one that died
   before its first assertion — an engine that never came up, a harness refused
   at source time. The skeleton then carries the stage table (`drive.sh` writes one
   `stages.tsv` row per verb it drives, verb and exit code) and the sentence
   **NO VERDICTS PRODUCED** in place of a beat table, so the document says what
   happened rather than nothing. The generator never answers with an exit code:
   a report's own failure may not replace the run's.

   **The log does not come back into the checkout.** It used to: one file per
   run under `docs/drive-logs/`, exempt from the home-path rule because a log's
   paths were its evidence. bl-244f burned all eleven and the exemption with
   them — they were the single largest carrier of operator-home paths in this
   repository's history, and they shipped inside the 0.0.1 crate. So the log
   stays where it is written, outside the tree, and **what comes back is step
   5's output: the balls filed.** A verdict worth keeping is a claim in
   `docs/STORIES.md` citing the run by date and verb, not a file.

   **Quote the evidence, not the operator**, even so — a log gets read aloud,
   pasted into a ball, and quoted in a PR. `logskel.sh` folds `$HOME` to `~` in
   every path it emits; fold the same way in every line you add by hand, and
   mind the two other rules a quoted log trips if any of it reaches a commit or
   a ball body: `personal-email` and `quoted-dialogue` (`scripts/leak-rules.sh`
   is the table; AGENTS.md, "What may never enter a ball body", is the rule).
   An absolute path was never the evidence, the verdict was.

The drive is triage, not repair: it files, it does not fix. Fix balls are
dispatched separately and re-driven by the next run.

---

## §4 The currency rule

A verdict — a drive log's PASS table — is a claim
about the sha it names, and about nothing newer. Main *works* and *looks
right* exactly as far as the newest log whose sha is an ancestor of main; a
merge that touches a judged surface voids that surface's rows, functional and
quality alike. A verdict is never "the state of yog"; it is the state of
one build, and re-establishing it after the surface moves is ordinary work,
filed and dispatched like any other. Cadence is the operator's policy;
expiry is this rule.
