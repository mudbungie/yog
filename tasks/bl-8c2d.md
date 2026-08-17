+++
title = "RULING: a default install has no browser-login row — 'sign the sphere in' resolves to the config editor, not bz --login; is that the intent?"
created = 1786685596
updated = 1786937419
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Found by Marlin closing bl-3b62 (2026-08-13): brazen's shipped provider table carries NO oauth2 row, so on a default install every row's honest empty-state answer is 'set the key in Config' rather than a Login button. The §9.1 editor is now live on the same wall in the empty state (bl-3b62), so a key can be authored before the first turn — discoverable rather than learned by dying — but nobody gets a browser sign-in out of the box: the bl-9b52 ruling's phrase 'sign the sphere in' is satisfied via the config editor, not via bz --login, for a stranger with no custom row.

This is §8.3-as-amended working as written. The question for the operator: is key-authoring the intended stranger path, or should brazen ship (or yog surface) a browser-login-capable row by default? If the former, close this with the ruling recorded; if the latter, the work is brazen-side and this ball becomes the yog-side consume.

---

Operator ruling 2026-08-16: browser login SHOULD ship — key-authoring as the only stranger path is not the intent. Per this ball's own framing the work is brazen-side (ship a browser-login-capable row in the default provider table); a brazen ball is being filed for it, and this ball becomes the yog-side consume: once brazen releases the default row, verify 'sign the sphere in' resolves to the Login affordance on a default install and bump the pin.

---

Brazen side LANDED (brazen ball bl-77fa, delivery 5024850 on brazen main): the default provider table now ships an eighth row 'openai-chatgpt' (oauth2, no model_prefixes, last in the table), so 'bz --login --provider openai-chatgpt --browser' completes on a bare install with no config file — end-to-end test pins it. The Anthropic OAuth path stays unshipped per brazen bl-a661's terms ruling; the guard invariant narrowed to 'no Anthropic OAuth row', which is what that ruling actually bought. CONSUME here is a two-repo lockstep once release-plz ships brazen 0.0.6: lernie bumps its brazen pin to =0.0.6 and publishes FIRST, then yog bumps (Cargo.toml pins brazen =0.0.5 as the §16.7 one-brazen parity check with lernie — bumping yog alone breaks it). Then verify the empty-state 'sign the sphere in' path surfaces the Login affordance for the new row on a default install.
