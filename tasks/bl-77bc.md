+++
title = "V3 seat: the fan group in the window — pick N, compare candidates, judge/synthesize/deliver affordances; graduates the Adjudicator story"
created = 1786845247
updated = 1786845247
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Source: VISION §5 V3 (bl-c2bd landed the mechanism; this ball is the §11 seat and the graduation). Verify VISION/DESIGN on main before editing; VISION wins over this body.

bl-c2bd landed: the Deliver-candidate gesture (Action::Fan(fan::Verb::Deliver), /deliver <handle> <summary...>, {"op":"deliver"}), the derived acceptance mark (fan::delivered_commit — the [<handle>] tag-scan over the target's history, no winner field), and the fan candidates as work-diff rows (workdiff::candidates: ruled work/<id>..attempt/<handle> range + delivered mark per candidate, patch drill-in addressed by ball AND handle, both frontends via Query::WorkDiff). Judge/Synthesize needed no primitive: they are V2's fire path with a goal carrying the candidates' exact terminal refs (the WorkDiff rows' source_oid per candidate); rework is Message + balls' stale-source refusal; rejection is the absence of a delivery.

This ball builds the seat V3.1/V3.3 render, over those mechanisms and nothing new:
- A fan group surface in the window: the cohort (fan::cohort::members joined with the candidate work-diff rows) rendered as one group — per candidate its handle, state, terminal response, usage, wall time, project churn, and the delivered mark as a rendered consequence of target history.
- Judge and Synthesize affordances on the group: seed the ordinary composer with a goal carrying each candidate's handle + attempt/<handle> tip OID + base + target (compose the goal in one pure fn; fire through the existing Prompt/Fork doors — add NO fan-in primitive).
- Deliver/Retire affordances spending the existing boundary gestures; an N picker that spends Action::Fan(Spread) then N ordinary Prompts.
- Response diff between candidate terminal responses (V3.3); wall time from step records — compose with bl-40ab's projection, do not duplicate its query.
- Graduation: write the Adjudicator story in docs/STORIES.md (V3's rung; note bl-c2bd's body said 'graduates as S12' — stale, S12 is V2's, graduated by bl-dc0c) with story tests and drive beats per QUALITY.md.

Composes with bl-40ab (attempt science projection); neither owns the other's surface.