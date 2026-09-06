+++
title = "the workspace is the trust domain on the wire and not on the disk: an agent's bash at the engine's front-door rung reads every other workspace's goals and transcripts"
created = 1788673805
updated = 1788674524
claimant = "Cantaloups-Y3"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 multitenant lane. DESIGN §3.1 ("a workspace is an app-wide blast
radius … different sets of conversations, settings, providers, all of it"),
REMOTE §1.5 (the workspace is the whole trust domain), §5.4 (`bash` reaches
"the engine's own front door where none consents", the lane's last rung).

## Setup

One engine, two workspaces `alpha` and `beta`, no thrall enrolled anywhere, so
`bash` takes the last rung and runs on the engine's own box.

## Gesture and reply

From a conversation in **beta**, one `bash` call:

    ls ../../../alpha/agents/*/

    Exit code: 0
    ../../../alpha/agents/<agent-id>/:
    ESSAY.md  descriptions  goal.md  messages  name  soul.md  spec.md

    ../../../alpha/agents/<agent-id>/:
    a1.txt … a8.txt  descriptions  goal.md

and separately, `ls ../../../alpha/agents | wc -l` answered `10`.

`goal.md` and `messages/` are the other workspace's transcript; the sibling
files are its agents' working trees. A `cat` of them is the same call.

## What is working, and why that makes this sharper

Adjudication is not asleep — the same conversation's second call was refused
in band:

    "bash" was refused by the workflow's tool control (ARCH §3.3 Tool control):
    bash {"command":"env | grep -E \"YOG_WALL|LITANY_HOME|XDG_\""}
    classified secret (dumps the environment)

So the classifier bites on a secret-shaped command. A relative `ls` up three
directories is not secret-shaped, and no classifier should have to be the thing
that holds a trust boundary.

The wire half is enforced structurally and correctly: a client registered only
in `alpha` gets the resolver's own `unknown workspace "beta"`, and the walls
are genuinely separate on disk (`world/walls/alpha/brazen/` and
`world/walls/beta/brazen/`, each with its own config and model cache). It is
only the filesystem the agent stands on that carries no boundary.

## Expected

The last rung either confines the working directory to the workspace it belongs
to, or DESIGN records — where the blast-radius ruling is written — that
filesystem confinement between workspaces is not claimed, so an operator
choosing to host two spheres on one engine knows what they are choosing.
REMOTE §5 already carries an "Honesty about containment" bullet, but it is
about a machine the adjudicator cannot inspect; this is the engine's own box.

## Severity

p2. It needs a co-tenant to matter, which is exactly the configuration this
lane exists to test and exactly the one the four-component split invites. It
takes one obvious command and no privilege.