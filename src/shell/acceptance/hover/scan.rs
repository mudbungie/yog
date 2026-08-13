//! How [`super`] reads the tree: the source reduced to its call structure, and
//! the method chain hanging off one call.
//!
//! Split from the invariant at §12's cap on the seam between *what must hold*
//! and *how the source is read*. Nothing here knows what a hover is — it hands
//! back where every control is constructed and what is chained onto it, and the
//! parent decides whether that is enough. The reduction that produces the
//! skeleton is [`super::lex`], split off at the same budget.

use std::path::{Path, PathBuf};

/// Every `.rs` file under `root`, depth-first.
pub(in crate::shell::acceptance) fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
    found
}

/// Where each patterned call is constructed: the byte index of its opening
/// `(`, and the pattern that matched, for the failure message. The pattern
/// list is the caller's — [`super`]'s interactive controls, the naming scan's
/// text paints — so the two invariants read the tree through one finder.
pub(in crate::shell::acceptance) fn sites(
    skeleton: &str,
    patterns: &[&str],
) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for pattern in patterns {
        let mut from = 0;
        while let Some(hit) = skeleton.get(from..).and_then(|rest| rest.find(pattern)) {
            let at = from + hit + pattern.len() - 1;
            found.push((at, (*pattern).to_owned()));
            from = at + 1;
        }
    }
    found
}

/// The argument span of the call whose `(` is at `open` — what sits between
/// the parens, or `""` when they never close (a truncated skeleton indicts
/// nothing).
pub(in crate::shell::acceptance) fn args_of(skeleton: &str, open: usize) -> &str {
    balance(skeleton.as_bytes(), open)
        .and_then(|end| skeleton.get(open + 1..end - 1))
        .unwrap_or("")
}

/// Every member in the chain hanging off the call whose `(` is at `open`: its
/// name, and the byte index of its own opening `(` — `None` for a field hop
/// (`.response`), which is handed nothing. Builder steps and field hops are
/// members like any other, which is what lets the hover sit at the end of
/// `CollapsingHeader::new(…)…show(…).header_response` and still count for the
/// control at the head of the chain. The index is what lets a caller read what
/// a chained call was *handed* — the words a hover says (bl-478d) — where the
/// name alone only says that it was called.
pub(super) fn chain_of(skeleton: &str, open: usize) -> Vec<(String, Option<usize>)> {
    let bytes = skeleton.as_bytes();
    let mut names = Vec::new();
    let Some(mut i) = balance(bytes, open) else {
        return names;
    };
    loop {
        while bytes.get(i) == Some(&b' ') {
            i += 1;
        }
        if bytes.get(i) != Some(&b'.') {
            return names;
        }
        i += 1;
        let mut name = String::new();
        while let Some(c) = bytes
            .get(i)
            .filter(|c| c.is_ascii_alphanumeric() || **c == b'_')
        {
            name.push(char::from(*c));
            i += 1;
        }
        let called = (bytes.get(i) == Some(&b'(')).then_some(i);
        names.push((name, called));
        if let Some(at) = called {
            match balance(bytes, at) {
                Some(next) => i = next,
                None => return names,
            }
        }
    }
}

/// The index just past the `)` matching the `(` at `open`.
fn balance(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in bytes.iter().enumerate().skip(open) {
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}
