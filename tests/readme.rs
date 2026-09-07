//! **The README's two mechanical promises** (bl-f8e3).
//!
//! Everything else on that page is prose no test can judge. These two are
//! facts, and both had already failed: the page never named the published
//! install route at all — `cargo install` appeared nowhere, so the one command
//! that puts yog on a stranger's box was missing from the document a stranger
//! reads first — and the image example named a tag six versions stale, which
//! fails at the copy-paste for anyone who follows it.
//!
//! The second is the same rule the rest of the tree already keeps: `Cargo.toml`
//! is the version authority and no doc restates a version it owns. A literal
//! `yog:0.0.5` in a runnable example is exactly that restatement, and it rots
//! silently because nothing runs a README.

use std::path::Path;

/// The page, or the empty string — a README that cannot be read fails both
/// beats below with their own sentences, which is a better answer than a panic
/// in a helper that is not itself a test.
fn readme() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join("README.md")).unwrap_or_default()
}

/// The published route is on the page, spelled the way it must be typed.
/// `--locked` is not decoration: the lockfile is what pins the three embedded
/// substrate crates, and an install that resolved them fresh would be a
/// different program.
#[test]
fn the_readme_names_the_published_install() {
    let text = readme();
    assert!(
        text.contains("cargo install yog --locked"),
        "the README does not say how to install yog"
    );
}

/// **No example pins yog's own version** — `Cargo.toml` owns it and
/// `make print-image-tag` derives the tag from it. This is the check the stale
/// `yog:0.0.5` would have failed for the six releases it stood.
#[test]
fn no_example_restates_this_crates_version() {
    let text = readme();
    let stale: Vec<&str> = text
        .lines()
        .filter(|line| {
            line.split("yog:").skip(1).any(|tail| {
                let digits: String = tail.chars().take_while(|c| *c != ' ').collect();
                digits.split('.').count() == 3
                    && digits
                        .split('.')
                        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
            })
        })
        .collect();
    assert!(
        stale.is_empty(),
        "a README example pins a version Cargo.toml owns — say `yog:<version>` \
         and let `make print-image-tag` answer it:\n{}",
        stale.join("\n")
    );
}
