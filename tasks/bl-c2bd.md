+++
title = "V3 Adjudicator: judge, rework, and deliver an attempt"
created = 1785719121
updated = 1785823885
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-dc0c"
on = "claim"

[[blockers]]
id = "bl-2b8c"
on = "claim"

[[blockers]]
id = "bl-8746"
on = "claim"
+++
VERIFY docs/VISION.md and docs/DESIGN.md on main before editing; VISION wins over this body.

Implement VISION §5 V3 over V2's N >= 1 attempt surface. With N > 1, attempts are alternative candidates for one delivery target.

- Judge and Synthesize are ordinary dispatches carrying exact candidate terminal refs; add no fan-in primitive.
- A judgment is an advisory committed result, not acceptance state.
- Rework is an ordinary fork from an attempt's exact agent/project state.
- Acceptance is successful recursive source-to-target project delivery. Call the action Deliver candidate, never Adopt.
- Delivery updates the parent obligation's branch; it neither closes that parent nor changes which branch the parent's later close delivers.
- Rejection leaves the target unchanged. Losing attempts remain inspectable until the project contract's retention rule removes them.
- Render response diff, usage, wall time, and, after project support lands, the ruled project diff.
- Derive cohort, provenance, and delivery outcome from committed refs, ancestry, messages, and delivery facts. Store no winner field.

The read-only Judge/Synthesize path uses V2. Do not graduate V3 until bl-2b8c and every project-binding/delivery implementation task it files have landed.

Graduates as S12.