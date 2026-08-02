+++
title = "picker: selection applies immediately — drop the Set button; one control scoped to the selected role, not worker+compactor buttons"
created = 1785645674
updated = 1785645682
claimant = "picker-fixer"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator feedback 2026-08-02 on the bl-bd89 picker (b392937), verbatim intent: 'I didn't click set. I don't think that we need that. just selecting should make it automatically apply. The worker/compactor split also doesn't need two buttons: you're setting whichever one you have selected.'

So: (1) choosing a provider/model in the dropdowns commits right then — no separate Set. The four refusal invariants from bl-bd89 stay (never offer/write an unroutable pair); refusal surfaces on the selection gesture. Custom-model free entry commits on confirm (Enter), since half-typed ids must not write. (2) Replace any per-role (worker/compactor) apply buttons with one control that acts on whichever role is currently selected — the role selection is the scope, no second button row. Keep DESIGN §9.4 in step. Mind: bl-68ac (hover sweep) is touching the same shell surface concurrently — expect merge, resolve by hand.