+++
title = "Wordlist curation + bl identity charset check (gates Z1)"
created = 1784523881
updated = 1784523881
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Diligence for Z1 (bl-9769), per operator: (1) confirm bl accepts hyphenated identities as --as values — names also ride into the <id>-<claimant> bl-delivery worktree-path variant, so they must be path-safe; (2) curate the wordlist so no single word is a plausible human identity or the literal 'unknown' (bl's identity fallbacks are $USER then 'unknown'; avoid common first names and usernames), and record the wordlist's source + license. Deliverable: findings recorded here + the curated words.txt constraints stated in Z1.