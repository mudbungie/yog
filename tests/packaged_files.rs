//! The packaged-file guard (bl-8340): **what `cargo publish` would ship is
//! read off the real `cargo package --list`, and every path in it must be a
//! class that was ruled in.**
//!
//! Why a test and not a checklist item: AGENTS.md's publication checklist is
//! explicit that this one is not recoverable — *"`cargo publish` is
//! irreversible: a yanked version stays downloadable"* — and it records the
//! sighting. yog 0.0.1 shipped the operator's home paths across four files and
//! three `docs/drive-logs/` transcripts *because `Cargo.toml` declared no
//! `include`/`exclude` and so packaged the whole tree*. The manifest now
//! declares an `include` allowlist, and an allowlist without a test is a
//! comment: nothing else notices when a later edit widens it, and the notice
//! arrives after the version is public.
//!
//! The classes below are a **second statement** of the manifest's policy, which
//! is deliberate and is the only shape that can work. A check that derived its
//! allowlist from the `include` key would widen with it and stay green through
//! the exact edit it exists to catch — the same reason the module-map guard
//! restates DESIGN §12's rules instead of reading them out of the table.
//!
//! Both directions, because a shape guard dies by matching nothing:
//! [`the_list_is_not_vacuous`] fails a spawn that answered with a short list,
//! and [`the_allowlist_sees_its_own_violations`] fails an `is_ruled_in` that
//! has quietly become true of everything — including of
//! `scripts/leak-fixtures/README.md`, which is not a hypothetical: a bare
//! `README.md` include pattern is gitignore-style and unanchored, and it
//! shipped that file (the index of a corpus of fabricated secrets) out of a
//! list naming no `scripts` entry, until the pattern was anchored to `/`.
//!
//! The fail-closed direction has one cost — a build input added tomorrow
//! silently does NOT ship — and [`every_compile_time_embed_ships`] is the
//! answer to it: the `include_bytes!`/`include_str!` targets outside `src` are
//! swept out of the tree and each one must be in the list.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The package root — this crate's own manifest directory, so the guard reads
/// the tree it was compiled from rather than a working directory.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The real answer to *"what would `cargo publish` upload?"*, one path per
/// line. `--offline` keeps the guard hermetic (the lockfile is committed and
/// every dependency is already resolved by the time a test binary runs);
/// `--allow-dirty` is required because `cargo package` refuses a worktree with
/// uncommitted changes outright, and a claim worktree mid-edit is the normal
/// case for the author this test is addressed to. Spawned through
/// [`yog::git_env::command`], the crate's one `Command` constructor — a bare
/// `Command::new` is a defect (`rules/no-bare-command.yml`), and here it would
/// hand cargo the ambient `GIT_DIR` of whatever hook invoked the suite, which
/// is precisely the repository this must not read.
fn packaged() -> Vec<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let out = yog::git_env::command(Path::new(&cargo))
        .current_dir(root())
        .args(["package", "--list", "--offline", "--allow-dirty"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cargo package --list did not answer: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The classes ruled into the published crate: the crate's own source, the
/// icon artifacts `src/theme/icon/tests/artifacts.rs` embeds, and the files
/// crates.io renders. `Cargo.toml.orig` and `.cargo_vcs_info.json` are minted
/// by cargo into the tarball and are not tree files at all.
fn is_ruled_in(path: &str) -> bool {
    let named = matches!(
        path,
        "Cargo.toml"
            | "Cargo.lock"
            | "Cargo.toml.orig"
            | ".cargo_vcs_info.json"
            | "README.md"
            | "LICENSE"
            | "CHANGELOG.md"
            | "assets/yog.svg"
    );
    named
        || path
            .strip_prefix("src/")
            .is_some_and(|p| p.ends_with(".rs"))
        || path
            .strip_prefix("assets/yog-")
            .is_some_and(|p| p.ends_with(".png"))
}

/// Every `.rs` file under `dir`, recursively — forgiving, like the citation and
/// module-map guards' sweeps; [`the_list_is_not_vacuous`] is what stops
/// "nothing" from passing.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// One source file's compile-time embeds, resolved against its own directory
/// and expressed relative to the package root. Only targets OUTSIDE `src` are
/// interesting: `src/**` ships whole, so an embed of a sibling module's data
/// cannot be dropped by the allowlist.
fn embeds_of(file: &Path, root: &Path, out: &mut BTreeSet<String>) {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    let dir = file.parent().unwrap_or(root);
    for macro_name in ["include_bytes!(\"", "include_str!(\""] {
        for tail in text.split(macro_name).skip(1) {
            let Some((target, _)) = tail.split_once('"') else {
                continue;
            };
            let Ok(abs) = dir.join(target).canonicalize() else {
                continue;
            };
            let Ok(rel) = abs.strip_prefix(root) else {
                continue;
            };
            let rel = rel.display().to_string();
            if !rel.starts_with("src/") {
                out.insert(rel);
            }
        }
    }
}

/// Every non-`src` path the crate embeds at compile time — the build inputs an
/// allowlist can silently drop, gathered from the tree rather than listed here.
fn embedded_paths() -> BTreeSet<String> {
    let root = root();
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let mut out = BTreeSet::new();
    for file in &files {
        embeds_of(file, &root, &mut out);
    }
    out
}

/// The defect: design commentary, gate apparatus and agent guides shipping to
/// crates.io with the binary. Stated as an allowlist so the NEXT file class
/// added to the tree is red here instead of public there.
#[test]
fn no_commentary_or_apparatus_ships() {
    let strays: Vec<String> = packaged().into_iter().filter(|p| !is_ruled_in(p)).collect();
    assert!(
        strays.is_empty(),
        "paths `cargo publish` would upload that no class rules in. yog 0.0.1 \
         published private context exactly this way and a yanked version stays \
         downloadable — widen `include` in Cargo.toml only with a reason, and \
         add the class here:\n{}",
        strays.join("\n")
    );
}

/// The other side of a fail-closed list: the crate must still be a crate.
#[test]
fn the_files_crates_io_needs_ship() {
    let list = packaged();
    for needed in [
        "Cargo.toml",
        "Cargo.lock",
        "README.md",
        "LICENSE",
        "src/lib.rs",
        "src/main.rs",
    ] {
        assert!(
            list.iter().any(|p| p == needed),
            "{needed} is not in the packaged list — `include` dropped a file \
             crates.io or the build needs"
        );
    }
}

/// An `include` allowlist's one cost, paid: an asset added under `assets/` and
/// embedded from `src` would compile here and fail to compile for anyone who
/// downloaded the crate. The sweep is over the tree, so it covers embeds that
/// do not exist yet.
#[test]
fn every_compile_time_embed_ships() {
    let list = packaged();
    let missing: Vec<String> = embedded_paths()
        .into_iter()
        .filter(|p| !list.contains(p))
        .collect();
    assert!(
        missing.is_empty(),
        "paths `src` embeds with include_bytes!/include_str! that the package \
         does not carry — the published crate cannot compile:\n{}",
        missing.join("\n")
    );
}

/// A guard that measured nothing must not read as a pass: a failed spawn, an
/// empty stdout or a sweep that found no embeds all land here.
#[test]
fn the_list_is_not_vacuous() {
    let list = packaged();
    let sources = list.iter().filter(|p| p.starts_with("src/")).count();
    assert!(
        sources > 300,
        "the packaged list carries {sources} src paths over {} entries — the \
         spawn is broken, not the tree",
        list.len()
    );
    let embeds = embedded_paths();
    assert!(
        embeds.len() >= 7,
        "the embed sweep found {} non-src compile-time embeds; the icon \
         artifacts alone are seven",
        embeds.len()
    );
}

/// The negative direction for the restated policy: each excluded class, and
/// the measured unanchored-pattern trap, must be seen as a violation — and the
/// classes that ship must not.
#[test]
fn the_allowlist_sees_its_own_violations() {
    for stray in [
        "docs/DESIGN.md",
        "docs/VISION.md",
        "docs/QUALITY.md",
        "docs/REMOTE.md",
        "docs/STORIES.md",
        "AGENTS.md",
        "CLAUDE.md",
        "Makefile",
        "deny.toml",
        "tests/packaged_files.rs",
        "examples/icon.rs",
        "rules/no-bare-command.yml",
        ".github/workflows/ci.yml",
        ".githooks/pre-commit",
        "scripts/leak-scan.sh",
        // the unanchored-pattern sighting: a bare `README.md` include pattern
        // shipped this, and no `scripts` class rules it in
        "scripts/leak-fixtures/README.md",
        "assets/yog.desktop",
    ] {
        assert!(!is_ruled_in(stray), "{stray} must not be ruled in");
    }
    for shipped in [
        "src/main.rs",
        "src/app/mod.rs",
        "assets/yog-16.png",
        "assets/yog.svg",
        "LICENSE",
    ] {
        assert!(is_ruled_in(shipped), "{shipped} must be ruled in");
    }
}
