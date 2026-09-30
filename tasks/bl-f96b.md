+++
title = "on restart the engine re-punches every stale call still in the inbox: the already-punched memory is per process, so a boot after a deploy spends a 20 s punch window per old nonce"
created = 1790732916
updated = 1790732916
priority = 4
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Seen 2026-09-29 on the deployed engine's first minutes after the 0.0.74 restart: two old nonces from the previous day's dials were opened and punched ('punch started' then 'expired after 20s with no stream' for each). Harmless but wasteful, and it will mislead an operator reading the log after a deploy. A call carries seq = the caller's unix time (REMOTE §13.3); a call older than the caller's punch window (35 s) plus one poll cannot be answered by anyone still listening, so the poll should treat a call whose seq is older than that bound as already spent and say so once ('call nonce N is stale (age Ns) — no punch'), instead of persisting the nonce memory across restarts. Bench test: a call with an old seq is not punched and is said once.