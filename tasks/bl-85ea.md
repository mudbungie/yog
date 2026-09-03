+++
title = "drive preflight refuses on a world-seed file no founded world holds: template/providers.yaml is an operator override, and it gates the whole drive family"
created = 1788235195
updated = 1788407718
claimant = "Spellbind-B"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "drive"]
+++
## Symptom

On a box whose world was founded by current code, `make drive-preflight`
refuses and nothing is driven:

```
required — every run verb dies without these:
  models.yaml    OK       <home>/.local/share/yog/world/litany/models.yaml
  providers.yaml MISSING  the workspace-birth template — the role→provider rows …
     want: <home>/.local/share/yog/world/litany/template/providers.yaml

preflight: 1 required prerequisite(s) missing — nothing was driven.
```

`make drive` and `make drive-cleanroom` both run `preflight.sh` before their
first world, so the whole family is refused.

## The file is optional, and nothing founds it

A world founded by this build holds `litany/models.yaml`, `skills/`, `tools/`,
`workflows/` and `workspaces/` — and **no `template/` directory at all**.
Verified twice: on a world founded today by a bare `yog` boot in a scratch
`XDG_DATA_HOME`, and on a second world founded a day earlier.

The seeded `models.yaml` says what the file is, in its own words:

> Which model each role uses is the conversation repo's `providers.yaml`
> `roles:` section (ARCH §4.3) — authored per repo, **overridable install-wide
> via `<config-root>/template/providers.yaml`**.

An install-wide *override*. It exists only where an operator hand-wrote one, so
the harness's required tier names a file whose absence is the normal state of a
newly founded world.

## The file's own text disagrees with its tier

`preflight.sh` prints, under the wire section:

> Nothing here blocks a run: since bl-00ee retired the §9.2 birth gate a
> workspace is born whatever its template names, and a row its wall lacks
> surfaces at the first dispatch (§8.3), not as a refusal to create anything.

and its header says of the provider fixture "It is ADVISORY, not required
(bl-00ee)". QUALITY §3 step 0 says the same: *"Both are advisory since bl-00ee
retired the §9.2 birth gate."* The `seedfile` call is nevertheless in the
required tier and exits 1.

The tier is not simply wrong, though — `stories.sh`'s `seed()` copies the file
unconditionally under `set -e`:

```
  cp "$real_world/template/providers.yaml" "$data/yog/world/litany/template/providers.yaml"
```

so the requirement is honest about the implementation. **One of the two has to
move**, and the question the fix answers is which: either the seed tolerates an
absent template (a scratch world then births on the shipped role rows, which is
what a real fresh install does), or the preflight keeps refusing but says the
file is the operator's to author and prints the shape.

## Repro

    # on any box with no <config-root>/template/providers.yaml
    make drive-preflight            # exit 1, nothing driven

Confirmed the other direction too: authoring a two-role
`template/providers.yaml` under a synthetic `HOME` makes preflight report
"every required prerequisite present" and the ladder proceeds.

## Note

This is what a fresh agent hits **first**, before any beat: the drive protocol's
step 0 refuses on a file the product never creates, so the ladder reads as
unrunnable rather than as unconfigured.