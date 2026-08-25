+++
title = "detached-row failure should be a state predicate, not sink content: apply the driver.log rule to the stderr sink"
created = 1787379499
updated = 1787622450
claimant = "Cleat"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Follow-on to the notice-classification ball: DESIGN:9296 rules that driver.log content is never the trigger — only a derived state alarms, the log supplies diagnosis. The detached stderr sink (src/opslog/detached.rs) violates this: classification reads bytes. The right endpoint is a state predicate — the driver no longer holds the agent's lock (§3.5 liveness) while the conversation shows no reply — with the sink tail as diagnosis only, reusing the wound/orphan machinery incl. the bl-90bf grace window (src/steps_view/orphan.rs:14-33 is the template). Subsumes the notice marker list; when this lands, opslog::notice can retire. Also dissolves the append-only-sink permanence (one old notice keeps the tail non-empty for the driver's life). Blocked on appetite, not on code — the notice ball fixes the operator-visible defect first.