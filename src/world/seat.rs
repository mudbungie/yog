//! **Which seat may open a window** (DESIGN §16.4, bl-3ff4).
//!
//! The world seeds a `yog` shim ([`super::tools::YOG`]) so an agent's bash can
//! drive the §8.5 control boundary. That shim passes argv through verbatim, like
//! every other one — and a bare `yog`, with no gesture and no subcommand, is the
//! GUI. So the shim that lets an agent *ask* yog something also, unguarded, lets
//! an agent **launch a window on the operator's desktop**. Nothing in the design
//! wants that: I7's whole shape is that a window is the operator's own act.
//!
//! **The guard keys on an existing explicit signal, not a new flag.** `YOG_NAME`
//! is already stamped on every workspace-scoped spawn and already rides the
//! whole chain — detached driver, tool subprocess, the agent's bash, the shim
//! (§3.3, [`crate::multiplex`]'s default actor reads it for exactly this
//! reach). Its presence *is* "this seat is inside an agent". So an agent seat
//! that asks for a window is refused, and told what it can run instead.
//!
//! **Why the env rather than the shim.** A guard written into the shim script
//! would cover only the shim. Keyed on the environment it holds however yog was
//! reached — including the case that motivated the whole ball: an agent finding
//! the operator's *installed* `yog` on the ambient `PATH`, which is stale
//! against the build under drive (bl-d1af). The refusal is the same either way,
//! which is what makes it a property of the seat rather than of one path to it.
//!
//! **The operator is never caught by it.** `yog env` prints `LERNIE_HOME`,
//! `XDG_STATE_HOME` and `PATH` and nothing else (§8.4), so a human who ran
//! `eval "$(yog env)"` carries no `YOG_NAME` and opens windows exactly as
//! before. Nor is any yog-spawned child: every one of them names a namespace
//! verb, so none reaches the window seat at all.

/// The exit code an agent seat's window request answers with — the usage class,
/// the same one the wall-less `bz` refuses with (§16.2), because both are "you
/// asked for something this seat structurally does not have".
pub const REFUSED: i32 = 64;

/// The refusal for a seat that asked to open a window, or `None` when the seat
/// may have one. Pure over the raw `$YOG_NAME` so every branch is testable
/// without mutating process env; empty reads as absent, the env convention the
/// rest of yog follows.
///
/// Called at the one place a window is actually opened — after the namespace
/// arms, the hatches, `headless` and `tool-control` have all had their say — so
/// it judges only argv that would really have painted.
pub fn window_refusal(yog_name: Option<String>) -> Option<String> {
    let name = yog_name.filter(|n| !n.is_empty())?;
    Some(format!(
        "yog: no window from an agent seat (this is {name}'s). A window is the \
         operator's own act; what you can drive from here is yog's headless \
         surface — `yog gesture '/attention'` and every other gesture (`yog \
         gesture --help` lists them), `yog headless` for a windowless consumer, \
         and the `bl` / `lernie` / `bz` namespaces."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_operator_seat_opens_its_window() {
        assert_eq!(window_refusal(None), None);
    }

    /// Empty reads as absent — the env convention, and the case a shell that
    /// exported the var without a value would otherwise trip on.
    #[test]
    fn an_empty_name_is_no_name() {
        assert_eq!(window_refusal(Some(String::new())), None);
    }

    /// The refusal names the seat and points at what the agent can actually
    /// run — a refusal that names no paved path is the defect it replaces.
    #[test]
    fn an_agent_seat_is_refused_and_told_what_it_can_run() {
        let refusal = window_refusal(Some("cobalt".to_owned())).expect("refused");
        assert!(refusal.contains("cobalt"), "names the seat: {refusal}");
        assert!(refusal.contains("yog gesture"), "paved path: {refusal}");
        assert!(refusal.contains("headless"), "paved path: {refusal}");
    }
}
