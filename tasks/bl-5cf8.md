+++
title = "the descent graph needs a seat: where V1.3's two edges are drawn now that the gutter is gone"
created = 1786064834
updated = 1786064834
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Filed by bl-1802, which dissolved the history rail's `SidePanel` gutter into
the chat (one horizontal rule per operable commit) and, in doing so, retired
the one surface that drew VISION V1.3's **two edges** as strokes.

## What bl-1802 decided, so this ball does not re-litigate it

V1.3's taxonomy is **not** dropped. Both edges are still derived, in
`src/rail/cards.rs`, off the shared commit prefix of parent and child
(`Agent::steps`), at no git cost:

- the *context* edge — what the child inherited — is git ancestry;
- the *provenance* edge — who dispatched it — is the descent id plus the
  dispatch notch.

What was dropped is their **second rendering**. The gutter drew them as a solid
and a dashed vertical stroke above each card; a rule across a chat has no
column to stroke in. The distinction survived in words, on the card's own fork
label — `from here` / `from <Name>@<oid>` name an ancestry, `from config/<name>`
names a clean child that has none — which the card already carried, so the
strokes were a second home for a fact the label states. bl-1802 also deleted
`ChildCard::context_notch`, the stored index only the stroke read.

VISION V1 item 3 now records this verbatim, and names this ball as where the
graph question is decided.

## The question this ball answers

**Is a drawn descent graph worth a seat, and if so which seat?** An operator
looking at a fleet may want to see the shape of a descent — who forked from
whom, where ancestry diverges from provenance — as a picture rather than as a
label per card. The chat is not that seat: a conversation is linear and one
agent's, and the graph is neither.

The candidate that costs least is §11 **Altitude 1's descent tree** in the
conversations panel, which already renders the descent-**id** tree (§5.1 #8) —
the provenance relation, drawn as indentation. Adding the *context* edge there
would put both facts on one picture that already exists. DESIGN §5.1 #8 is
explicit that §11 membership stays the descent-id tree, so any change there is
a doc edit first.

Attack it before building: what does a drawn graph solve that the fork label
does not? If the answer is "nothing an operator asked for", the right outcome
is to **close this ball as declined** and record in VISION V1.3 that the words
are the rendering — that is a legitimate landing, not a failure.

## Verify before editing

Written 2026-08-06 against bl-1802's tree. Check `src/rail/cards.rs`,
`src/transcript/spine.rs`, `src/nav/convs/`, DESIGN §5.1 #8 / #30 and
VISION V1.3 against HEAD before acting on any premise above.