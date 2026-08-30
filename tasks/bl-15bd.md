+++
title = "android: one app named yog shipping all three components, each behind a bootstrap"
created = 1788067966
updated = 1788067966
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-223f"
on = "claim"
+++
Operator ruling 2026-08-30: the Android app is named yog and ships all three runnable components — the lernie seat, the thrall foot, and the yog server — each gated behind an explicit bootstrap rather than auto-started. The default bootstrap path is the mTLS client enrolment (seat or foot dialing a host engine, material provisioned out of channel per REMOTE §1.4); running the yog server locally on the phone is allowed but is the deliberate, non-default choice. The existing development client app is superseded and cleaned up — its landed foundations (JNI classloader work, the proven wire client, the tool host) are the starting material, not discarded work. Work lands in the android repo with its own tracking; this ball is yog's coordination record and closes when the app exists, the old app is retired, and REMOTE's client story names the phone as one more box.