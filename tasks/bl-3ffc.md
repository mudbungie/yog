+++
title = "steps tab: add column headers — the fields are unlabeled and unreadable"
created = 1785645586
updated = 1785645586
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-02: the steps tab shows rows of values with no headers; it's not clear what the columns/fields mean. Add headers (or per-field labels if it's not a grid) naming each column in operator terms. Find the steps view (src/steps_view/ or current post-split home), enumerate every field it renders, and label them; a header tooltip may carry a one-line explanation where a word isn't self-evident (same pattern as the Workspaces label, bl-2d87). Amend DESIGN where it describes the steps tab.