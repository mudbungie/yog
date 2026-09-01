+++
title = "the wire refusals name commands the operator cannot run: 'make wire-certs' on a box with no checkout, and env-only settings spelled as argv that yog wire-certs accepts, ignores and calls success"
created = 1788235220
updated = 1788235474
claimant = "Roamer"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "wire"]
+++
## Two wire refusals hand the operator a command they cannot run

### 1. `make wire-certs` on a box with no checkout

`src/wire/material.rs` holds one remedy const for the whole wire:

```rust
/// The target that mints the lot, named in every refusal so a seat that cannot
/// start says how to make it start.
pub const REMEDY: &str = "make wire-certs";
```

and three refusals spend it — the half-provisioned one, the address-less one,
and `/enroll`'s "holds no wire material … run `make wire-certs` where the CA
lives".

A deployed engine is an installed binary or the OCI image (DESIGN §10.1). It
has no Makefile and no repository. REMOTE §8 already made exactly this argument
when it retired the script:

> `scripts/wire-certs.sh` was the recipe until bl-ae05 and is retired — **an
> installed binary has no repository to find a script in**, and two spellings of
> one act drift within a week; `make wire-certs` runs the verb…

The same sentence condemns `make wire-certs` as a *remedy*: the verb is
`yog wire-certs`, and it is the one spelling every box has. `make wire-certs`
is the developer's convenience wrapper over it, not the act.

### 2. The `:0` refusal spells env settings as argv, and running it literally
succeeds at the wrong thing

`/enroll` on a self-provisioned box refuses with:

> State the endpoint — `yog wire-certs WIRE_HOST=<host> WIRE_PORT=<port>`

`yog wire-certs` reads those six settings from the **environment**
(`verb::READS`, performed at `main.rs`'s process edge); it parses no argv at
all. Follow the sentence literally and the words are silently discarded:

```
$ WIRE_DIR=<empty-dir> yog wire-certs WIRE_HOST=engine.example.com WIRE_PORT=9999
yog wire-certs: <dir> holds ca.pem, ca.key, address, …
  the engine binds and a local seat dials 127.0.0.1:7737
$ cat <dir>/address
127.0.0.1:7737

$ WIRE_DIR=<empty-dir2> WIRE_HOST=engine.example.com WIRE_PORT=9999 yog wire-certs
  the engine binds and a local seat dials engine.example.com:9999
$ cat <dir2>/address
engine.example.com:9999
```

Both exit 0 and both report success. The first minted a CA and four leaves for
**loopback on the default port**, for an operator who asked for a named host —
and the material is then non-rotatable without `FORCE=1`, which distrusts
everything already issued. The verb's own `--help` gets the spelling right
(`WIRE_HOST=engine.example.com WIRE_PORT=7737` as a prefix); the refusal that
sends people there does not.

## Repro

    yog gesture --ws <ws> '/enroll laptop'     # on a :0 world → the argv-shaped remedy
    WIRE_DIR=<empty> yog wire-certs WIRE_HOST=h WIRE_PORT=9999 && cat <empty>/address
    # 127.0.0.1:7737 — both words ignored, exit 0

## The fix

- `material::REMEDY` is the verb, `yog wire-certs`. One const, three refusals,
  and `make wire-certs` stays what it is: a wrapper a developer types.
- The `/enroll` refusal and the mint's own "issue another client with" line
  spell the settings as the environment prefix they are.
- **`verb::stray` refuses an argv tail**, which is what makes the other two
  impossible to get wrong again. A word after the verb now names itself, names
  all six readings, and exits 2 having written nothing:

  ```
  $ WIRE_DIR=<dir> yog wire-certs WIRE_HOST=engine.example.com WIRE_PORT=9999
  yog wire-certs: "WIRE_HOST=engine.example.com" is not a setting this verb reads
    — WIRE_DIR, WIRE_HOST, WIRE_PORT, FORCE, WIRE_LEAF, WIRE_FOOT are environment
    readings, so they go BEFORE the verb: `WIRE_HOST=<host> WIRE_PORT=<port> yog wire-certs`
  ```

## A third instance, found while fixing the first two

The make target passed **four** of the six readings, so the very spelling its
own comment block teaches was inert for the other two: `make wire-certs FORCE=1`
did not rotate, and the mint's refusal — *"Re-run with FORCE=1 if that is what
you mean"* — answered the re-run with the same refusal. Same shape, other
direction: a setting the operator states and the act never sees. All six pass
through now.