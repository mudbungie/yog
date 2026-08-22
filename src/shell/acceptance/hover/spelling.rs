//! §11 discoverability **rule 3**, machine-held: a control's hover names its
//! keyboard spelling (bl-478d).
//!
//! The ruling is one sentence: everything is keyboard-operable, and every
//! button's mouseover states the combo that fires it. [`super`] holds that a control says *what pressing it
//! does*; this holds the other half — that it also says **how to press it
//! without the mouse**.
//!
//! The vocabulary is derived from the two authorities and restated nowhere:
//! [`crate::keymap::spell`] sweeps the §11 binding table itself, and
//! [`crate::boundary::help`] is the §8.5 verb roster. So a control may spell
//! itself as a key (`(f)`, `Ctrl+N`) or, where its gesture's address is a line,
//! as that line (`/release`, `/config`) — F1's own wording, and the reason a
//! pick is not a hover exemption. Rebinding a key rewrites what this test accepts,
//! in lockstep, because there is no second list to update.

use super::lex::{skeleton, text_of};
use super::scan::{args_of, chain_of, rust_files, sites};
use super::{CONTROLS, HOVERS};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A name looked up outside the seat's own file is looked up once. The tree is
/// one string by then, so the memo is what keeps the walk from re-reading it
/// per site.
type Memo = HashMap<String, Vec<String>>;

/// How far a delegated hover is followed. A seat that hands its words to a
/// `const` may itself have been handed them (`let hover = if pinned { PINNED }
/// else { RULE }`), and three hops covers every chain in this tree while
/// keeping the walk finite.
const HOPS: usize = 3;

/// How much of a mute control's words the failure quotes — enough to recognize
/// the seat, short of pasting the tree back at the reader.
const WIDTH: usize = 100;

/// **The invariant.** Every control that carries a hover names, in it, the
/// keyboard spelling of the gesture it makes. Enumerating nothing is itself a
/// failure — the same two-direction discipline the rest of the scan keeps.
#[test]
fn every_control_hover_names_its_keyboard_spelling() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let sources: Vec<(PathBuf, String)> = rust_files(&root)
        .into_iter()
        .map(|file| {
            let skeleton = skeleton(&std::fs::read_to_string(&file).unwrap());
            (file, skeleton)
        })
        .collect();
    let tree: String = sources
        .iter()
        .map(|(_, s)| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let vocabulary = vocabulary();
    let (mut mute, mut seen, mut memo) = (Vec::new(), 0usize, Memo::new());
    for (file, skeleton) in &sources {
        for (at, control) in sites(skeleton, CONTROLS) {
            let hovers: Vec<usize> = chain_of(skeleton, at)
                .into_iter()
                .filter(|(name, _)| HOVERS.contains(&name.as_str()))
                .filter_map(|(_, called)| called)
                .collect();
            if hovers.is_empty() {
                // A control that says nothing at all is [`super`]'s failure,
                // reported there with its own words. One violation, one home.
                continue;
            }
            seen += 1;
            let said: String = hovers
                .into_iter()
                .map(|open| says(skeleton, &tree, &mut memo, args_of(skeleton, open)))
                .collect();
            if !vocabulary.iter().any(|spelling| said.contains(spelling)) {
                let words: String = said.chars().take(WIDTH).collect();
                mute.push(format!("{}: {control} — said {words}", file.display()));
            }
        }
    }
    assert!(
        seen > 50,
        "the scan matched {seen} hovers — the pattern list has rotted"
    );
    assert!(
        mute.is_empty(),
        "these controls hide their keyboard spelling — §11 rule 3 requires every \
         hover to name the key that presses it, or the §8.5 line that addresses \
         it:\n{}",
        mute.join("\n")
    );
}

/// The vocabulary a hover may spell a gesture in: every press the §11 table
/// binds, and one `/verb` per §8.5 line. Read from the authorities themselves,
/// so a binding renamed or a verb retired takes its spelling with it.
fn vocabulary() -> Vec<String> {
    let mut words = crate::keymap::spell::spellings();
    words.push(crate::keymap::spell::FLOOR.to_owned());
    words.extend(
        crate::boundary::help::table()
            .iter()
            .map(|row| format!("/{}", row.verb)),
    );
    words
}

/// The words a hover argument says. A seat either **says** its sentence — then
/// the literal is the whole answer — or **hands it over by name**, which is
/// rule 4's own carve-out: a phrase worn twice is a named `const`, and a set of
/// seats is one exhaustive `fn`. Names are followed only where nothing was
/// said, which is what stops a prose hover dragging in an unrelated item that
/// happens to share a word with it.
fn says(file: &str, tree: &str, memo: &mut Memo, arg: &str) -> String {
    let (mut words, mut spans, mut asked) = (String::new(), vec![text_of(arg)], Vec::new());
    for _ in 0..HOPS {
        let mut next = Vec::new();
        for span in spans {
            let said = span.contains('"');
            words.push_str(&quoted(&span));
            // A sentence that names nothing but words says what it says; a
            // `SCREAMING` name in it is a const, and prose is never one, so
            // the phrase it holds is followed either way.
            for ident in idents(&span) {
                let named = ident == ident.to_uppercase();
                if (named || !said) && !asked.contains(&ident) {
                    next.extend(homes(file, tree, memo, &ident));
                    asked.push(ident);
                }
            }
        }
        spans = next;
    }
    words
}

/// Where a name's words are written, the seat's own file answering first — a
/// hint is nearly always a `const` beside the control it serves — and the tree
/// only for a name that file does not define.
fn homes(file: &str, tree: &str, memo: &mut Memo, ident: &str) -> Vec<String> {
    let own = defined_in(file, ident);
    if !own.is_empty() {
        return own;
    }
    memo.entry(ident.to_owned())
        .or_insert_with(|| defined_in(tree, ident))
        .clone()
}

/// Every definition of `ident` in one file's skeleton, as its text: a `const`
/// or a `let` ends at its semicolon, a `fn` where the next one begins. Both
/// bounds are coarse on purpose — over-reading costs a false pass on a
/// neighbour's words, while brace-matching a skeleton that has no braces left
/// would cost the walk itself.
fn defined_in(skeleton: &str, ident: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (keyword, closes) in [("const ", ";"), ("let ", ";"), ("fn ", "fn ")] {
        let needle = format!("{keyword}{ident}");
        let mut from = 0;
        while let Some(hit) = skeleton.get(from..).and_then(|rest| rest.find(&needle)) {
            from = from + hit + needle.len();
            let rest = skeleton.get(from..).unwrap_or_default();
            // `const HINT` is not `const HINTS`.
            if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let end = rest.find(closes).unwrap_or(rest.len());
            found.push(text_of(rest.get(..end).unwrap_or_default()));
        }
    }
    found
}

/// The literals in a skeleton span, stripped of the code around them. A
/// definition is read for its **words**: `CenterTab` is a name in the source,
/// never something a hover said, and only the quoted halves can be either.
fn quoted(span: &str) -> String {
    span.split('"')
        .skip(1)
        .step_by(2)
        .collect::<Vec<_>>()
        .join(" ")
}

/// The identifier-shaped words in a span — what a seat that said nothing is
/// asking for by name.
fn idents(span: &str) -> Vec<String> {
    span.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word| word.len() > 1)
        .map(str::to_owned)
        .collect()
}
