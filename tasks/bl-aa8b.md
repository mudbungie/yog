+++
title = "freeze project instructions into agent context with visible provenance"
created = 1785649815
updated = 1786685790
claimant = "Rabbet"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design"]

[[blockers]]
id = "bl-2b8c"
on = "claim"

[[blockers]]
id = "bl-6654"
on = "claim"
+++
Source: bl-e249's Claude Code comparison at snapshot 06d29efd02547a586a33cab60e8acf3dba2997e8.

## Verified gap

Claude Code deterministically discovers project instruction files and rules from the project hierarchy before work. Yog and pinned lernie discover neither this suite's `AGENTS.md` hierarchy nor a generic configured instruction hierarchy. Lernie's shipped worker soul is only harness mechanics. Yog currently sends target location and task content in the editable goal, so repository standards depend on the model finding them itself.

Claude Code does NOT normally auto-load `AGENTS.md`; this is evidence for a project-context mechanism, not a request to hardcode Claude's filenames or copy its memory system.

## Deliverable

bl-2b8c ruled (VISION §4.10): the authoritative target is the bound attempt worktree, passed typed at fire by bl-6654 (this task's claim blocker). Make project instructions a frozen, inspectable spawn input:

- filename set and precedence are yog policy/config, with a documented default that covers this repository's `AGENTS.md`;
- discovery is deterministic from authority root to the typed target;
- exact bytes and source provenance are snapshotted before the first inference through lernie's generic pinned-document mechanism — **released and pinned** (bl-fb5c, lernie 0.0.4; yog pins 0.0.6), so no upstream wait remains beyond bl-6654;
- the operator sees what was included; the user's goal remains their payload, not a hidden concatenation;
- child inheritance follows VISION §4.10 (a write-capable child's instructions re-freeze against its own attempt, never inherit stale bytes);
- size is bounded and unreadable files, traversal, external includes, and untrusted parent instructions have explicit fail/skip policy;
- no opaque automatic memory or yog-side duplicate store.

---


## Delivered (Rabbet) — DESIGN §3.7, and where the tree corrected this body

The body predates lernie `=0.0.8`, bl-6654 and bl-0cea. Verified against the
tree; four corrections, recorded here as the ruling:

1. **The pin freezes but does not compose — the body's central premise was
   incomplete.** The body says the bytes are "snapshotted before the first
   inference through lernie's generic pinned-document mechanism", implying that
   is the whole job. It is not: whether a pinned blob reaches assembled context
   is the governing `manifest.yaml`'s question (lernie ARCH §5.2), and the
   shipped worker role pins only `goal.md`, `soul.md`, `descriptions/**` and
   orders `summary/**`, `skills/**`. A pin at `instructions/…` under the stock
   manifest is a committed file no model ever sees. So the deliverable grew a
   second half: yog authors `roles.worker.pinned: - instructions/**` onto
   `config/default` at every start, by the same fixed-point convergence bl-0cea
   landed for `tool_control:` — and in the *same* `lernie config` drive, since
   the two files are one policy (one checkout, one commit, one ops row).

2. **Discovery anchors on the typed `--cwd` binding, not on goal prose.**
   bl-6654 retired the prose channel, so `Prepared::binding` is the single
   input: ball rung → claim-derived `work/<id>`, path rung → the directory,
   bare rung → nothing, which discovers nothing. No rung table.

3. **"Untrusted parent instructions" needed no policy, only a rule.** The
   authority root is the nearest ancestor of the binding (inclusive) holding a
   `.git` entry — a *file* for a worktree — and the walk never ascends above
   it. Ambient/home/parent-of-repo instruction files are unreachable by
   construction rather than skipped by a check. Over-size files are skipped
   whole, never truncated; symlinks, non-UTF-8 paths and paths carrying `=`
   (`--pin`'s own separator) are skipped; includes are never followed.

4. **Provenance stores nothing, because three existing surfaces already carry
   it.** The fire's `ops.jsonl` argv (every `--pin dest=src`, and `clip_goal`
   trims only the goal), the frozen blobs on the dispatch commit (the §11 Files
   tab lists `instructions/**` already), and the destination itself —
   `instructions/<NN>/<rel>`, where the rank prefix is what makes precedence
   survive lernie's lexical sort (its own `summary/NNN.md` device) and the rel
   path is the source. No yog registry, no second copy; yog reads no
   instruction bytes at all.

Also: the body's "lernie 0.0.4; yog pins 0.0.6" is stale — the pin is `=0.0.8`.
Child inheritance needed no mechanism: a child forks its parent's tree and
binds no directory of its own (VISION §4.10 items 2–3), so the bytes it
inherits are the same bytes for the same target; the re-freeze rule is stated
for when bl-8746 gives a child its own attempt.

Filename policy: default `AGENTS.md` in code, override at
`config/default:instructions.yaml` read at the live tip (the `capability.yaml`
pattern) — an existing file is authoritative even when it names nothing, which
is the explicit opt-out.

Found, not fixed: this machine has a stray `/tmp/.git`, so any binding under
`/tmp` takes `/tmp` as its authority root. Debris, not a yog defect — but it is
why the walk's "no repository above" case is asserted at `/` rather than in a
tempdir.
