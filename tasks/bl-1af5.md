+++
title = "a failed start leaves a half-created workspace directory, and every later start on that name is refused unknown workspace with no remedy"
created = 1788673621
updated = 1788675146
claimant = "Cantaloups-Y1"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 install lane (STORIES S0/S9). Sequel to the git-identity ball filed beside this one; the wedge is separable from its cause and will outlive any one cause.

## Scenario step

First conversation on a fresh world. The start's `litany new` fails part way
through (in the sighting, because the box had no git identity — but any failure
between the bare-repo creation and the config commit does this).

## Gesture and reply

    $ lernie start home "hi"
    {"error":"`new` failed (exit 1): … Author identity unknown …","ok":false}

    $ git config --global user.email u@example.com   # fix the cause
    $ lernie start home "hi"
    {"error":"unknown workspace \"home\" — none is enumerated here","ok":false}

    $ lernie workspaces
    {"kind":"workspaces","ok":true,"rows":[]}

The name is now permanently unusable: `start` refuses it as unknown, and
`workspaces` does not enumerate it, so nothing in the interface can see the
thing that is blocking. What is on disk:

    <yog-data-root>/workspaces/home/repo.git      # and nothing else

`rm -rf` on that directory recovers fully — the very next `lernie start home`
reaches the sign-in gate normally. Nothing says so.

## Expected

Either the create is atomic (build under a temporary name and rename on success,
so a failure leaves no directory at all), or the refusal names the directory and
what to do with it. The first is the better answer: it dissolves the class rather
than documenting one instance, and it matches the posture the rest of the start
flow takes about spending nothing on a refusal.

## Severity

p1. `home` is the fixed name the empty-world bootstrap uses without asking
(DESIGN §3.1), so the wedge lands on the ONE name a first-time user will type,
and the state that causes it is invisible to every read the interface offers.