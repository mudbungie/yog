//! The **reduction**: one `.rs` source to the skeleton [`super::scan`] walks.
//!
//! Split from the walk at §12's cap on the seam the walk's own doc names —
//! *how the source is read* versus *what is found in it*. Everything here is
//! lexical: comments dropped, whitespace runs collapsed, and every literal kept
//! as its own words between quotes, with the three characters that would let a
//! string fake structure escaped on the way in.

use std::borrow::Cow;

/// A literal's own parens, escaped, so a `)` in a hover cannot close a call the
/// walk is counting. [`text_of`] puts them back — which is what lets the
/// spelling scan read `(f)` out of a sentence (bl-478d).
const OPEN: char = '⟨';
/// The closing half of [`OPEN`].
const CLOSE: char = '⟩';
/// A literal's own `;`, escaped for the same reason: the skeleton reads a
/// semicolon as where a `const`'s words end, and a sentence is full of them.
const SEMI: char = '¦';

/// A skeleton span as an operator would read it: the escaped punctuation
/// restored.
pub(in crate::shell::acceptance) fn text_of(span: &str) -> String {
    span.replace(OPEN, "(")
        .replace(CLOSE, ")")
        .replace(SEMI, ";")
}

/// The call structure with comments dropped, whitespace runs collapsed, and
/// every literal reduced to its own words between quotes. What is left has
/// parens that balance for real — a `)` inside a string or a `//` inside a URL
/// cannot fake one, because a literal's parens are escaped to [`OPEN`]/[`CLOSE`]
/// on the way in — and it still says whether a hover was handed nothing (`""`).
/// The words survive because the §11 discoverability scan reads them: a hover
/// must name its keyboard spelling, and a skeleton that flattened every string
/// to `"x"` could only say that *something* was said.
pub(in crate::shell::acceptance) fn skeleton(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let (next, emit) = token(bytes, i);
        // Whitespace runs collapse to one space, so the only gap the walk below
        // ever steps over is a single character wide.
        if emit != " " || !out.ends_with(' ') {
            out.push_str(&emit);
        }
        i = next;
    }
    // A wrapped method chain (`ui\n    .button(…)`) is the same expression as an
    // unwrapped one, so the line break rustfmt chose must not decide whether a
    // control is found.
    out.replace(" .", ".")
}

/// One lexical step from `at`: where the next token starts, and what the
/// skeleton keeps of this one.
fn token(bytes: &[u8], at: usize) -> (usize, Cow<'static, str>) {
    let two = (bytes.get(at), bytes.get(at + 1));
    match two {
        (Some(b'/'), Some(b'/')) => (line_end(bytes, at), Cow::Borrowed(" ")),
        (Some(b'/'), Some(b'*')) => (block_end(bytes, at + 2), Cow::Borrowed(" ")),
        _ => literal_or_char(bytes, at),
    }
}

/// A string, raw string, char literal, or a single plain byte.
fn literal_or_char(bytes: &[u8], at: usize) -> (usize, Cow<'static, str>) {
    if let Some((end, body)) = string_at(bytes, at) {
        return (end, Cow::Owned(body));
    }
    let emit = match bytes.get(at) {
        Some(b'\'') => return char_at(bytes, at),
        Some(c) if c.is_ascii_whitespace() => " ",
        Some(b'(') => "(",
        Some(b')') => ")",
        Some(b'.') => ".",
        Some(b'"') => "\"",
        Some(c) => kept(*c),
        None => "",
    };
    (at + 1, Cow::Borrowed(emit))
}

/// Identifier bytes survive verbatim; everything else the walk does not read is
/// flattened to a space, which keeps the skeleton ASCII and short. The `;` is
/// read: it is where a `const`'s or a `let`'s words end, which is how a
/// delegated hover is followed to the name that holds it (bl-478d).
fn kept(c: u8) -> &'static str {
    const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_:;";
    ALPHABET
        .as_bytes()
        .iter()
        .position(|a| *a == c)
        .and_then(|i| ALPHABET.get(i..=i))
        .unwrap_or(" ")
}

