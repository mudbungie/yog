+++
title = "the flow follows the row: fire the device flow where brazen's table declares a device endpoint"
created = 1787548693
updated = 1788493150
claimant = "Spellbind-X"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-c285"
on = "claim"

[[blockers]]
id = "bl-c5fe"
on = "claim"
+++
## What this lands (DESIGN §8.3 rule 1 as amended by bl-61bf; needs bl-c285 and upstream bl-c5fe)

- Consume the brazen release that answers bl-c5fe: bump the exact pin (Cargo.toml is the pin authority; rule 6 — no version restated here).
- The engine-side spawn (bl-c285) drops `--browser` exactly where the row's projected device-capability column says the row serves a headless flow — bz then runs its device flow, the URL and user code stream down the lane, and the sign-in completes from any seat, a phone's browser included. Everywhere else `--browser` stands, byte for byte.
- Loginability stays brazen's column, never a yog reclassification (DESIGN §8.3 rule 2's discipline): yog reads the projected fact, branches ONCE at the spawn, and no surface grows a flow selector.
- The window paints the device flow's verification URL as an opening link, so the co-located case keeps its one-gesture feel (DESIGN §8.3 rule 1 as amended).
- Sweep the interim prose bl-c285 left ('unconditional --browser at this pin') in code headers, DESIGN §8.3 rule 1's tail and rule 2's 'until it lands' clause, and src/config_edit/brazen/providers.rs's module note ('the projection carries no device-endpoint fact').
- Acceptance: a beat proving a device-declared row streams its URL + code down the lane with no loopback bound on the engine (fake substrate script), and a browser-only row still binds and opens locally.

---

The upstream half now lives in brazen's own store as bl-6680 (re-homed from bl-c5fe, closed here). The blocker edge on bl-c5fe is satisfied by that close; the real gate is a brazen release carrying the device column, and this ball bumps the pin to it.
