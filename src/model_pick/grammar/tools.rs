//! The `tools:` half of the §9.4 block grammar: the flow-sequence round-trip
//! the picker's list control (§9.5) reads and writes a role's `tools:` field
//! through.
//!
//! The value is a **flow sequence** (`[a, b, c]`) — the one place lernie's
//! template leaves the block form — so this is the only field the grammar reads
//! as a list.

/// The names in an inline flow sequence (`[a, b, c]` → `["a","b","c"]`), or
/// `None` when the value is not that form. `[]` is a value (no tools), not a
/// fault; interior spacing is normalized away because the rewrite re-emits it.
/// Shared with the §9.5 pane, whose list control is this same round-trip.
pub fn flow_members(value: &str) -> Option<Vec<String>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?;
    Some(
        inner
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

/// Member names back into the inline flow sequence the template writes — the
/// inverse of [`flow_members`], so the shape survives every round trip.
pub fn flow_value(members: &[String]) -> String {
    format!("[{}]", members.join(", "))
}
