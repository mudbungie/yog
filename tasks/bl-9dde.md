+++
title = "operator rescue ruling 2026-08-11: amend DESIGN/QUALITY in place — workspace blast radius, composer focus, per-agent task branches, keyboard everywhere with combos on hover, overlays become tabs, conversation settings at the bottom"
created = 1786508775
updated = 1786508775
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
yog is judged failed on two counts: the UI is confusing and obtuse, and the
workflows don't work. This ball is the design half of the rescue: amend the
authorities IN PLACE (edit the governing sections — no addendum sections, no
appended rulings list). Implementation balls are filed separately and gated on
this one.

Operator ruling, verbatim (2026-08-11):

> Top level design: workspaces are an entirely separate space; essentially an
> app-wide blast radius. Different sets of conversations, settings, providers,
> all of it. When you open the app, focus to the chat prompt. When you select
> an agent, focus to the chat prompt. The balls interface is broken right now.
> The default is going to be that each agent gets its own balls branch for
> tracking, but that an agent can have its branch set at launch, and that
> subagents, by default in yog, get their parents' space passed to them.
> Obviously an agent can amend their own branch, to change their config
> (necessary when an agent is launched but told to work on a project). Everything
> should be keyboard-operable, and mouseovers on all buttons should indicate the
> combo to implement it. Several places (config, eg) are interface overlays
> instead of tabs (toggle on), but cover everything so really should just be a
> tab focus. Move all the settings for a conversation to the bottom, instead of
> the top.

The amendment map (verify each cited section against the current tree first —
ball bodies have drifted before):

1. WORKSPACE = APP-WIDE BLAST RADIUS. DESIGN §3.1's wall ("conversations,
   config, and the balls it claims") extends to providers and ALL settings.
   §16.2's "brazen's config/credentials/cache stay ambient — shared" is
   REVERSED: providers live inside the wall, per workspace. Repo AGENTS.md's
   §16 summary line repeats the ambient-brazen claim — amend it too. Sweep §1
   taxonomy and §2 invariants for consequences.

2. FOCUS LANDS IN THE COMPOSER. §11 "Focus discipline": launch → composer
   already holds; selecting an agent must now land the composer UNCONDITIONALLY
   — including keyboard selection, which rule 2 ("a keyboard gesture leaves the
   keyboard plane alone") currently exempts. Amend rule 2 deliberately and
   reconcile the roster-walk concern in the doc (don't silently keep the old
   rule beside the new one).

3. PER-AGENT TASK BRANCHES. Rewrite §16.3 ("shared by default, with a no-marks
   knob"): the DEFAULT is each agent gets its own balls branch for tracking; the
   branch can be set at launch; subagents by default get their parent's space
   passed to them; an agent can amend its own branch (the launched-then-pointed-
   at-a-project case). The shared-store default and the stealth framing are
   superseded; the bl-conf mechanism may survive as the write path.

4. KEYBOARD EVERYWHERE, COMBO ON HOVER. §11 discoverability invariant ("every
   interactive control carries on_hover_text stating what pressing it does")
   extends: the hover also names the key combo. QUALITY F1 stands as written —
   this ruling RESOLVES the open F1-vs-§11 contradiction (everything operable
   gets a keyboard spelling, pins included). Re-audit §11's "deliberately
   unbound" list against "everything should be keyboard-operable".

5. OVERLAYS BECOME TABS. A surface that covers the whole center as a toggled
   overlay (Config is the named case; enumerate the others — Login, world
   search, any full-cover toggle) is reseated as a tab focus in §11. Modals
   that own the frame for one small form (new-workspace) are not the target.

6. CONVERSATION SETTINGS AT THE BOTTOM. §11 altitude 1: the header's
   config-shaped rows (model line + change… picker, budget/spend figures — the
   settings, not the identity line) move to the BOTTOM of the conversation
   surface, beside the composer. Decide and record where the birth-config block
   (bl-824e) sits under the same rule.

Deliverable: docs/DESIGN.md (+ docs/QUALITY.md and repo AGENTS.md where cited)
amended in place. No code changes. Where an amendment contradicts a standing
"operator ruling" paragraph, the new ruling supersedes and the paragraph is
rewritten, not annotated.