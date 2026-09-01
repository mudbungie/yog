+++
title = "the wire refusals name commands the operator cannot run: 'make wire-certs' on a box with no checkout, and env-only settings spelled as argv that yog wire-certs accepts, ignores and calls success"
created = 1788235220
updated = 1788235220
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

## Shape of the fix

- `REMEDY` becomes the verb, `yog wire-certs`. One const, three refusals, and
  `make wire-certs` stays what it is: a wrapper a developer types.
- The `/enroll` refusal spells the settings as the environment prefix they are.
- **And the verb should refuse an argv tail it cannot read.** That is the half
  that makes the other two impossible to get wrong again: a stray word today is
  accepted, ignored, and reported as success — the failure mode where the
  operator is told the act worked and it did the default instead.