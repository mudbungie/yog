+++
title = "B3: locks to state.rs + probe-cache RefCell->Mutex + lock/rc ast-grep rules"
created = 1784433624
updated = 1784433989
claimant = "filtered"
parent = "bl-97fb"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-cbf9"
on = "claim"
+++
(1) New src/state.rs: move DirtySet (watch/mod.rs) and a WatchSetHandle type alias/newtype for Arc<Mutex<WatchSet>> there; the pure WatchSet registry stays in watch/. (2) probe_cache: RefCell -> Mutex and relocate the TtlCache lock-holding into state.rs OR keep TtlCache in git_tree with its Mutex and list probe_cache.rs in the rule ignores — prefer the state.rs move per the named-chokepoint intent; use .lock().unwrap_or_else(PoisonError::into_inner) (poison-immune, no panic path — B5 will demand it anyway). (3) Test doubles: the bootstrap bans Rc/RefCell REPO-WIDE (no test exemption): convert FakeClock Rc<Cell<Instant>> -> Arc<Mutex<Instant>> (test_support + ui_state/tests), FakeFs RefCell<HashMap> -> Mutex, brazen tests RefCell<Vec<String>> log -> Mutex, and any other Rc/RefCell test sites (assessment lists them; Cell alone is tolerated — keep bare Cell counters). (4) rules/locks-outside-state.yml exactly per the bootstrap doc but with ignores extended: ["**/state.rs", "**/test_support.rs", "**/tests/**", "**/tests.rs", "**/*tests.rs"] (SPAWN_LOCK/ENV_LOCK and test-double locks are not app state — surfaced adaptation); rules/no-rc-refcell.yml verbatim (no ignores). Extend rules/fixtures/violations.rs; smoke-test both rules. Gate green.