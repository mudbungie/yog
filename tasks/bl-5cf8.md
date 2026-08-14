+++
title = "the descent graph needs a seat: where V1.3's two edges are drawn now that the gutter is gone"
created = 1786064834
updated = 1786685000
claimant = "Oxbow"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
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

---

Ruling on bl-8905 (Fretwork, 2026-08-11) moves this ball's cheapest candidate seat rather than removing it. **The altitude-1 descent tree is retired.** After bl-fa82 the §11 conversation list renders the descent-id membership itself — every visible row is the subtree rooted at its agent, indented, foldable — so the centre's compact tree was a second rendering of one fact on one screen, and bl-8905 deleted it (src/shell/members.rs, AppModel::conversation_members).

What that means here: this body proposes '§11 Altitude 1's descent tree in the conversations panel, which already renders the descent-id tree — the provenance relation, drawn as indentation' as the seat that costs least. **That surface no longer exists; its replacement is the altitude-0 conversation list, which draws the same provenance relation as the same indentation.** So the candidate is unchanged in substance and changed in address — read src/shell/conv_row.rs and src/nav/convs/expand.rs, not src/shell/members.rs, and note the list is foldable where the tree was always-open, which is a real difference for a picture: a graph whose edges are hidden behind a fold is not a picture of a shape.

Not deciding this ball. The 'attack it first' question it sets itself — what does a drawn graph solve that the fork label does not — is untouched by bl-8905, and 'close as declined, record in VISION V1.3 that the words are the rendering' is still the landing that ball's own body calls legitimate. DESIGN §5.1 #8 now reads 'Since bl-fa82 the §11 conversation list is a rendering of this tree, and since bl-8905 the only one', which is the premise to verify before acting on anything above.

---

RULING (Oxbow, 2026-08-13): DECLINED — a drawn descent graph earns no seat; the words are the rendering.

Argument: (1) Both edges already have exactly one home each. Provenance is DRAWN — the §11 conversation list's indentation is the descent-id forest (bl-fa82), and since bl-8905 its only rendering. Context is WORDED — the fork label in src/rail/cards.rs (from here / from <Name>@<oid> / from config/<name>), computed from the shared commit prefix and spent immediately, per bl-1802. A two-edge graph is a second rendering of both facts at once, the exact debt bl-1802 paid down for one of them. (2) The cheapest seat is disqualified by its own virtue: the conversation list (src/shell/conv_row.rs + src/nav/convs/expand.rs) folds — the all-collapsed default is roots only, so a context edge to a collapsed ancestor has nowhere to terminate; a picture whose edges vanish behind a fold is not a picture of a shape (Fretwork's own observation), and pinning the list open to keep it honest spends the fold on a stroke nobody asked for. Any honest seat is a new always-open panel — the thing bl-1802 just subtracted. (3) Untestable: acceptance tests hold paint to account through galley text (and even that only the input string, bl-bc06); a solid-vs-dashed stroke between rows is painter geometry no test in this harness could catch lying. What can't be tested mustn't be built. (4) No operator ask exists: bl-83f3's rulings wanted the taxonomy, the child card, and the streaming tail — the strokes were the 2026-08-02 rendering proposal, already amended away by bl-1802 for the context edge; nothing since has asked for the picture.

Escape hatch recorded in VISION V1.3: a whole-descent-at-a-glance surface, if ever asked for, is argued on its own evidence as a new surface, not as this taxonomy's missing rendering.

Edits: docs/VISION.md V1.3 (ruling paragraph replaces the 'bl-5cf8 decides' pointer), docs/DESIGN.md §5.1 #8 and #30 (open pointer replaced with the declined ruling). No code change; no follow-up ball.
