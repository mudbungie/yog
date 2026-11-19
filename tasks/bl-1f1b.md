+++
title = "the advisories mend is held red by an unmaintained notice with no upgrade (chrono, RUSTSEC-2026-0543, via litany), so two fixable webpki advisories never land: ignore it by ruling; and a landing whose fast-forward is refused because main moved exits green, since the next run heals it"
created = 1795071706
updated = 1795071707
claimant = "Knuckled-Y"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Two operator rulings. (1) RUSTSEC-2026-0543 is an unmaintained notice, not a vulnerability: no fixed version exists and the crate arrives through litany. It joins deny.toml's ignore list with its reason, to be dropped when cargo-deny warns advisory-not-detected. (2) land-through-gate.sh: a refused fast-forward (main moved while the gate ran) prints its line and exits 0 with nothing landed and the branch deleted, so the alarm does not open; red CI, a missing token and everything else stay red.