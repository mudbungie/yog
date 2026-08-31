+++
title = "a conversation's first driver can load a remote tool but can never call one: `prompt` mints the agent, so `Injection::driving` is None and `tools()` declares only `clients`"
created = 1788150343
updated = 1788150949
claimant = "OrderArbiter"
priority = 9
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["remote", "tool-host"]
+++
A conversation's FIRST driver is the one `lernie start` fires, and the verb it
fires is `prompt` — which mints the agent rather than naming one. So
`Injection::driving` is `None` for that whole process, and
`ToolInjection::tools()` (`src/tool_host.rs`) takes its `if let Some((workspace,
agent)) = &self.driving` branch not at all: it declares the `clients` tool and
nothing else, for every step that driver takes.

The `clients` tool still works. `op=load` reads the roster, writes the loaded
document, and answers, verbatim:

    Callable from the next step on. This conversation now holds 3 loaded tools.

That sentence is false in this process. Nothing was declared, so litany's grant
gate (`prompt/dispatch/tool_step/permit.rs`) refuses every call of a loaded name
with its inherited-transcript decline, and the toolset it enumerates back is the
role grant plus `clients` — the loaded names are absent from it, which is the
gate telling you exactly what `tools()` returned.

The field's own doc comment states the mechanism and files it as harmless:

> A verb that names no agent (`prompt`, which mints one, and every operator
> verb) declares the `clients` tool and nothing else, which is exactly what a
> conversation with no loads reads as.

It is not the same. A conversation with no loads has not been told it holds
three. `prompt` is not an operator verb that incidentally names no agent — it is
the conversation's entire first run, it is where a model discovers the roster,
and it can load.

## Repro, and the control that proves it is the verb

One workspace, one foot registered in it, three advertised tools. Two runs of
the SAME conversation, the same `op=load` call with the same arguments:

1. `lernie start <ws> <goal>` — the model runs `clients op=list`, `op=get`,
   `op=load`; the load answers ok; the very next step's calls are DECLINED by
   the grant gate. The model reports it cannot act and gives up. The whole run
   is spend for nothing.
2. `lernie message <ws> <agent> "try again"` on that same conversation — the
   deposit resumes it under a verb that NAMES the agent. Identical `op=load`,
   and the next step's calls run, route to the foot and return captures. The
   task completes.

Nothing about the workspace, the roster, the advertisement or the loaded
document differs between the two. The only difference is which verb launched
the driver.

Seen in one further conversation from the other direction: its first driver's
load was never reached (the foot was not yet registered), the operator
deposited, and the SECOND driver loaded and worked first try. Every observed
success loaded in a resumed driver; the one load in a `prompt` driver failed.

## Why this is the worst version of the bug

- It is the default path. A new conversation is started with `start`, and
  `start` is `prepare` + `prompt`.
- It fails silently in the wrong direction: the act that would tell you
  succeeds, and the failure lands one step later on a different surface, with a
  message about the inherited transcript that names no cause a reader could act
  on.
- Under the total router (see the sibling ball on the dead grant) a loaded
  remote name is the ONLY tool that can do anything at all. A first driver that
  cannot call one can do nothing but talk.

## Shape of the fix

`prompt` mints the agent id — it is known inside the driver process by the time
the first step is assembled, which is after the mint. Either hand it to the
injection once it exists, or let `tools()` resolve the agent the way the rest of
the driver does instead of reading it off argv. The seam being "per-process, not
per-agent" is the premise to re-examine: a process that mints exactly one agent
knows which one.

A second, cheaper guard regardless: `op=load` should not promise callability it
cannot deliver. If the injection knows it is declaring nothing, the load is a
refusal, not an ok with a sentence about the next step.