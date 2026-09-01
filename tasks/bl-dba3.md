+++
title = "the provider pane re-derives brazen's credential column from a file probe, so an ambient credential renders as 'not signed in' with a Login button"
created = 1788235095
updated = 1788235095
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["providers"]
+++
Driving the flagship build-an-app workflow on a scratch world, `/providers`
answered, for an oauth2 row whose credential brazen resolves from outside its
own store:

    auth oauth2 · not signed in     (blocked: null → the Login verb is offered)

The same row, asked of brazen through the same wall in the same second:

    credential = ambient

and a live model call through that row succeeded. So the boundary tells the
operator to sign in to a row that is already answering, and the §8.3 Login pane
renders a button for a login that has nothing to do.

## Why it is wrong, structurally

`ProviderRow` already carries brazen's `credential` column, and its own doc
comment states the rule this violates verbatim
(`src/config_edit/brazen/providers.rs`):

> **The credential this row would actually use**, in brazen's own spelling:
> `stored` … `ambient` … `inline` … `not required` … or `missing`. brazen
> computes it through the very `fetch_cred` a run spends, minus the network, so
> it is the authority on "could this row answer at all" and yog re-derives none
> of it.

`view()` then re-derives it anyway. `credential_words(present)` reads `present`
— the §5.1 #22 existence probe, `credential_presence()` in
`src/config_edit/brazen/paths.rs`, which is `<credentials-dir>/<name>.json`
exists. That probe cannot see `ambient` or `inline` at all, so both render as
"not signed in" / "no credential stored".

Two representations of one fact, and they have drifted. The §8.1 start gate
reads the authoritative column (`WallCredit::read` — `credential != missing &&
credential != not required`), so the gate lets the start through while the pane
beside it says the row is not signed in.

## Shape of the repro

1. Give a wall a brazen config declaring an oauth2 row whose credential brazen
   resolves ambiently rather than from its own credentials directory.
2. `yog exec --ws <ws> bz --list-providers --json` → that row reads
   `"credential": "ambient"`.
3. `yog gesture --ws <ws> /providers` → that row reads `not signed in`.
4. A model call through the row succeeds, proving (3) false.

## The fix, which is subtraction

`credential_words` reads `self.credential`, not a probe. `credential_presence`
and the `creds` argument threaded into `row_views` then have no caller and go.
One predicate — "does this row carry a credential a run would spend" — with one
home, which `WallCredit` can share instead of spelling a second time.