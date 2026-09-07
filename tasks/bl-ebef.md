+++
title = "pin litany =0.0.11 and brazen =0.0.17: Fx.tool_id, stop children retired, remember built-in, from_name on deposits, max_output_tokens role key, OutputTruncated is a failure not a rest"
created = 1788743327
updated = 1788743327
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Release-train step after the round-1 yog lanes land (Y1 birth, Y2 enrol, Y5 seat-facing; Y3/Y4 done). Move both pins together: litany =0.0.10→=0.0.11 and brazen =0.0.14→=0.0.17 (litany 0.0.11 links brazen 0.0.17; the lockfile's one-brazen resolution is the parity check).

Migration, from the litany release lane:
1. Compile break: cmd::Fx gained `tool_id: Option<OsString>`; fill from `cmd::seam::ENV_TOOL_ID` beside conv_branch in src/multiplex/litany.rs. Pass the REAL LITANY_TOOL_ID through: litany config / proposal --accept refuse under it (settles yog bl-baed — close it with this ball, citing litany bl-d273).
2. stop lost its children mode (litany bl-3114): stop_children on agent views/rows is a word with no mode behind it — see yog bl-6efc; retire or document as accepted-and-ignored on this PROTOCOL bump.
3. cmd::BUILTIN_TOOLS is now 10 (`remember`, a facts proposal): tool_host::subject must classify it; the exemption ledger moves by one.
4. Deposits carry `from_name:` beside `from:` (litany bl-a457): src/transcript/read.rs takes sender from the filename — read the envelope (yog bl-6661, close with this ball).
5. `max_output_tokens:` per-role providers.yaml key; new Error::OutputTruncated is a named failure, never committed — the seat's attention row must render a failure, not 'came to rest — your turn' (litany bl-155f). Update src/test_support.rs's providers.yaml copy.
6. Template retune (litany bl-ce09): compaction n 60, extract_bytes 8192, tool_output 2 KiB+2 KiB; worker grant lists remember; schemas/tools/remember.json new, apply_patch.json changed — refresh anything yog vendors or asserts.
7. litany mint's contract: per-creation entropy (yog bl-d88f, Y1's).
Then: release yog (release-plz auto-merge), both engines take it by reconciler; lernie bl-183b and yog-android re-vendor follow.