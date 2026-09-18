//! **Every file `build.rs` reads reaches the image's build context** (bl-d58a).
//!
//! `Containerfile` COPYs by name and `.containerignore` keeps the rest out, so
//! the build context is an include list (DESIGN §10.1) — and an include list
//! has the failure mode `Cargo.toml`'s own `include` was written to avoid: a
//! missing entry is a missing input the compiler can only report as *not
//! found*. bl-1be7 made `corpus/shapes.json` the third input of the build
//! script and added it to the crate's package; the image's list was not
//! touched, and four releases published no image while the reconciler on the
//! engine box truthfully reported the last one that had.
//!
//! The roster is derived, not restated: the inputs are exactly the paths
//! `build.rs` names in its `rerun-if-changed` lines, a `{CONST}` in one
//! resolved through the file's own `const NAME: &str = "…"`. So a fourth input
//! declared there fails here until it is COPYd, and the test has no list of
//! its own to go stale. Two directions: the roster must be non-empty, or a
//! rewrite of `build.rs` that stopped spelling the lines this way would pass
//! by finding nothing.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// A file that cannot be read reads as EMPTY, and the vacuity guard below is
/// what turns that into a failure with the file's name in it.
fn read(name: &str) -> String {
    fs::read_to_string(root().join(name)).unwrap_or_default()
}

/// The paths `build.rs` declares it reads, in the order declared.
fn inputs() -> Vec<String> {
    let source = read("build.rs");
    let consts: HashMap<String, String> = source
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("const ")?;
            let (name, value) = rest.split_once(": &str = \"")?;
            Some((name.to_owned(), value.split('"').next()?.to_owned()))
        })
        .collect();
    source
        .lines()
        .filter_map(|line| line.split("rerun-if-changed=").nth(1))
        .map(|tail| {
            let spelled = tail.split('"').next().unwrap_or(tail);
            match spelled.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                // A const the file does not define stays spelled `{NAME}`,
                // which no COPY line names, so the assertion below says so.
                Some(name) => consts
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| spelled.to_owned()),
                None => spelled.to_owned(),
            }
        })
        .collect()
}

/// Every source a `COPY` line in the Containerfile names, one token each.
fn copied() -> Vec<String> {
    read("Containerfile")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("COPY "))
        .filter(|rest| !rest.starts_with("--from="))
        .flat_map(|rest| {
            let mut tokens: Vec<String> = rest.split_whitespace().map(str::to_owned).collect();
            tokens.pop(); // the destination
            tokens
        })
        .collect()
}

/// The prefixes `.containerignore` keeps out of the context.
fn ignored() -> Vec<String> {
    read(".containerignore")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_build_script_declares_its_inputs_in_a_shape_this_test_can_read() {
    let inputs = inputs();
    assert!(
        inputs.len() >= 2,
        "build.rs declares {inputs:?}; it reads PROTOCOL and the corpus record, so a roster this short means the lines are no longer spelled `rerun-if-changed=`"
    );
}

#[test]
fn every_build_input_is_copied_into_the_image_and_not_ignored_out_of_its_context() {
    let copied = copied();
    let ignored = ignored();
    for input in inputs() {
        assert!(
            copied.contains(&input),
            "build.rs reads {input:?} and no COPY line in the Containerfile names it; the image build fails at the build script with a bare NotFound"
        );
        let shadow = ignored.iter().find(|rule| {
            let rule = rule.trim_end_matches('/');
            input == *rule || input.starts_with(&format!("{rule}/"))
        });
        assert!(
            shadow.is_none(),
            ".containerignore rule {shadow:?} keeps {input:?} out of the build context, so its COPY line cannot find it"
        );
    }
}

#[test]
fn the_record_is_one_of_the_inputs() {
    // The case this file was written for, by name, so a rewrite of `build.rs`
    // that dropped the record also drops this line's reason to exist.
    assert!(inputs().contains(&"corpus/shapes.json".to_owned()));
}
