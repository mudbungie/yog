+++
title = "the path rung's goal preface tells the agent not to trust its cwd — which IS the bound directory — and one run answered by running `find /` for the file and finding three sibling copies"
created = 1788673451
updated = 1788673451
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 swdev lane. yog 0.0.38, litany 0.0.10, worker claude-sonnet-5.

`src/start/goal.rs:59` (`path_preamble`) prefixes every path-rung goal with, verbatim:

    Working directory: <dir>
    Do all work there, by absolute path. Do not rely on the current directory.

The second sentence is false. The binding is also passed typed (bl-6654's channel), and the agent's tools land in that directory: in all three conversations this lane ran, the first `bash` call's `pwd` printed the bound path.

What the sentence costs, measured on three identical-shape starts on three fixture projects:

- The refactor run's FIRST tool call was a `cd` into an invented generic home directory followed by `find / -name "report.ts" -not -path "*/node_modules/*"` — a whole-filesystem search. It returned three copies of the same file (this lane had one project per harness under one root), so the model then spent calls 2-6 on `pwd`, `find / -maxdepth 6 -iname "*.git"`, an environment dump and a second `pwd` disambiguating which of them it was supposed to edit. Call 6 was refused by litany's tool control as `classified secret (dumps the environment)`, which is that gate working, but the model only reached for it because it did not believe `pwd`.
- The bug-fix run's first call was `cd "$(pwd)" && pwd && ls && python3 -m pytest` — the same distrust, one call instead of six.
- The feature run's fifth call was another `cd` into that same invented home directory, then `cd -`, then `pwd`, then `cargo test`.

That invented `/home/<generic-account>` `cd` appears in two of three unrelated conversations: the preamble reads to the model as 'you are somewhere else', and a generic sandbox home is what it guesses at.

That refactor conversation ran 30 steps and 3.49M tokens and triggered a compaction; the bug fix ran 6 steps and 114k. The tasks are comparable in size. The orientation phase is not the whole difference, but it is the whole of the difference in the first six calls, and it is the only part that is yog's own text.

Expected: the headline stays (bl-6654's display-ladder invariant needs line one to be the path), and the second sentence goes, or becomes true — 'Your working directory is already this; relative paths resolve there.' bl-6654 retired the ball rung's location prose on the ground that 'location stops being prose'; the path rung kept a sentence that is not merely redundant with the typed channel but contradicts it.

Severity p2: it costs steps and tokens on every path-rung start, it is one line of text, and the failure is invisible — the conversation still succeeds, just longer.