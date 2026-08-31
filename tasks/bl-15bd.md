+++
title = "android: one app named yog shipping all three components, each behind a bootstrap"
created = 1788067966
updated = 1788139182
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-223f"
on = "claim"
+++
Operator ruling 2026-08-30: the Android app is named yog and ships all three runnable components — the lernie seat, the thrall foot, and the yog server — each gated behind an explicit bootstrap rather than auto-started. The default bootstrap path is the mTLS client enrolment (seat or foot dialing a host engine, material provisioned out of channel per REMOTE §1.4); running the yog server locally on the phone is allowed but is the deliberate, non-default choice. The existing development client app is superseded and cleaned up — its landed foundations (JNI classloader work, the proven wire client, the tool host) are the starting material, not discarded work. Work lands in the android repo with its own tracking; this ball is yog's coordination record and closes when the app exists, the old app is retired, and REMOTE's client story names the phone as one more box.

---

Android side landed, overnight, in the android repo (five balls, all delivered to its main). What the ruling asked for, and what it actually cost:

**Wire v1 (its bl-93e3) — the highest-value piece, and the one that was load-bearing.** The client did not speak the §3 version preface at all, so it could not have opened a connection to a post-split engine: the engine reads a gesture envelope where a preface belongs and refuses before decoding. It now states its version and its request in one breath and confirms the engine's on the way to the answer, with the server's refusal sentence word for word.

The corpus is vendored there and replayed: every frame in both directories, the round trip on everything the client emits, and a recorded decision per shape — exhaustive in both directions, so a shape with no row and a row with no shape are each a red test. Nine request shapes and nine reply shapes are read; the rest refuse by name. **It caught a live defect on its first pass**: `prompt` answers `kind: "started"` and the client had no arm for it, so the one gesture that makes a conversation reported a failure over a conversation that was in fact running. Fixed there.

Two things the client had to grow for it: a decode side for requests (§3's "a client that only sends requests still decodes the request fixtures"), and the recognition that §3's third rule reaches INSIDE an envelope — that client spells one staging rung and predicts no conversation name, so a frame stating another rung or a real seed is refused rather than flattened into the shape it has.

**The rename and the bootstraps (its bl-7714).** applicationId and package moved from a name that spelled one component to the app's own. The component a launch runs is derived from the leaf, never stored: no material and nothing runs, and §4.2's grade on the certificate says seat or foot. So enrolling a phone as a tool host is minting it a foot-grade leaf rather than tapping a setting on the phone, and there is no stored choice that can disagree with the certificate. The first-run surface has no button, deliberately — §1.4 stands.

**Foot-shaped (its bl-2040).** §4.2's set is now a type there with three methods that owns the transport and never hands it out, so a fourth verb is a compile error rather than a refusal at this end.

**The server bootstrap (its bl-d6c6) — evaluated, not built, and the answer was not the expected one.** yog's library AND binaries cross-compile for aarch64-linux-android against a current NDK with balls, brazen, litany, ureq, rustls and ring in the graph and no C toolchain acquired; the release link is a ~13 MB PIE executable for that platform's loader. The Rust half is not the obstacle. Two rungs stop it: Android ships no git, and the world seeds shell shims its own agents run into app-private storage, which that platform refuses to execute by policy since API 29. The second is yog's own shape and is filed here as bl-6a6a — no ask attached, nothing blocked on it. The app offers the bootstrap, states both blockers in the operator's terms, and starts nothing.

Evidence: a debug APK builds, the whole suite is green, and the coverage floor holds at 100%.

Still open on the android side: §8.2 entries — that device as a client of many engines — filed there as its bl-d0d2. Nothing in REMOTE needs amending for any of the above; every ruling above was already written.
