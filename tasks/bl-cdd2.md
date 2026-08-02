+++
title = "design-doc jargon leaks into operator UI: '§9 — stage → validate → hash-guard → atomic rename', 'project marks (bl store branch)'"
created = 1785646882
updated = 1785646882
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
The Config pane's subtitle is '§9 — stage → validate → hash-guard → atomic rename' and a section header reads 'project marks (bl store branch)'. Section references and internal mechanism names are for DESIGN.md, not the operator; an operator cannot dereference '§9'. Fix: say what the surface does in operator words ('edits are staged and validated before they land; a file changed underneath refuses the apply') or say nothing. Sweep the UI for other §-references. NOTE: bl-c225 (claimed) overhauls the config pane — check its delivery first; if it rewrites these strings, close this as subsumed.