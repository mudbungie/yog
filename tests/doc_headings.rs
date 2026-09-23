//! **A section number is an ADDRESS, so it must name exactly one section**
//! (bl-1732). `docs/REMOTE.md` carried two sections numbered §9.20 — bl-e59e's
//! trail ruling and bl-d13d's spend ruling, landed the same evening, each
//! taking the next free number without seeing the other. Both were then cited
//! by number: §9.20 from lernie's and yog-android's own `docs/DESIGN.md`, and
//! the spend one from this repo's DESIGN §3.5 and VISION §4.5. A citation
//! cannot resolve a number that names two things, and with four separately
//! installed components (REMOTE §12) the mis-resolution happens in a tree
//! whose author cannot see this doc.
//!
//! `design_citations.rs` already guards the other end — every `§N.M` cited
//! from `src/` resolves to a DESIGN heading — but it asks whether a number
//! *resolves*, never whether it resolves to ONE thing. This file asks that,
//! over the three living design documents, and it is a separate test target
//! because `design_citations.rs` is a crate root already at the pre-split
//! band and the two guards share no scanner.
//!
//! **Only `## N.` and `### N.M` count.** That is the depth at which these
//! documents number their sections and the exact depth `design_citations.rs`
//! parses, so the two guards agree on what a section IS. `#### ` is out on
//! purpose: DESIGN §3.7 numbers five prose *steps* `#### 1.` … `#### 5.`,
//! which are list items in one section and not addresses anything cites.
//!
//! **An `amended:` heading is exempt, deliberately.** REMOTE carries
//! `### 9.11 amended: the ledger refuses against the PUBLISHED floor`, and it
//! is not a second §9.11 — it is a back-reference that revises a section that
//! lives 400 lines earlier, written where the wave that revised it lands so a
//! reader of the wave meets the amendment in its own context. Folding it into
//! §9.11 was the alternative and is worse: it would move the argument away
//! from the ruling that made it, and §9.11's own body already carries the
//! forward pointer. So the shape is admitted BY RULE rather than by an
//! allowlist, and [`an_amendment_names_a_real_section`] keeps the rule from
//! becoming a hiding place: the number an amendment heading states must
//! already exist as a real heading in the same document, so a typo'd or
//! invented number fails here instead of passing as an amendment.

use std::collections::BTreeMap;

/// The documents whose section numbers are addresses other repositories hold.
const DOCS: &[&str] = &["docs/DESIGN.md", "docs/REMOTE.md", "docs/VISION.md"];

/// One numbered heading: its key (`"9"` or `"9.20"`), the text after the key,
/// and the 1-based line it stands on.
struct Head {
    key: String,
    tail: String,
    line: usize,
}

impl Head {
    /// An amendment back-reference (`### 9.11 amended: …`) rather than a
    /// section of its own — see this file's doc comment.
    fn is_amendment(&self) -> bool {
        self.tail.starts_with("amended")
    }
}

/// Parse a section key (`digits`, optionally `.digits`) off the front of `s`;
/// returns the key and the rest of the line, or `None` if `s` does not open
/// with a digit.
fn key_at(s: &str) -> Option<(String, String)> {
    let mut key = String::new();
    let mut rest = s.chars().peekable();
    while rest.peek().is_some_and(char::is_ascii_digit) {
        key.extend(rest.next());
    }
    if key.is_empty() {
        return None;
    }
    let mut lookahead = rest.clone();
    if lookahead.next() == Some('.') && lookahead.peek().is_some_and(char::is_ascii_digit) {
        key.push('.');
        rest.next();
        while rest.peek().is_some_and(char::is_ascii_digit) {
            key.extend(rest.next());
        }
    }
    let tail: String = rest.collect();
    Some((key, tail.trim_start_matches(['.', ' ']).to_owned()))
}

/// Every `## N.` / `### N.M` heading in `text`, in document order.
fn headings(text: &str) -> Vec<Head> {
    text.lines()
        .enumerate()
        .filter_map(|(n, line)| {
            let rest = line
                .strip_prefix("## ")
                .or_else(|| line.strip_prefix("### "))?;
            let (key, tail) = key_at(rest)?;
            Some(Head {
                key,
                tail,
                line: n + 1,
            })
        })
        .collect()
}

/// The keys that name more than one non-amendment heading, each with the lines
/// that claim it — the finding this guard exists to produce.
fn collisions(text: &str) -> Vec<String> {
    let mut seen: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for h in headings(text).iter().filter(|h| !h.is_amendment()) {
        seen.entry(h.key.clone()).or_default().push(h.line);
    }
    seen.into_iter()
        .filter(|(_, lines)| lines.len() > 1)
        .map(|(key, lines)| format!("§{key} claimed by lines {lines:?}"))
        .collect()
}

/// A forgiving read, the sweep idiom `design_citations.rs` already uses: a
/// vanished document yields nothing, and [`the_scan_is_not_vacuous`] is what
/// keeps "nothing" from passing as a clean verdict.
fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn every_section_number_names_one_section() {
    for doc in DOCS {
        let found = collisions(&read(doc));
        assert!(
            found.is_empty(),
            "{doc} numbers two sections the same — a number is an address \
             other repositories cite, so give the LATER section the next free \
             number rather than shifting the ones already cited:\n{}",
            found.join("\n")
        );
    }
}

#[test]
fn an_amendment_names_a_real_section() {
    for doc in DOCS {
        let text = read(doc);
        let all = headings(&text);
        let real: Vec<&String> = all
            .iter()
            .filter(|h| !h.is_amendment())
            .map(|h| &h.key)
            .collect();
        for h in all.iter().filter(|h| h.is_amendment()) {
            assert!(
                real.contains(&&h.key),
                "{doc}:{} amends §{}, which is not a section of this document",
                h.line,
                h.key
            );
        }
    }
}

#[test]
fn the_scan_is_not_vacuous() {
    for doc in DOCS {
        let all = headings(&read(doc));
        assert!(all.len() >= 10, "{doc}: only {} headings parsed", all.len());
        assert!(
            all.iter().any(|h| h.key.contains('.')),
            "{doc}: no dotted subsection headings parsed"
        );
    }
}

/// The other direction: the finder must FIRE. A guard that matches nothing
/// passes forever, so the shape it is written to catch is pinned here beside
/// the shape it must wave through (`scripts/leak-scan.sh --self-test` and
/// `make rules-audit` keep the same two-direction discipline).
#[test]
fn the_finder_fires_on_a_duplicate_and_not_on_an_amendment() {
    let dup = "## 9. Build\n\n### 9.20 One (bl-aaaa)\n\n### 9.20 Two (bl-bbbb)\n";
    assert_eq!(
        collisions(dup),
        vec!["§9.20 claimed by lines [3, 5]".to_owned()],
        "the duplicate finder did not fire on a planted duplicate"
    );

    let amended = "### 9.11 A rule (bl-aaaa)\n\n### 9.11 amended: and its revision (bl-bbbb)\n";
    assert!(
        collisions(amended).is_empty(),
        "an `amended:` back-reference was read as a second section"
    );

    // `#### ` is a prose step, not an address (DESIGN §3.7's five).
    let steps = "#### 1. Discovery\n\n#### 1. Also discovery\n";
    assert!(
        collisions(steps).is_empty(),
        "a `#### ` list item was scanned"
    );

    // A heading with no number at all is not a section key.
    assert!(key_at("Open questions (living)").is_none());
}
