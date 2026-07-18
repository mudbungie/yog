+++
title = "fs_watcher: macOS FSEvents rename-source (.tmp) leaks as a spurious Removed change — coalesces_atomic_rename_to_destination red on macos-14"
created = 1784350143
updated = 1784350166
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Follow-up to bl-592b. That ball fixed 8 of 9 macOS failures (all probe canonicalization + 6 of 7 fs_watcher tests). CI run 29620934183 (commit faccc40) macos-14 job: 203 passed, 1 FAILED — fs_watcher::tests::coalesces_atomic_rename_to_destination.

PANIC (src/fs_watcher/tests.rs): "rename source leaked: [Change { path: "/private/var/.../steps/abc/001/request.json.tmp", kind: Removed }]".

ROOT CAUSE. bl-592b added is_rename_departure(kind,path) = matches!(Name(_)) && !exists, dropped in the coalesce loop when a Name event for a gone path is seen. On macOS, fs::write(tmp) sets FSEvents ITEM_MODIFIED and the rename sets ITEM_RENAMED, so for the .tmp path notify emits BOTH a Modify(Data) AND a Name(Any) (FSEvents coalesces per-path flags CREATED|MODIFIED|RENAMED into one event; translate_flags pushes Create, then Name(Any), then the Modify). When the Modify(Data) for tmp is processed AFTER the Name(Any) departure, it RE-ADDS tmp to the coalesce map; tmp no longer exists, so classify -> Removed and it leaks. The Name-only departure check is too narrow: it depends on the LAST event for the gone path being a Name event, which macOS violates.

FIX (validated locally on Linux: fmt+clippy clean, fs_watcher tests pass x2, module 100% covered mod.rs 60/60 tests.rs 27/27). Reframe: drop the special-case departure entirely; make classify return Option and drop ANY vanished path that carries no explicit Remove (rename source or transient), keep Remove->Removed and live->Touched. In coalesce, remove the is_rename_departure branch and map->filter_map:

    order.into_iter().filter_map(|p| {
        let change_kind = classify(latest[&p], &p)?;
        Some(Change { path: p, kind: change_kind })
    }).collect()

    fn classify(kind: EventKind, path: &Path) -> Option<ChangeKind> {
        match kind {
            EventKind::Remove(_) => Some(ChangeKind::Removed),
            _ if path.exists() => Some(ChangeKind::Touched),
            _ => None,
        }
    }

Delete is_rename_departure. Update the two classify tests to the Option return: classify(Remove,..) == Some(Removed); classify(Any, present) == Some(Touched); classify(Any, "/no/such/path") == None. This subsumes Linux (Name(From) on a gone tmp -> None) and macOS (any last flag on a gone tmp, non-Remove -> None). A genuine deletion still carries a Remove event (detects_removal_under_summary passes on both backends).

ACCEPTANCE: macos-14 make test green (all fs_watcher tests), linux still green, 100% coverage. Note the Linux ci job is separately red on a pre-existing lib.rs coverage regression (see the sibling task) — that must also land for CI to be fully green.