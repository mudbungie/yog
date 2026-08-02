+++
title = "transcript: tool-use row keeps pulsing 'running' after the tool finished — its result file doesn't retire the badge"
created = 1785645745
updated = 1785645745
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-02, literal rendering (ops workspace, gpt-5.4 conversation):

    · ⚙ bash — running{"command":"pwd"}
    ▶ 021-tool.json[{"type":"tool_result","tool_use_id":"call_QxWh5oDZm5GNM4nbnIFVb7Ou","content":[{"type":"text","text":"/home/u/.local/share/yog/workspaces/ops/agents/2026080…
    · gpt-5.4:`/home/u/.local

The bash call's row pulses 'running' even though its tool_result exists on disk (021-tool.json) and the model already consumed it. Pulsing must only happen while the tool actually runs. Two smells visible in the rendering: (a) the result row shows as a bare '▶ 021-tool.json' filename — the file isn't classified as a tool result, so it can't retire the matching call's running state; (b) pairing is by tool_use_id ('call_…', OpenAI-style id) — check the matcher doesn't assume anthropic 'toolu_…' shapes or a different file naming. Fix the classification/pairing so a present result marks the call done; the pulse derives from 'result absent', not a stored flag. Also the raw '[{"type":"tool_result"...' preview suggests the -tool.json reader may want the same treatment the deposit envelope got in bl-6ec6 (parse, don't dump).