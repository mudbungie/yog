+++
title = "DECISION: docs/CLAUDE_CODE_GAP_ANALYSIS.md is now published — keep it, trim it, or unship it"
created = 1786683106
updated = 1786683106
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["publication"]
+++
Surfaced by the post-publication sweep of bl-4f96. Not a leak — the leak scan
passes on it, and it discloses nothing about the operator or the machine. It is
a judgement call about what yog's public repository says, and it needs an
operator answer rather than an implementer's guess.

`docs/CLAUDE_CODE_GAP_ANALYSIS.md` (280 lines) ships in the repository AND in
the crate: it appears in `cargo package --list`, so it is inside yog 0.0.2 on
crates.io as well. Its own header:

    **Status:** evidence review reconciled through yog `9b0b2f4`, balls `0.5.9`,
    brazen `0.0.5`, and lernie `0.0.3`; Claude Code source snapshot
    `06d29efd02547a586a33cab60e8acf3dba2997e8` (dated 2026-03-31). The local
    installed CLI was `claude 2.1.220` when its public `--help` surface was checked.
    The GitHub repository is an unofficial third-party source snapshot, not an
    Anthropic release; provenance and completeness are unverified.

and its verdict opens:

    Claude Code is presently the stronger **single-repository coding cockpit**.

What the sweep checked, so the decision is made on facts:

- it reproduces NO source — zero code fences, zero function/const/import lines;
- it names no third-party repository URL, only the snapshot's commit sha;
- it is prose comparison throughout.

So the exposure is reputational and editorial, not legal or disclosive. Three
things it does that a public repository may not want to do: it publishes a
competitive assessment of another vendor's product under the maintainer's name;
it cites an admittedly unverified unofficial snapshot of that product as
evidence; and it is stale on its face — reconciled against lernie `0.0.3` and
brazen `0.0.5`, where the tree now pins lernie `=0.0.8`.

Three ways:

1. **Keep it.** It is honest analysis, it reproduces nothing, and the status
   header already states its provenance and its limits.
2. **Keep it, refresh it, and exclude it from the crate.** Re-reconcile against
   the current pins, and add an `exclude` to Cargo.toml so design commentary
   stops shipping to crates.io with the binary — which is a good idea for
   `docs/` generally, and is the same lever that would have kept the drive logs
   out of 0.0.1.
3. **Unship it.** Move it out of the published tree; it was internal reasoning
   and does not need to be a public artifact.

Recommended: 2. The `exclude` half is worth doing on its own merits whichever
way the doc goes.