+++
title = "hover::CONTROLS is a hand-listed scan with ID_IDENTS' failure mode: a new widget constructor is silently unpoliced"
created = 1786685253
updated = 1786686401
claimant = "Tenon"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Filed from bl-45c7's close-out (Lintel, 2026-08-13). src/shell/acceptance/hover/mod.rs:40 enumerates 17 egui constructor names by hand; a new click-sensing widget lands unpoliced with no red anywhere — the same shape bl-45c7 just retired for ID_IDENTS (which recurred verbatim before it was fixed). Lower risk (it enumerates a FOREIGN API yog's authors do not rename, and it carries a 'seen >' vacuity guard) but the value-derived alternative exists: drive the real window and require every click-sensing Response to carry a tooltip, deriving the set from behavior instead of names. Prove the rebuild bites per the bl-45c7 standard: plant an unlisted tooltip-less control and show red. NOT the same mode (verified by Lintel, do not 'fix'): bands::ORDER/CENSUS (the subject of its own claim) and legible::KNOWN (a deliberately-empty excuse list).