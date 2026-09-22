+++
title = "an agent with the shipped bash grant advances its own config lineage: EDITOR plus the world's litany shim rewrites souls, facts, skills and role models, and the learning loop's veto is walkable"
created = 1788673868
updated = 1790058389
claimant = "Urinalyses-E"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Scenario: lane KNOWLEDGE, round 1. A workspace bound to a notes corpus, no foot enrolled (thrall bl-f88f), so bash takes REMOTE §5.4 rung 3 and runs at the engine's own front door.

WHAT HAPPENED

Asked a conversation to "remember this durably so a NEW conversation knows it", the worker (shipped grant: apply_patch, bash, cd, dispatch, load_skill, message, python, read_file, search_history) discovered `litany config` on PATH — yog's own world shim, DESIGN §16.4 — wrote a shell script and ran it as $EDITOR:

    export EDITOR=<its own script>
    litany config <yog-data-root>/workspaces/<ws>

The script wrote `facts.md` into the checkout `litany config` opened. Result, from the workspace bare repo:

    $ git -C repo.git log --oneline config/default
    d78023e config: advance [config/default]
    ...
    $ git -C repo.git show config/default:facts.md
    # Durable facts
    - Tidewheel authoritative raw-observation retention figure: 45 days ...

The agent's own account: "Ran `litany config <workspace>` against this workspace, which opens an editor over a checkout of the `config/default` branch. ... The tool committed this as a real git commit on `config/default`."

THE CEILING, MEASURED BY HAND

Re-ran the same gesture from the world with two other editor scripts, on a scratch workspace:

- `souls/worker.md` — rewritten. `git show config/default:souls/worker.md` ends `# OWNED BY THE AGENT`.
- `providers.yaml` — the compactor's row changed from `model: claude-haiku-4-5` to `model: claude-opus-5`, committed. A role's model, provider and `tools:` grant are all in this file.
- `workflow.yaml` — the `tool_control:` block was deleted by the script and yog PUT IT BACK, exactly as the block's own comment promises ("rewritten whenever it drifts from the installed yog"). That one axis is defended.

Nothing else is. So an agent can rewrite every role's soul, the facts document, every workspace skill, and each role's model/provider.

WHY IT MATTERS

litany docs/DESIGN_LEARNING_LOOP.md §3 builds a staged `proposal/<reviewer-id>` branch and an operator `litany proposal --accept` precisely so that what an agent learns is a diff a person reads and vetoes. The reviewer role is confined to [apply_patch, read_file] for the same reason. A worker with `bash` reaches the same files directly and skips the veto. It is also a spend escalation (haiku to opus is a config edit) and a policy escalation (its own soul).

EXPECTED

The config lineage is control state. An agent's tool window should not reach the act that advances it — either the capability control refuses `litany config` (and the other lineage-advancing verbs) the way it refuses `rm` outside the writable root and `env` dumps, or the world's `litany` shim declines the `config` verb for a caller that is an agent, naming the proposal path as the remedy.

SEVERITY

p1: it is reachable by a model that was merely asked to remember something, with no adversarial prompting at all, and it silently rewrites the policy every other conversation in the workspace resolves.

RELATED: litany has no agent-facing way to record a durable fact, which is what sent the model looking. Filed separately in litany.

---

Closed from the litany side by litany bl-d273 (landed b17889d4 on litany main, round-1 fix lane L3, triage ruling 3).

`litany config` and `litany proposal --accept` — the only two acts that advance a `config/*` branch — now refuse when `LITANY_TOOL_ID` is set. That marker is on every tool subprocess the executor spawns (ARCH §3.3, litany bl-e8d7) and is owed by a routing host on any spawn it makes, so the exact escalation in this body is refused at the door: the `bash` step exports its `$EDITOR`, runs the world shim, and gets

    litany config: advance a config lineage from inside a step: LITANY_TOOL_ID is set, so this process is a tool invocation of a running conversation (ARCH §3.3). The config lineage a conversation runs on is advanced by the operator, or by a proposal the operator accepted — never from inside a step (docs/DESIGN_LEARNING_LOOP.md §3). Stage the change as a proposal and say so in your answer; `litany proposal <workspace>` is where the operator reads and accepts it

The refusal stands ahead of the root resolution and the transient checkout, so a refused call materializes nothing and reads no ref, and the lineage is byte-identical after it. Reading is not refused: `litany proposal` bare, an id, and `--reject` stay open to a step — reading is nobody's risk, and a rejection deletes a branch no lineage points at rather than advancing one.

