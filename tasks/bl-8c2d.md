+++
title = "RULING: a default install has no browser-login row — 'sign the sphere in' resolves to the config editor, not bz --login; is that the intent?"
created = 1786685596
updated = 1786685596
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Found by Marlin closing bl-3b62 (2026-08-13): brazen's shipped provider table carries NO oauth2 row, so on a default install every row's honest empty-state answer is 'set the key in Config' rather than a Login button. The §9.1 editor is now live on the same wall in the empty state (bl-3b62), so a key can be authored before the first turn — discoverable rather than learned by dying — but nobody gets a browser sign-in out of the box: the bl-9b52 ruling's phrase 'sign the sphere in' is satisfied via the config editor, not via bz --login, for a stranger with no custom row.

This is §8.3-as-amended working as written. The question for the operator: is key-authoring the intended stranger path, or should brazen ship (or yog surface) a browser-login-capable row by default? If the former, close this with the ruling recorded; if the latter, the work is brazen-side and this ball becomes the yog-side consume.