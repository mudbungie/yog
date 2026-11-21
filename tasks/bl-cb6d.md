+++
title = "The router's wait for a routed tool counts polls, so its two minutes run about three"
created = 1794986956
updated = 1795234712
claimant = "Rabid-fix"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["inference"]
+++
Found by the two-machine drive (bl-faf3, REMOTE 5.7 'The drive, as observed'). tool_host::remote::invoke waits patience.waits polls of patience.tick each, and every poll is also a deposit round trip through the gestures inbox, so the bound the in-band refusal names is shorter than the time actually waited: the bz arm's identical loop measured a little over three minutes against a stated two, and was changed to a span of the clock in bl-faf3. Do the same here: wait until patience.span() has elapsed, not until a count runs out, and keep the one stop check where it is.

Tested by: the router's deadline case driven with a zero span and exactly one poll scripted, as the arm's now is.