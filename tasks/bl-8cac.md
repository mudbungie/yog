+++
title = "encode the icon PNGs properly; the encoder belongs in dev-dependencies"
created = 1785730263
updated = 1785730263
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["icon"]
+++
**the operator's ruling:** "mmmm, I like the purity impulse, but there are reasons that
I'm going to want to drop the icon in places. It's okay to encode the built
images. Couple pngs, and an svg. Not big."

The stored-deflate encoder in `icon/png.rs` preserved a property nobody asked
for. The property worth keeping is **no codec in the shipped binary** — and that
is better served by making PNG encoding a *build-time* concern: `image` moves to
`[dev-dependencies]`, where the example that generates the artifacts and the
test that checks them can both reach it, and the binary never links it. Same
guarantee, honest reason, and it deletes 104 lines rather than adding any.

`image` is already in the graph at exactly the features eframe asks for
(`default-features = false, features = ["png"]`), so this must add ZERO crates
— verify `Cargo.lock` gains only the dev edge before landing.

Real compression also buys the sizes worth having: the set becomes 16 / 32 / 48
/ 64 / 128 / 256, so there is something to drop into a README or a web page,
and the whole checked-in set should still come in well under the 60 KB the five
uncompressed ones cost today.