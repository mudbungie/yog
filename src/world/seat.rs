//! **Which seat may boot the engine** (DESIGN §16.4, bl-3ff4; retargeted for
//! the server by bl-7942).
//!
//! The world seeds a `yog` shim ([`super::tools::YOG`]) so an agent's bash can
//! drive the §8.5 control boundary. That shim passes argv through verbatim,
//! like every other one — and a bare `yog`, with no gesture and no subcommand,
//! **boots the engine**. So the shim that lets an agent *ask* yog something
//! also, unguarded, lets an agent found a second engine on the world it is
//! already running inside: two pilots, two sentries, two derivation workers —
//! the instance-coordination shape DESIGN §14 rejects.
//!
//! It guarded a *window* until bl-7942, on I7's reasoning that a window is the
//! operator's own act. The severance did not weaken that reasoning, it moved
//! what stands behind the bare word: booting the box's server is the
//! operator's own act for the same reason and one better, since the engine an
//! agent is speaking to is the one it would be duplicating.
//!
//! **The guard keys on an existing explicit signal, not a new flag.** `YOG_NAME`
//! is already stamped on every workspace-scoped spawn and already rides the
//! whole chain — detached driver, tool subprocess, the agent's bash, the shim
//! (§3.3, [`crate::multiplex`]'s default actor reads it for exactly this
//! reach). Its presence *is* "this seat is inside an agent". So an agent seat
//! that asks to boot is refused, and told what it can run instead.
//!
//! **Why the env rather than the shim.** A guard written into the shim script
//! would cover only the shim. Keyed on the environment it holds however yog was
//! reached — including the case that motivated the whole ball: an agent finding
//! the operator's *installed* `yog` on the ambient `PATH`, which is stale
//! against the build under drive (bl-d1af). The refusal is the same either way,
//! which is what makes it a property of the seat rather than of one path to it.
//!
//! **The operator is never caught by it.** `yog env` prints `LITANY_HOME`,
//! `XDG_STATE_HOME` and `PATH` and nothing else (§8.4), so a human who ran
//! `eval "$(yog env)"` carries no `YOG_NAME` and boots exactly as before. Nor
//! is any yog-spawned child: every one of them names a namespace verb, so none
//! reaches the boot at all.

/// The exit code an agent seat's boot request answers with — the usage class,
/// the same one the wall-less `bz` refuses with (§16.2), because both are "you
/// asked for something this seat structurally does not have".
pub const REFUSED: i32 = 64;

/// The refusal for a seat that asked to boot the engine, or `None` when the
/// seat may. Pure over the raw `$YOG_NAME` so every branch is testable without
/// mutating process env; empty reads as absent, the env convention the rest of
/// yog follows.
///
/// Called at the one place the engine is actually booted — after the namespace
/// arms, the hatches and `tool-control` have all had their say — so it judges
/// only argv that would really have booted.
pub fn boot_refusal(yog_name: Option<String>) -> Option<String> {
    let name = yog_name.filter(|n| !n.is_empty())?;
    Some(format!(
        "yog: no engine boot from an agent seat (this is {name}'s). You are \
         already inside a running yog and a second one would be a rival engine \
         on one world; what you can drive from here is its boundary — `yog \
         gesture '/attention'` and every other gesture (`yog gesture --help` \
         lists them), and the `bl` / `litany` / `bz` namespaces."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_operator_seat_boots() {
        assert_eq!(boot_refusal(None), None);
    }

    /// Empty reads as absent — the env convention, and the case a shell that
    /// exported the var without a value would otherwise trip on.
    #[test]
    fn an_empty_name_is_no_name() {
        assert_eq!(boot_refusal(Some(String::new())), None);
    }

    /// The refusal names the seat and points at what the agent can actually
    /// run — a refusal that names no paved path is the defect it replaces.
    #[test]
    fn an_agent_seat_is_refused_and_told_what_it_can_run() {
        let refusal = boot_refusal(Some("cobalt".to_owned())).expect("refused");
        assert!(refusal.contains("cobalt"), "names the seat: {refusal}");
        assert!(refusal.contains("yog gesture"), "paved path: {refusal}");
        assert!(refusal.contains("bl"), "paved path: {refusal}");
    }
}
