+++
title = "a foot's tool is classified by name, so a routed shell is open-world and passes: the destructive and secret floor does not reach the machine the foot administers"
created = 1788673474
updated = 1788673474
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round 1, devadmin lane. STORIES S15 (Warden) says nothing runs unadjudicated.

WHAT HAPPENS

`src/control/classify.rs` dispatches on `request.name` against seven built-in
names and sends everything else to one arm:

    other => Classified::new(
        Effect::OpenWorld,
        format!("{other} is not a tool this control can classify"),
    ),

A routed foot tool arrives under its loaded name — `box2_shell`,
`box2_install_package` — which is never one of the seven, so every routed call
is `OpenWorld`. The shipped table (`src/control/judge.rs`) is:

    Effect::Read | Effect::TargetWrite | Effect::Process | Effect::OpenWorld
        => Ruling::Pass,
    Effect::Destructive | Effect::Secret => Ruling::Refuse,

So every routed call passes, unconditionally, and the two classes that exist
to be refused are unreachable on that leg. `src/control/bash.rs` — the one
place that reads a command line and can answer `Destructive` — is keyed on the
built-in name `bash` and never sees `box2_shell`.

The consequence, stated plainly: the engine`s own `bash` running `rm -rf` or
`git push --force` is refused in band; the SAME line sent to a foot as
`box2_shell {"command": "..."}` is passed without a word. The control is
strictest about the machine the operator is sitting at and blind about the
remote boxes a foot exists to administer — which is backwards, because the
foot is the leg whose blast radius the operator cannot see.

Observed: a conversation in workspace `ops` loaded `box2_shell` and ran
fourteen shell lines on the container, including `find / -iname ...` and an
`apk add`, with no hold, no attention item and no ops-trail question. The
transcript records every capture, so the audit trail is there; the
adjudication is not.

EXPECTED, and this is a ruling to make rather than a patch to apply:

- A routed call carries the foot`s own declared tool name and its input. The
  control can classify a routed name whose input schema has a `command`-shaped
  string through `control::bash` exactly as it classifies the built-in — but
  only if it decides a foot`s shell is a shell.
- Or the class is right and the RULING is wrong: a routed call reaches a
  machine the adjudicator cannot see into (REMOTE 5.4`s honesty clause), which
  is an argument for `OpenWorld: hold` on the routed leg specifically rather
  than for classifying it finer.

Either way the current answer — silently the most permissive class, by falling
off the end of a `match` — is the one answer nobody chose.

SEVERITY p1: it is the safety property S15 names, on the leg that has the
widest reach, and it is reached by the ordinary use of the feature.