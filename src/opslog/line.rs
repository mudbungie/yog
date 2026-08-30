//! The `ops.jsonl` line codec, both directions (DESIGN §4.2): the ≤[`CAP`]
//! serializer, the caller-side argv clip that keeps a composed prompt goal
//! inside that bound, and the forgiving parser that reads a line back. Writer
//! and reader live together because they are one format — split apart, the two
//! can drift and nothing catches it. Nothing here touches the filesystem or a
//! clock, so every branch is a pure, deterministic test.

use super::{CAP, OpEntry, Origin};
use serde_json::{Map, Value};

/// The line's §7.3 attribution key (see [`Origin`]). A fixed field like
/// `ts`/`cwd`/`exit`: short, never truncated, and absent only from a line an
/// older yog wrote (which the parser reads as the default).
const ORIGIN: &str = "origin";

/// The pure ≤[`CAP`] serializer: JSON object plus `'\n'`, truncating `stdout`
/// then `stderr` (heads kept) with a `"truncated":true` marker whenever the
/// full line would exceed the cap. The fixed fields never truncate.
pub fn build_line(entry: &OpEntry) -> Vec<u8> {
    let full = serialize(entry, &entry.stdout, &entry.stderr, false);
    if full.len() <= CAP {
        return full;
    }
    // Truncation required. stdout is sacrificed before stderr (§4.2).
    if serialize(entry, "", &entry.stderr, true).len() <= CAP {
        let kept = largest_fit(entry.stdout.len(), |n| {
            serialize(entry, head(&entry.stdout, n), &entry.stderr, true).len()
        });
        return serialize(entry, head(&entry.stdout, kept), &entry.stderr, true);
    }
    let bare = serialize(entry, "", "", true);
    if bare.len() > CAP {
        // Fixed fields alone exceed the cap — the one unavoidable overflow.
        return bare;
    }
    let kept = largest_fit(entry.stderr.len(), |n| {
        serialize(entry, "", head(&entry.stderr, n), true).len()
    });
    serialize(entry, "", head(&entry.stderr, kept), true)
}

/// Serialize one line (object + newline) with the given `stdout`/`stderr`
/// bodies and truncation marker. Keys land in `BTreeMap` (alphabetical) order,
/// deterministic since serde_json here has no `preserve_order` feature.
fn serialize(entry: &OpEntry, stdout: &str, stderr: &str, truncated: bool) -> Vec<u8> {
    let mut map = Map::new();
    map.insert("ts".into(), Value::from(entry.ts.as_str()));
    map.insert("argv".into(), Value::from(entry.argv.clone()));
    map.insert("cwd".into(), Value::from(entry.cwd.as_str()));
    map.insert("exit".into(), Value::from(entry.exit));
    map.insert(ORIGIN.into(), Value::from(entry.origin.as_str()));
    map.insert("stdout".into(), Value::from(stdout));
    map.insert("stderr".into(), Value::from(stderr));
    if truncated {
        map.insert("truncated".into(), Value::Bool(true));
    }
    // A JSON object of strings and integers is always serializable; the
    // impossible error degrades to an empty (newline-only) line, never a panic.
    let mut bytes = serde_json::to_vec(&Value::Object(map)).unwrap_or_default();
    bytes.push(b'\n');
    bytes
}

/// Largest `n` in `0..=max` with `len(n) <= CAP`, given `len(0) <= CAP` and
/// `len` monotonic non-decreasing. Plain upper-biased binary search.
fn largest_fit(max: usize, len: impl Fn(usize) -> usize) -> usize {
    let (mut lo, mut hi) = (0usize, max);
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if len(mid) <= CAP {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

/// Return `entry` with its **last argv element** — the caller's one deliberately
/// large field (the composed `litany prompt` goal, §8.1) — clipped so
/// [`build_line`] holds ≤ [`CAP`] *after* JSON escaping. `argv` is never
/// truncated inside [`build_line`] (a pathological argv is the one unavoidable
/// overflow), so the known-large element is clipped here, at its source, against
/// the real serialized length ([`largest_fit`] over [`clip_arg`]) — [`CAP`] stays
/// the single bound the module derives from, never a raw-byte proxy JSON escaping
/// can blow past. An already-fitting goal rides back verbatim; a clipped one
/// carries the `… [+N bytes elided]` marker. An empty argv has nothing to clip.
pub fn clip_goal(entry: &OpEntry) -> OpEntry {
    let Some((goal, head_argv)) = entry.argv.split_last() else {
        return entry.clone();
    };
    if build_line(entry).len() <= CAP {
        return entry.clone();
    }
    let rebuilt = |g: String| {
        let mut argv = head_argv.to_vec();
        argv.push(g);
        OpEntry {
            argv,
            ..entry.clone()
        }
    };
    let kept = largest_fit(goal.len(), |n| {
        build_line(&rebuilt(clip_arg(goal, n))).len()
    });
    rebuilt(clip_arg(goal, kept))
}

/// Clip an **argv element** to at most `max` bytes for the log, appending an
/// explicit `… [+N bytes elided]` marker when it truncates (§4.2). [`clip_goal`]
/// drives it under a [`largest_fit`] search so the *serialized* line lands ≤
/// [`CAP`]/PIPE_BUF post-escape; `argv` never truncates inside [`build_line`]
/// itself, so the one deliberately-large element (a composed `litany prompt`
/// goal) is clipped here at its source — the *spawned* goal stays full (it is
/// derivable from the workspace, so full fidelity is never the log's job).
pub(super) fn clip_arg(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    let kept = head(s, max);
    format!("{kept}… [+{} bytes elided]", s.len() - kept.len())
}

/// The longest UTF-8-valid prefix of `s` that is at most `max_bytes` long.
fn head(s: &str, max_bytes: usize) -> &str {
    let mut i = max_bytes.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    // `i` is a char boundary `<= s.len()`, so the get always yields `Some`.
    s.get(..i).unwrap_or(s)
}

/// Parse one line into an [`OpEntry`], or `None` when it is not a JSON object.
/// Individual fields default when absent or mistyped (forgiving, per §4.2).
pub(super) fn parse_line(line: &str) -> Option<OpEntry> {
    let value: Value = serde_json::from_str(line).ok()?;
    let obj = value.as_object()?;
    Some(OpEntry {
        ts: str_field(obj, "ts"),
        argv: obj
            .get("argv")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        cwd: str_field(obj, "cwd"),
        exit: obj.get("exit").and_then(Value::as_i64).unwrap_or(0) as i32,
        stdout: str_field(obj, "stdout"),
        stderr: str_field(obj, "stderr"),
        origin: Origin::parse(&str_field(obj, ORIGIN)),
    })
}

/// A string field of `obj`, or `""` when absent or non-string.
fn str_field(obj: &Map<String, Value>, key: &str) -> String {
    obj.get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests;
