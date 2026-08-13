//! The Rust literal lexer behind the tofu guard: a real state machine over the
//! source text, not a regex.
//!
//! Comments must be excluded (they are full of `§ — …` and of glyph names
//! quoted in prose, none of which anything ever paints), and raw
//! strings/lifetimes must be told apart from ordinary strings/chars — neither
//! is expressible as a pattern match, which is why this is a scanner. It knows
//! nothing about fonts; [`super`] owns the probe that asks whether a character
//! the scan found has a glyph.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One non-ASCII character and the `path:line` sites it was found at.
pub type Sites = BTreeMap<char, Vec<String>>;

/// Push a literal character onto the finding map.
fn record(out: &mut Sites, c: char, path: &str, line: usize) {
    if !c.is_ascii() {
        out.entry(c).or_default().push(format!("{path}:{line}"));
    }
}

/// The char at `i`, or `None` past the end. The repo denies `indexing_slicing`
/// (AGENTS.md rule 4), and a lexer walks off its own end constantly — so every
/// lookahead here is an `Option`, and "past the end" is just `None`.
fn at(s: &[char], i: usize) -> Option<char> {
    s.get(i).copied()
}

/// Extract every non-ASCII char inside a string, raw-string or char literal of
/// one Rust source file. Line/block comments (nested) are skipped, as are
/// lifetimes (`'a`), which share their opening quote with char literals.
pub fn scan(src: &str, path: &str, out: &mut Sites) {
    let s: Vec<char> = src.chars().collect();
    let (mut i, mut line) = (0usize, 1usize);
    while let Some(c) = at(&s, i) {
        let next = at(&s, i + 1);
        if c == '\n' {
            line += 1;
            i += 1;
        } else if c == '/' && next == Some('/') {
            while at(&s, i).is_some_and(|c| c != '\n') {
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            i = skip_block_comment(&s, i, &mut line);
        } else if c == 'r' && matches!(next, Some('"' | '#')) {
            i = raw_string(&s, i, path, &mut line, out);
        } else if c == '"' {
            i = string(&s, i, path, &mut line, out);
        } else if c == '\'' {
            i = char_literal(&s, i, path, line, out);
        } else {
            i += 1;
        }
    }
}

/// Consume a `/* … */` comment from its opening slash, honouring nesting.
fn skip_block_comment(s: &[char], start: usize, line: &mut usize) -> usize {
    let (mut i, mut depth) = (start + 2, 1usize);
    while depth > 0 {
        let Some(c) = at(s, i) else { return i };
        if c == '/' && at(s, i + 1) == Some('*') {
            depth += 1;
            i += 2;
        } else if c == '*' && at(s, i + 1) == Some('/') {
            depth -= 1;
            i += 2;
        } else {
            if c == '\n' {
                *line += 1;
            }
            i += 1;
        }
    }
    i
}

/// Consume an ordinary `"…"` literal from its opening quote (escapes honoured).
fn string(s: &[char], start: usize, path: &str, line: &mut usize, out: &mut Sites) -> usize {
    let mut i = start + 1;
    while let Some(c) = at(s, i) {
        if c == '\\' {
            i += 2;
        } else if c == '"' {
            return i + 1;
        } else {
            if c == '\n' {
                *line += 1;
            }
            record(out, c, path, *line);
            i += 1;
        }
    }
    i
}

/// Consume an `r"…"` / `r#"…"#` literal from its `r`, or fall through (return
/// `start + 1`) when the `r` was just an identifier's first letter.
fn raw_string(s: &[char], start: usize, path: &str, line: &mut usize, out: &mut Sites) -> usize {
    let (mut i, mut hashes) = (start + 1, 0usize);
    while at(s, i) == Some('#') {
        hashes += 1;
        i += 1;
    }
    if at(s, i) != Some('"') {
        return start + 1;
    }
    i += 1;
    while let Some(c) = at(s, i) {
        if c == '"' && (1..=hashes).all(|k| at(s, i + k) == Some('#')) {
            return i + 1 + hashes;
        }
        if c == '\n' {
            *line += 1;
        }
        record(out, c, path, *line);
        i += 1;
    }
    i
}

/// Consume a `'x'` char literal from its opening quote. A lifetime (`'a`, and
/// the `'_` placeholder) has no closing quote in that position, so the quote is
/// simply skipped and the identifier lexes as ordinary code.
fn char_literal(s: &[char], start: usize, path: &str, line: usize, out: &mut Sites) -> usize {
    if at(s, start + 1) == Some('\\') {
        let mut i = start + 2;
        while at(s, i).is_some_and(|c| c != '\'') {
            i += 1;
        }
        return i + 1;
    }
    if at(s, start + 2) == Some('\'') {
        if let Some(c) = at(s, start + 1) {
            record(out, c, path, line);
        }
        return start + 3;
    }
    start + 1
}

/// Every `.rs` file under `dir`, recursively, in stable order. A directory that
/// cannot be read yields nothing — `the_sweep_is_not_vacuous` is what makes a
/// silently empty walk a failure rather than a green no-op.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = path.path();
        if path.is_dir() {
            out.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// The whole `src/` finding map: non-ASCII literal char → the sites using it.
pub fn sweep_src() -> Sites {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut out = Sites::new();
    for file in rust_files(&root.join("src")) {
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(&file)
            .display()
            .to_string();
        scan(
            &std::fs::read_to_string(&file).unwrap_or_default(),
            &rel,
            &mut out,
        );
    }
    out
}
