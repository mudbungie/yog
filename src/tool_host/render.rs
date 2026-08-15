//! **The dated observation** (REMOTE §5, bl-c907): what the `clients` tool
//! actually appends to the model's context.
//!
//! REMOTE §5: *"Every reply is a dated observation appended to context, free to
//! go stale, never a prefix mutation — a presence flap cannot touch what the
//! model already read, and the prompt cache (keyed on the prefix) survives every
//! blip."* So every rendering here opens with the instant it was observed at.
//! That is not decoration: presence is true only at the moment it was read, and
//! a line that did not say when it was read would be a claim about now.
//!
//! It is **text, not JSON**, because the reader is a model and the result
//! envelope carries bytes (lernie ARCH §3.3): a shape a model has to parse to
//! act on buys nothing a sentence does not. Each rendering is a list of lines
//! joined once, so no line is appended to a string that has already been built.

use super::loaded::Entry;
use crate::registry::roster::ClientRow;

/// `op=list`: every registered client of the workspace, and which hold a live
/// connection this instant.
pub fn list(workspace: &str, observed: &str, rows: &[ClientRow]) -> String {
    let head = format!("clients registered in workspace {workspace:?}, observed {observed}");
    if rows.is_empty() {
        return lines(&[head, String::new(), "(none is registered here)".to_owned()]);
    }
    let mut out = vec![head, String::new()];
    out.extend(rows.iter().map(|row| {
        format!(
            "  {} — {}, advertising {}",
            row.client,
            presence(row.present),
            plural(row.tools.len(), "tool")
        )
    }));
    out.push(String::new());
    out.push("op=get with client=<name> for one client's advertised tools.".to_owned());
    lines(&out)
}

/// `op=get`: one client's detail, and each advertised tool with the name it
/// would become callable under.
pub fn get(workspace: &str, observed: &str, row: &ClientRow) -> String {
    let mut out = vec![
        format!(
            "client {:?} of workspace {workspace:?}, observed {observed}",
            row.client
        ),
        format!("  {}", presence(row.present)),
        String::new(),
    ];
    if row.tools.is_empty() {
        out.push("It advertises no tools.".to_owned());
        return lines(&out);
    }
    out.push(format!(
        "It advertises {}:",
        plural(row.tools.len(), "tool")
    ));
    for tool in &row.tools {
        let entry = Entry {
            client: row.client.clone(),
            tool: tool.clone(),
        };
        out.push(format!("  {} — {}", tool.name, tool.description));
        out.push(format!("    loads as: {}", entry.presented()));
    }
    out.push(String::new());
    out.push(format!(
        "op=load with client={:?} and tools=[…] makes them callable, \
         by the names above, from the next step on.",
        row.client
    ));
    lines(&out)
}

/// `op=load`: what became callable, and how large this agent's set now is.
pub fn load(observed: &str, added: &[Entry], total: usize) -> String {
    let mut out = vec![format!("loaded, observed {observed}"), String::new()];
    out.extend(
        added
            .iter()
            .map(|entry| format!("  {} — {}", entry.presented(), entry.tool.description)),
    );
    out.push(String::new());
    out.push(format!(
        "Callable from the next step on. This conversation now holds {}. \
         There is no unload: a fresh conversation starts with none.",
        plural(total, "loaded tool")
    ));
    lines(&out)
}

/// The one join: newline-separated, newline-terminated.
fn lines(rows: &[String]) -> String {
    format!("{}\n", rows.join("\n"))
}

/// The presence half of a row, in the model's own reading: a connection now, or
/// none now. Never a duration — presence is an instant, not a history.
fn presence(present: bool) -> &'static str {
    if present {
        "connected right now"
    } else {
        "not connected right now"
    }
}

/// `n noun` / `n nouns`, so no rendering carries a "1 tools".
fn plural(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("{n} {noun}")
    } else {
        format!("{n} {noun}s")
    }
}
