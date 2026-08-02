+++
title = "world template/providers.yaml births every new workspace, but no yog surface can see, edit, or validate it"
created = 1785645290
updated = 1785645290
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found closing the bl-662f incident (2026-08-01): $XDG_DATA_HOME/yog/world/lernie/template/providers.yaml is the birth config for every new workspace, yet §9.2's config editors reach only models.yaml + workflows/* — a stale/broken template (e.g. the codex row) is invisible and unfixable in-app; every new workspace is born broken and the operator can't see why. Close the class, minimally: prefer VALIDATION over surface — at workspace creation (or at template read), check the template's provider rows against brazen's live table exactly as bl-bd89's picker now does (PickError::UnknownProvider machinery exists in src/model_pick), and refuse/flag with a pointer, or auto-offer the picker. Exposing the template in the §9.2 editors is the fallback if validation alone can't repair. Check DESIGN §9.2/§16 first; amend the doc. Note: lernie bl-9391 fixes the seed data upstream, so yog's job is only 'never trust the template silently'.