+++
title = "the flow follows the row: fire the device flow where brazen's table declares a device endpoint"
created = 1787548693
updated = 1788493581
claimant = "Spellbind-Y"
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

---

Upstream landed; this ball is blocked one level down.

brazen 0.0.8 is published (upstream brazen bl-6680, closed there): the builtin `openai-chatgpt` row now declares `device = { url = "https://auth.openai.com", style = "codex" }` — OpenAI's own pre-standard device-code wire, a flow VARIANT selected by row data rather than a URL, since that endpoint does not speak the RFC 8628 grammar and would refuse it at the first POST — and `bz --list-providers --json` carries a `device` column naming the headless flow a row serves (`codex`, `rfc8628`, or absent). It carries the STYLE and not a bool on purpose: this ball's consumer reads the value.

**It cannot land until litany releases.** The exact pins make the three crates one chain: litany 0.0.6 pins `brazen = "=0.0.7"`, and yog pins both `litany = "=0.0.6"` and `brazen = "=0.0.7"` deliberately — the lockfile is §16.7's own parity check that exactly ONE brazen resolves. Bumping yog's brazen pin alone does not resolve:

    error: failed to select a version for the requirement `brazen = "=0.0.7"`
    candidate versions found which didn't match: 0.0.8
    required by package `litany v0.0.6`

Filed as litany bl-8daa (bump litany's brazen pin to =0.0.8 and release). When that ships, the work here is: bump BOTH pins together; add the `device` column to `ProviderRow` and `provider_rows` (a plain string column, absent folding to `""` like every other, with a `headless_login()` predicate beside `login_blocked()`); give `login::start` and `by_hand` a flow selector, read ONCE in `runs::Runs::start` off `RealBzRunner::resolve(&wall).providers()` — the wall lens that spawn already builds — so the branch is one read at the spawn and no surface grows a flow selector; and sweep the interim prose this ball lists. Nothing was left in the tree: the claim was worked as far as the pin bump, found unresolvable, and reverted.