Two facts for the yog side.

1. The seam widened. litany's `cmd::Fx` gains one field, `tool_id: Option<OsString>`, filled once at the binding from `cmd::seam::ENV_TOOL_ID` — the same shape `conv_branch` has, and for the same reason (the environment is per-process, and reading it inside the verb makes a beat load-sensitive). A linked consumer that constructs `Fx` must fill it when it takes the next litany pin. The exec binding needs nothing.

2. The motive is being removed beside the refusal. The agent in this scenario was not attacking anything — it was asked to remember something and had no lawful way (litany bl-3c11, same lane, in flight): a `remember`-shaped door that stages a `facts.md` patch as a proposal, settled by the `litany proposal --accept` the operator already runs. A refusal without that door leaves an agent with no answer at all, which is why the two balls are one pair.

litany docs amended in the same delivery: ARCHITECTURE §3.3 (beside the `LITANY_TOOL_ID` bullet) and DESIGN_LEARNING_LOOP §3 ("One writer per branch holds" — the sentence that stated the veto as a description now states it as a mechanism).

---

Verified END TO END at HEAD, and closed with a regression test plus the DESIGN ruling the code had no doc for.

WHICH SIDE MEETS THE EXPECTED CLAUSE: litany's door, not yog's capability control.

The ball offered two remedies. The one that landed is the second: litany refuses `config` and `proposal --accept` under `LITANY_TOOL_ID` (upstream bl-d273, in the litany 0.0.13 pin yog carries). yog's capability control (DESIGN §8.6) does NOT refuse the verb and should not: it adjudicates a bash COMMAND LINE, so it would have to recognize a verb inside a string, which a different spelling of the path defeats; and it governs yog-hosted conversations only, leaving the same act open elsewhere.

THE THREE LINKS yog OWES, each verified at HEAD:

1. src/tool_host/engine_act.rs — `const TOOL_ID: &str = "LITANY_TOOL_ID"` (line 145) is put on the front-door spawn in `run()` beside LITANY_CONV_REPO/LITANY_CONV_BRANCH. Beat: `every_spawn_carries_the_invocations_own_id` (src/tool_host/engine_act/tests/set.rs). The bash rung reaches the same `perform` — src/tool_host/subject.rs `Lane::Engine => super::engine_act::perform(...)`, selected by `performs(name)` = litany BUILTIN_TOOLS minus engine_act::NAMES, and `bash` is in that difference.
2. src/world/tools.rs `shim_script` — `#!/bin/sh` plus `exec <yog> litany "$@"`. No env is cleared or reset, so the marker survives the shim.
3. src/multiplex/litany.rs line 90 — `let tool_id = std::env::var_os(cmd::seam::ENV_TOOL_ID);` filled into `Fx.tool_id` unchanged. Beat: `lineage_refusal_reaches_through_the_binding` (tests/multiplex_litany.rs).

And the link that is upstream's: litany's `bash` built-in spawns `Command::new("sh")` with no `env_clear`, so a step's shell inherits the marker; the world PATH fold puts the shim first, so a bare `litany` in that shell IS yog.

WHAT WAS MISSING: a test of the CHAIN. Each link had a beat; none drove the gesture the ball measured. Added tests/litany_lineage.rs — out of process against the built binary, hermetic world, real `litany` shim: a `bash` step (`<shim> tool bash`, the block on stdin, the stdio contract on the env) whose command is `litany config <workspace>` with a scripted EDITOR that WOULD write facts.md. Marked: non-zero, the refusal names LITANY_TOOL_ID and `litany proposal <workspace>`, the editor sentinel is absent (the refusal stands ahead of the checkout), and `config/default` rev-parse is unchanged. Same for `proposal <ws> <id> --accept`. Unmarked, the same command lands and the tip moves — so the beat cannot pass by refusing everything. Mutation-checked: dropping the value out of `Fx.tool_id` reddens it.

DOC: DESIGN 16.4 gained the ruling beside the `yog`-shim one (bl-3ff4) — why the refusal is litany's and not 8.6's, the three links, `remember` as the lawful door, and the routed rung being out of the chain by subject (a foot holds no workspace repository, so there is no lineage there to advance).

RESIDUAL, named not fixed: the routed leg carries no tool id on the wire (src/tool_host/remote.rs `invoke` sends op/client/tool/input/cwd). That is not this escalation — the ball's scenario is the no-foot rung, and a foot has no config lineage to advance. A foot that reached back at this server's world would be outside REMOTE 12's front-door-only posture and is that component's ruling.
