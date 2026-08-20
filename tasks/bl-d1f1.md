+++
title = "a lost boundary reply has no safe recovery: re-deposit can repeat a completed non-idempotent action"
created = 1787206349
updated = 1787206349
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design", "boundary", "security"]
+++
`boundary/deposit.rs` describes crash recovery this way:

> “A crash between claim and reply leaves the claimed file as debris naming exactly what was in flight — re-deposit to re-run.”

`README.md` also says exit 124 means no consumer answered while the deposit remains and may converge later.

That is an ambiguous outcome for non-idempotent actions. Create, update with a journal note, message, prompt and routed execution can land their effect before the reply is lost. Re-depositing the same request may perform it twice; waiting may also perform it later. The caller has no claimed-gesture query, terminal-status query, resume/abandon verb, or downstream idempotency key with which to distinguish pending from committed.

## Required design result

Make one recovery rule safe for every action. Likely ingredients are a stable request identity and a durable terminal receipt that a retry can read rather than re-execute, but the design should attack simpler alternatives first. Specify and drive the crash windows: before claim, after claim/before effect, after effect/before reply, and after reply.