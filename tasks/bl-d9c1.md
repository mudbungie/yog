+++
title = "yog: DHT walks take ~12 s (lookup) and ~20 s (put/get, at the 64-query cap) from the deployed engine box since the bl-d00f frontier"
created = 1790394224
updated = 1790394224
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
bl-d00f made the walk reliable from the deployed engine box (10/10 lookup, 10/10 put, 9/10 get over ten trials) by converging on the K closest RESPONSIVE nodes and re-asking the bootstrap when the frontier runs dry. The price, measured in the same run (REMOTE §13.7 ruling 3): lookup median 12.6 s / p90 18.1 s, put 19.2 / 21.3 s, get 16.5 / 20.7 s — about twice the pre-fix times, and put/get usually reach the 64-query cap.

Why, from the traces: near a target most nodes are silent (stale routing entries), and a round is synchronous — any silent query in it costs the whole 1 s deadline — so the tail of a walk is a run of 1 s rounds each yielding zero or one reply. The 15 s poll cadence (§13.4) is now shorter than a typical get.

Directions to evaluate, not decided: an asynchronous alpha (refill a slot as each answer lands, time out per query rather than per round); widening alpha only on rounds that were mostly silent; for get, whether a walk may stop once K token holders replied rather than once the K closest responsive have been asked. Each must be re-measured live the same way bl-d00f was (ten trials per verb, throwaway static probe, default Config).