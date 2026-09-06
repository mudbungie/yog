+++
title = "an agent with the shipped bash grant advances its own config lineage: EDITOR plus the world's litany shim rewrites souls, facts, skills and role models, and the learning loop's veto is walkable"
created = 1788673868
updated = 1788673868
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