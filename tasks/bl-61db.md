+++
title = "activity rows lead with raw epoch seconds — render human time"
created = 1785646885
updated = 1785646885
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Expanded activity accessory rows read '· 1785630266 lernie new /tmp/…'. A raw unix epoch is unreadable; the operator wants 'how long ago / when'. Render human time (HH:MM:SS or ISO8601, matching the chat header's convention from bl-16da) as the leading column; the epoch can stay in the expanded row detail if it earns its place. One source of truth for timestamp formatting shared with the transcript header.