//! **Every tool name that can reach this control from a granted invocation is
//! a name the control implements** (bl-d06c; VISION §4.11 item 1, DESIGN §8.6).
//!
//! The classifier folds names into a closed enum and sends everything outside
//! it to the fail-closed routed lane, whose answer for an input carrying no
//! command line is `opaque` — which the shipped table **holds**. That is the
//! right answer for a tool on a machine this control cannot see into, and the
//! wrong one for a built-in the engine grants every worker: the conversation
//! parks on every single call, the operator sees a question with tool calls and
//! no reply, and the remedy the hold offers (a `rules:` row naming a class) is
//! an operator statement about a tool the engine itself ships.
//!
//! That is not hypothetical, it is what bl-d06c filed. litany's `multi_tool`
//! envelope stood in the shipped worker grant with no row here, so a question
//! that fanned out two `clients get` reads held with nothing to answer. The
//! name retired upstream (litany bl-99bb, its `docs/DESIGN_CODE_EXECUTION.md`
//! §5 — the program is the envelope now), which settles that instance and
//! nothing about the class: the next name added to litany's pool, or a pin bump
//! that brings one, parks conversations exactly the same way. So the pinned
//! engine's own grant is read here — out of a workspace `yog litany new` really
//! authors, never a copy of it in this tree — and every name in it must
//! classify on the engine leg.
//!
//! Three sources, because a granted name has three origins and only one of them
//! is litany's file: the shipped `worker` grant, yog's own engine acts
//! ([`engine_act::NAMES`], which carries the compactor's injected pair — never
//! in any `providers.yaml`), and the roster tool yog adds to a workspace's grant
//! ([`clients::NAME`]).
//!
//! Both directions, like every guard in this repo. The grant must parse to a
//! non-empty list holding a name we know — a parse that quietly finds nothing
//! must not pass as green — and a name the map does not hold must still answer
//! [`Leg::Routed`]. The fail-closed lane is not softened by any of this; it is
//! only kept off the names the engine grants.
// The fixture helpers of an integration-test crate unwrap freely like any test.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use yog::control::classify::Leg;
use yog::tool_host::{clients, engine_act};

/// The role whose grant is what an interactive conversation can call out of the
/// box — litany's own words for it: "roots are workers".
const ROLE: &str = "worker";

/// Run the built `yog` with a hermetic world under `home`, so nothing here
/// reads or writes the operator's own state — and with git's own configuration
/// replaced, because `litany new` commits and a runner has no identity of its
/// own (`tests/multiplex_litany.rs` earned that lesson).
fn yog(home: &Path, args: &[&str]) -> i32 {
    let gitconfig = home.join("gitconfig");
    std::fs::write(
        &gitconfig,
        "[user]\n\tname = Tester\n\temail = t@test.invalid\n[commit]\n\tgpgsign = false\n",
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_yog"))
        .args(args)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home.join("home"))
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("GIT_CONFIG_GLOBAL", &gitconfig)
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .status()
        .unwrap()
        .code()
        .unwrap_or(-1)
}

/// `providers.yaml` as the pinned engine authors it, read off the first config
/// commit of a workspace this test just minted. `git_env::git()` rather than a
/// bare `Command`, so the read inherits the same scrub every child of this
/// crate does.
fn shipped_providers(repo: &Path) -> String {
    let out = yog::git_env::git()
        .arg("-C")
        .arg(repo)
        .args(["show", "config/default:providers.yaml"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// The `tools:` list one role declares. Textual on purpose: yog links no YAML
/// parser, and the shape being read is one inline sequence on one line of a
/// file the engine ships — a shape that changing would fail this parse loudly
/// rather than answering an empty grant.
fn grant(providers: &str, role: &str) -> Vec<String> {
    providers
        .lines()
        .skip_while(|l| l.trim_end() != format!("  {role}:"))
        .take_while(|l| l.starts_with("  "))
        .find_map(|l| l.trim().strip_prefix("tools: [")?.strip_suffix(']'))
        .map(|list| list.split(',').map(|n| n.trim().to_owned()).collect())
        .unwrap_or_default()
}

/// Mint a workspace with the real binary and answer its bare repo.
fn minted(home: &Path) -> PathBuf {
    let ws = home.join("ws");
    assert_eq!(yog(home, &["litany", "new", &ws.display().to_string()]), 0);
    ws.join("repo.git")
}

#[test]
fn every_name_a_granted_invocation_can_carry_has_an_intrinsic_row() {
    let home = tempfile::TempDir::new().unwrap();
    let providers = shipped_providers(&minted(home.path()));
    let granted = grant(&providers, ROLE);

    // The parse found the engine's own grant, not an empty list it could then
    // iterate zero times over.
    assert!(
        granted.len() > 3 && granted.iter().any(|n| n == "bash"),
        "the shipped {ROLE} grant did not parse: {granted:?}"
    );

    for name in granted
        .iter()
        .map(String::as_str)
        .chain(engine_act::NAMES)
        .chain([clients::NAME])
    {
        assert_eq!(
            Leg::of(name),
            Leg::Engine,
            "`{name}` is granted but has no row in the intrinsic map, so every \
             call of it classifies opaque and holds — the bl-d06c park. Add its \
             row to `src/control/classify/intrinsic.rs`, choosing the class its \
             reach earns."
        );
    }

    // The other direction: nothing here widened the map. A name no engine grants
    // is still the routed lane's, `multi_tool` — the retired envelope this ball
    // came from — included.
    for name in ["multi_tool", "box2_shell", "no_such_tool"] {
        assert_eq!(Leg::of(name), Leg::Routed, "`{name}`");
    }
}