/// A string literal starting at `at`, raw (`r#"…"#`, `br"…"`) or plain, as the
/// quoted token the skeleton keeps. `None` when `at` is not a string.
fn string_at(bytes: &[u8], at: usize) -> Option<(usize, String)> {
    let mut i = at;
    if bytes.get(i) == Some(&b'b') {
        i += 1;
    }
    if bytes.get(i) == Some(&b'r') {
        i += 1;
        let mut hashes = 0usize;
        while bytes.get(i) == Some(&b'#') {
            hashes += 1;
            i += 1;
        }
        if bytes.get(i) != Some(&b'"') {
            return None;
        }
        let (end, body) = raw_end(bytes, i + 1, hashes);
        return Some((end, quoted(bytes, i + 1, body)));
    }
    if bytes.get(i) != Some(&b'"') {
        return None;
    }
    let (end, body) = plain_end(bytes, i + 1);
    Some((end, quoted(bytes, i + 1, body)))
}

/// A literal's body, quoted and made safe to leave in the skeleton: its parens
/// and semicolons escaped so they cannot close a call or end an item, its own
/// quotes and backslashes spent
/// (a `\` + newline continuation is whitespace in the string it builds), and
/// every whitespace run collapsed to one space like the rest of the skeleton.
fn quoted(bytes: &[u8], from: usize, to: usize) -> String {
    let mut out = String::from("\"");
    for c in String::from_utf8_lossy(bytes.get(from..to).unwrap_or_default()).chars() {
        match c {
            '(' => out.push(OPEN),
            ')' => out.push(CLOSE),
            ';' => out.push(SEMI),
            '"' | '\\' | ' ' | '\n' | '\r' | '\t' => {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Scan to the unescaped `"` closing a plain string opened at `from`: the index
/// past it, and where its body ends.
fn plain_end(bytes: &[u8], from: usize) -> (usize, usize) {
    let mut i = from;
    while let Some(c) = bytes.get(i) {
        match c {
            b'\\' => i += 2,
            b'"' => return (i + 1, i),
            _ => i += 1,
        }
    }
    (i, i)
}

/// Scan to the `"` + `hashes` `#` closing a raw string opened at `from`.
fn raw_end(bytes: &[u8], from: usize, hashes: usize) -> (usize, usize) {
    let mut i = from;
    while let Some(c) = bytes.get(i) {
        i += 1;
        if *c != b'"' {
            continue;
        }
        let closed = (0..hashes).all(|h| bytes.get(i + h) == Some(&b'#'));
        if closed {
            return (i + hashes, i - 1);
        }
    }
    (i, i)
}

/// A char literal (`'x'`, `'\n'`) — or a lifetime, which is left as a byte.
fn char_at(bytes: &[u8], at: usize) -> (usize, Cow<'static, str>) {
    let mut i = at + 1;
    if bytes.get(i) == Some(&b'\\') {
        i += 1;
    }
    // The character itself: its lead byte and any UTF-8 continuations.
    i += 1;
    while bytes
        .get(i)
        .is_some_and(|c| *c & 0b1100_0000 == 0b1000_0000)
    {
        i += 1;
    }
    if bytes.get(i) == Some(&b'\'') {
        return (i + 1, Cow::Borrowed(" "));
    }
    (at + 1, Cow::Borrowed(" "))
}

/// Past the newline ending the line comment at `at`.
fn line_end(bytes: &[u8], at: usize) -> usize {
    let mut i = at;
    while bytes.get(i).is_some_and(|c| *c != b'\n') {
        i += 1;
    }
    i + 1
}

/// Past the `*/` closing the block comment whose body starts at `from`.
fn block_end(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    while let Some(c) = bytes.get(i) {
        if *c == b'*' && bytes.get(i + 1) == Some(&b'/') {
            return i + 2;
        }
        i += 1;
    }
    i
}
