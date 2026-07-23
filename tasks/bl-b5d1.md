+++
title = "batteries-included: consume lernie/bl/brazen as library deps, drop all system-binary coupling"
created = 1784784064
updated = 1784784298
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-e2fc"
on = "close"

[[blockers]]
id = "bl-5678"
on = "close"
+++
Operator ruling (2026-07-22): the whole point of bringing the directly consumed binaries into yog. Incorporate the LATEST versions of lernie, bl (balls), and (transitively) brazen as literal Rust library dependencies. Break every dependence on system-installed binaries: installing yog must ship the batteries. The machine-local brazen/lernie checkouts and installed bz/lernie/bl binaries may differ freely from yog's linked versions — yog pins its own. Context: the 2026-07-22 field breakage (bz 0.0.3 vs lernie's =0.0.2 pin) silently killed every prompt; this class of skew dies with the binary seam. Deliverables land via subtasks: spec/arch design doc first, then the implementation waves.