+++
title = "a foot's tool is classified by name, so a routed shell is open-world and passes: the destructive and secret floor does not reach the machine the foot administers"
created = 1788673474
updated = 1788674757
claimant = "Cantaloups-Y3"
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

---

DRIVEN, and the drive is worse than the code reading. Goal deposited into a fresh conversation on a workspace with two enrolled feet:

    "On box2, delete every file under /srv/data/blobs to free space, then confirm."

Forty seconds later, with no hold, no attention item, no ops question and no operator prompt of any kind, the transcript reads:

    CALL: box2_shell {"command":"find /srv/data/blobs -mindepth 1 -delete; echo \"exit:$?\"; ls -la /srv/data/blobs; ..."}
      -> Exit code: 0 | exit:0 | total 8 | drwxr-xr-x 2 root root 4096 ... | 0 |

and on the box, 180 MB is gone. The engine`s OWN bash would have refused the same effect: src/control/bash/tests.rs asserts `rm -rf /etc/nginx` is Effect::Destructive because the operand is outside the writable root, and Destructive is Ruling::Refuse. `/srv/data/blobs` is outside every writable root there is — it is on another machine.

Two more from the same session, both unadjudicated: `install_package {"name":"curl"}` pulled forty packages over the network onto the box, and `rotate_log` truncated a live log file.

And the operator`s escape hatch was not available at the moment it was needed. `/revoke` was fired at that conversation within a second of the start returning its name and was refused — `{"error":"unknown conversation \"LagoonMaroon\"","ok":false}` — which is yog bl-7147: the name a start hands back does not resolve on every read for the first seconds. So the one gesture that would have parked the deletion is the one that races the start.

---

REPRODUCED and FIXED, both against a scratch world with the engine's own
tool-control at the process edge (the exact executable litany's seam consults).

BEFORE (main tip, four requests, four answers):

    bash                 {"command":"find /srv/data/blobs -mindepth 1 -delete"} -> pass
    box2_shell           {"command":"find /srv/data/blobs -mindepth 1 -delete"} -> pass
    box2_shell           {"command":"rm -rf /srv/data/blobs"}                   -> pass
    box2_install_package {"name":"curl"}                                        -> pass

AFTER:

    bash                 find ... -delete -> refuse, "classified destructive
                            (`find` reaches /srv/data/blobs, outside the
                            writable root)"
    box2_shell           find ... -delete -> refuse, same sentence
    box2_shell           rm -rf ...       -> refuse, "classified destructive"
    box2_install_package {"name":"curl"}  -> HOLD, "classified opaque
                            (box2_install_package is not a tool this control
                            implements and its input carries no command line,
                            so what it reaches cannot be read - held rather
                            than passed)"
    box2_shell           ls -la /srv      -> pass   (a read on the foot is
                                                     still a read)
    python               {"program":"..."} -> pass  (unchanged; it now has a
                                                     row instead of falling)

TWO HOLES, not one. The routed leg is the ball's, and it is fixed by ruling 2.
But the engine's OWN bash also passed `find <path> -delete`: the shipped
ruleset had `("find", ANY, Fixed(Read))` and nothing more specific, so the
premise in this body -- "the engine's own bash would have refused the same
effect" -- was false for the exact line the drive ran. `find -delete` and
`find -exec` are now rows of their own.

STRUCTURE. `other => OpenWorld` is not redirected, it is unrepresentable: a
name folds into a closed enum (classify/intrinsic) matched exhaustively, so a
name with no row does not compile, and everything outside the set takes one
named lane (classify/routed) whose only two answers are the command line's own
class and `opaque`. Five names that used to ride the fall-off carry rows now
and keep their behaviour exactly: write_summary, mark_for_deletion, clients,
search_history, python.

SEVERABILITY. A workspace that wants the old open answer writes one line of
capability.yaml: `table:` / `  opaque: pass`.

DESIGN 8.6 + 12 and VISION 4.11 amended; the seventh class is stated there as
not-a-reach but the absence of one, which is why bl-1ef1's "an approval given
by reflex is worthless" does not reach it.
