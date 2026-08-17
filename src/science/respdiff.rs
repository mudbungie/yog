//! The **response diff** between two candidates' terminal responses (VISION V3
//! item 3, bl-77bc): a pure line comparison, computed at read time from the two
//! strings the science rows already carry and stored nowhere.
//!
//! Its own small algorithm rather than a dependency, deliberately: the
//! comparison the operator judges by is line-grained and bounded (a terminal
//! response, not a repository), the crate takes zero new dependencies without
//! an operator ruling, and `git diff` is the wrong tool here because the two
//! responses live in two different conversation repos — there is no one tree to
//! ask git about.
//!
//! The algorithm is the ordinary longest-common-subsequence table over lines,
//! capped: past [`LINE_CAP`] lines a response is compared on its head and the
//! diff says so, because an O(n·m) table over two unbounded model outputs is a
//! frame stall waiting for a long transcript — and the seat renders a
//! comparison, not an archive.

/// The most lines of each response the table compares. Past it the diff says
/// so ([`Diff::truncated`]) rather than silently comparing a prefix.
pub const LINE_CAP: usize = 400;

/// One compared line and whose it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// In both responses.
    Same(String),
    /// Only in the left response.
    Left(String),
    /// Only in the right response.
    Right(String),
}

/// The comparison: rows in order, and whether either side was cut at
/// [`LINE_CAP`] before comparing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Diff {
    pub rows: Vec<Row>,
    pub truncated: bool,
}

/// Compare two terminal responses line by line. Equal inputs answer all-`Same`
/// rows; an empty side answers the other side whole — the general path with
/// one input missing, not a special case.
pub fn lines(left: &str, right: &str) -> Diff {
    let heads: (Vec<&str>, Vec<&str>) = (
        left.lines().take(LINE_CAP).collect(),
        right.lines().take(LINE_CAP).collect(),
    );
    let truncated = left.lines().nth(LINE_CAP).is_some() || right.lines().nth(LINE_CAP).is_some();
    let table = Table::of(&heads.0, &heads.1);
    Diff {
        rows: walk(&heads.0, &heads.1, &table),
        truncated,
    }
}

/// The LCS length table, `(l.len()+1) x (r.len()+1)`, row-major — with checked
/// reads, so an out-of-range ask is the 0 the table's border rows hold.
struct Table {
    cells: Vec<usize>,
    width: usize,
}

impl Table {
    fn of(left: &[&str], right: &[&str]) -> Table {
        let width = right.len() + 1;
        let mut table = Table {
            cells: vec![0; (left.len() + 1) * width],
            width,
        };
        for (i, line) in left.iter().enumerate().rev() {
            for (j, other) in right.iter().enumerate().rev() {
                let cell = if line == other {
                    table.at(i + 1, j + 1) + 1
                } else {
                    table.at(i + 1, j).max(table.at(i, j + 1))
                };
                if let Some(seat) = table.cells.get_mut(i * width + j) {
                    *seat = cell;
                }
            }
        }
        table
    }

    /// One cell, checked: the border and anything past it read 0.
    fn at(&self, i: usize, j: usize) -> usize {
        self.cells.get(i * self.width + j).copied().unwrap_or(0)
    }
}

/// Read the table back into rows, left-first on ties so a replacement reads
/// as remove-then-add — the order every unified diff has taught.
fn walk(left: &[&str], right: &[&str], table: &Table) -> Vec<Row> {
    let (mut i, mut j) = (0, 0);
    let mut rows = Vec::new();
    while let (Some(line), Some(other)) = (left.get(i), right.get(j)) {
        if line == other {
            rows.push(Row::Same((*line).to_owned()));
            i += 1;
            j += 1;
        } else if table.at(i + 1, j) >= table.at(i, j + 1) {
            rows.push(Row::Left((*line).to_owned()));
            i += 1;
        } else {
            rows.push(Row::Right((*other).to_owned()));
            j += 1;
        }
    }
    rows.extend(left.iter().skip(i).map(|s| Row::Left((*s).to_owned())));
    rows.extend(right.iter().skip(j).map(|s| Row::Right((*s).to_owned())));
    rows
}
