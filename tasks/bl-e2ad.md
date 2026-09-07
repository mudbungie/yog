+++
title = "every path-rung conversation is named by the engine's own preface, so the roster and the attention queue are a column of identical absolute paths and the start receipt's name addresses nothing"
created = 1788745973
updated = 1788752385
claimant = "Cantaloups-Y9"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r2"]
+++
Round 2, lane swdev. yog main 5cd95986, litany =0.0.10, seat driving `lernie start <ws> <goal> <dir>`.

## The gesture

    lernie start swdev "The test suite in this project fails: … Commit the fix." ~/w/pyshop
    {"kind":"prepared","ok":true,"prepared":{"binding":"~/w/pyshop","goal":"Working directory: ~/w/pyshop\nThis is already your current directory: every tool call starts there and relative paths resolve there. Keep the work inside it.","…":"…"}}
    {"conversation":"HarvestCavern","kind":"started","ok":true}

Three such starts, then:

    lernie conversations swdev
      Working directory: ~/w/pyshop    stopped  2m  1 member  1 waiting
        "Working directory: ~/w/pyshop"
      Working directory: ~/w/rsroman   stopped  2m  1 member  1 waiting
        "Working directory: ~/w/rsroman"
      Working directory: ~/w/tsreport  stopped  2m  1 member  1 waiting
        "Working directory: ~/w/tsreport"

and the same in `lernie attention`. The frame confirms it is not a rendering
choice — `display` and `preview` are both the preface line:

    {"display":"Working directory: ~/w/pyshop",
     "preview":"Working directory: ~/w/pyshop",
     "root_id":"20260906T102556Z-12570b20", …}

## Two faults, one cause

1. The description is cut from the first line of the prompt, and on the path
   rung the first line is the engines own preface (bl-fea6s replacement
   text), never the operators goal. Every path-rung conversation in a
   workspace is therefore named the same shape, and the one column that must
   distinguish them carries the machinery instead of the work.

2. `HarvestCavern` — the name the start receipt hands back, and the only
   handle the operator was given — is not a handle:

       lernie steps swdev HarvestCavern
       refused: unknown conversation "HarvestCavern"
       lernie --json agent swdev HarvestCavern
       {"error":"unknown conversation \"HarvestCavern\"","ok":false}

   The `root_id` works. Nothing in the receipt names it. The lane recovered
   only by reading `--json conversations` and matching on the bound path.

By contrast a bare-rung conversation IS named (`ClampTurtle`, `BriocheGorge`),
so the roster reads: seven compactors with memorable names, five workers all
called `Working directory: /home/…`.

## Expected

The description is cut from the operators goal, not from the preface the
engine prepended to it; and the name the start receipt returns is a name every
verb accepts, or the receipt returns the id instead.

## Severity

p2. Nothing is lost, but the roster is the operators index and it currently
indexes by a string the operator did not write and that is identical for every
row. Coding work is the shape that always takes the path rung.

---

Fault 1 is fixed here and fault 2 is not this repo's, on the evidence.

Fault 1 (the description is the engine's preface): fixed. `start::path_preamble`
now composes from two constants and `start::strip_path_preamble` is its
inverse beside it; `git_tree::detect::payload_headline` applies both inverses —
the legacy identity stamp, then this one — so the §3.3 ladder's rung three is
the operator's own first line. A fire with no words of the operator's own keeps
the directory line, because there is nothing else to show and dropping to the
id would say less. DESIGN §3.3 amended in the same close.

Fault 2 (the receipt's name addresses nothing): not reproduced as an
addressing defect, and it is very likely a consequence of the dead birth
bl-6495 covers. `boundary::address::resolve_agent` already resolves a minted
name three ways — id-shaped straight through, the published derivation's
`(id, stored name)` set, then disk with a three-second settle for the branch a
fire has not had swept yet. Rungs two and three both read litany's stored
`name` blob, which litany writes on the DISPATCH commit that creates the agent
(its `workspace::agent_name`: "one home: a `name` file on the agent's own
dispatch commit"). So `unknown conversation "HarvestCavern"` means no branch
carried that blob — which on this drive is the same driver death that left
`exit -2` with an empty stderr and no step. The lane's own frames say the
conversations were `stopped` with `1 waiting` and had taken no step.

If it recurs on a drive where the birth SUCCEEDS, that is a different ball and
worth filing with the `--json conversations` row beside the receipt: the pair
(receipt name, row `name`) is the whole evidence, and this repo can then fix it
in `resolve_agent`. Re-drive is the next round's.
