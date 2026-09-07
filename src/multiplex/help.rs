//! The **argv seat's** help (DESIGN §8.5's higher-order rule, applied above the
//! router — bl-52ed). Help is asked *about* a command, so it is read once here
//! rather than by each command: `yog --help|-h|help [command]` is the whole
//! surface or one command's page, and `yog <command> --help|-h` is that
//! command's page. Nothing routes, composes, spawns, parks or writes first.
//!
//! **Which commands answer here, and which answer themselves.** yog's own
//! subcommands have no interface of their own to consult, so their page is
//! [`COMMANDS`] and this module prints it. A namespace that owns its argv
//! ([`super::Namespace::owns_argv`]) does not: its `--help` is the embedded
//! tool's own answer, so the ask is passed through to the arm, which must reach
//! it **world-free** — that is why [`is_discovery`] exists and why `bl`'s shim
//! converge and `bz`'s wall gate both step aside for a probe.
//!
//! **Neither side is enumerated here, deliberately** (bl-0a74). This paragraph
//! used to list both, and both rotted at the severance (bl-7942): it still
//! named `tool-host` as the one namespace that routes without owning its argv
//! and `seat` as one that does, long after the wire's client modes had left for
//! the seat crate — sending a reader after a distinction that no longer had a
//! member. The two tables below and beside are the answer to each question.
//!
//! [`COMMANDS`] is the single source: the top-level roster ([`super::usage`]),
//! every per-command page, and the parity tests all read this one list. A row
//! with an empty `summary` is answerable but **unadvertised** — the same rule
//! [`NAMESPACES`](super::NAMESPACES) applies to balls' two plugin binaries.

use crate::boundary::help::{HelpRow, Surface, render};

#[cfg(test)]
mod tests;

/// Every top-level word yog answers help about — its usage line, the one line
/// the roster gives it (empty = unadvertised), and the paragraph a `--help`
/// prints. `verb` is the const its dispatcher routes on, so a page cannot name
/// a word that does not run.
///
/// **Every row here is [`Surface::Machine`], and that is structural rather than
/// a judgement per row** (`docs/PARITY.md` §2, bl-8758). The class says whether
/// a *seat-class client* owes the op a discoverable interactable, and these are
/// not ops: they are process-level words that cross no §8.5 control boundary,
/// so no seat can owe one — a window cannot offer a control for "run the
/// embedded balls CLI in this process". The parity roster never sees them
/// either, being generated from [`boundary::help::table`](crate::boundary::help::table)
/// alone. They share [`HelpRow`] for its four text facts and [`render`], which
/// is the whole of what an argv page needs.
pub(super) const COMMANDS: &[HelpRow] = &[
    HelpRow {
        verb: crate::wire::provision::verb::SUBCMD,
        usage: "yog wire-certs   (every setting is an environment reading, stated before it)",
        summary: "mint this box's wire certificates, state where the engine listens, or issue \
                  one leaf for another box",
        detail: "Every setting is an environment reading, so it goes BEFORE the verb — a \
                 `VAR=value` after it is argv and is refused.\n\
                 \n  WIRE_HOST=<host>[,<host>…]  every host the server leaf answers to. The \
                 first, on WIRE_PORT, is the address; 127.0.0.1 always rides beside them, so a \
                 box reachable by name, by overlay address and on the LAN says so once.\n  \
                 WIRE_PORT=<port>            what the engine binds and a seat dials \
                 (default 7737). Unstated over material already here, the port `address` \
                 already names is kept.\n  \
                 WIRE_DIR=<dir>              the material directory (default: this world's own).\n  \
                 FORCE=1                     rotate: delete every artifact and found a new trust \
                 root. This distrusts every certificate already issued, everywhere.\n  \
                 WIRE_LEAF=<common-name>     issue ONE extra client leaf under that name.\n  \
                 WIRE_FOOT=1                 beside WIRE_LEAF, mint that leaf FOOT grade.\n\
                 \nWhat those select is one of three acts.\n\
                 \n1. A box with no material gains the lot — a private CA, the server, client \
                 and window leaves, and the `address` file. The engine's own boot performs this \
                 for a box that has none, aimed at loopback on a kernel-chosen port, so the \
                 explicit act is for a server another machine dials by name.\n\
                 \n2. On a box that already holds material, a stated WIRE_HOST says where this \
                 engine listens: it re-issues THE SERVER LEAF over the CA already there and \
                 writes `address` to match. No CA is founded and no other leaf is touched, so \
                 nothing already carried away stops verifying — restart the engine and it binds \
                 what was stated. That is the act for a box whose own boot provisioned it: a \
                 self-provisioned `127.0.0.1:0` is a request only the listener learns the answer \
                 to, and stating an endpoint over it costs one signature rather than a rotation. \
                 Stating nothing is refused, because a bare re-run asks for nothing this act \
                 could perform; FORCE=1 is the rotation and is never implicit.\n\
                 \n3. WIRE_LEAF issues one extra client leaf over the CA already here — no CA \
                 founded, no address written, no other leaf touched. That is the leaf a visiting \
                 box participates as; carry it, its key and `ca.pem` to that box by hand and \
                 place them in its `wire/workspaces/<workspace>/` as `client.pem`, `client.key` \
                 and `ca.pem`, beside an `address` naming this engine. That directory is named \
                 for the WORKSPACE the client will address, never for the common name the leaf \
                 was issued under — a seat routes a gesture by the workspace it names, so a \
                 directory named for the leaf is a channel nothing can reach. The common name \
                 INSIDE the certificate is the identity, not the basename, so the rename costs \
                 nothing. WIRE_FOOT=1 mints it foot grade: a tool host that may advertise its \
                 tools, take the invocations addressed to it and complete them, and say nothing \
                 else — a thrall refuses to open on anything else, and an operator-grade leaf \
                 carried to a tool host is turned away by it. Unset is operator grade, which is \
                 a seat AND a tool host under one name.\n\
                 \nA leaf this act issues is registered in NO workspace, and an advertisement \
                 reaches only the workspaces its client is registered in — so enrol the same \
                 common name from a seat in the workspace it should serve (`/enroll <name> \
                 [foot]`), which adopts this leaf rather than issuing a second one.\n\
                 \nIt shells to `openssl`: provisioning is the operator's out-of-channel act \
                 and yog links no certificate library.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: crate::fixture::verb::SUBCMD,
        usage: "yog fixture [state]",
        summary: "lay a named, deterministic world state for a client harness to dial",
        detail: "Write one of a fixed roster of world states into a scratch data root and \
                 print, as one JSON object, everything a harness needs to dial an engine \
                 booted on it: the root, the address, the CA and the client leaf. Bare, it \
                 lists the roster. Booting is the caller's — `XDG_DATA_HOME=<root> yog` — \
                 because the caller is the one that has to kill it, and tearing down is `rm \
                 -rf <root>`. `FIXTURE_ROOT` names the root (the default is a stable \
                 path under this box's cache root), and `WIRE_HOST`/`WIRE_PORT` state the address \
                 the material is minted for, exactly as `wire-certs` reads them; with no \
                 port stated a free one is taken from the kernel, because a `127.0.0.1:0` in \
                 the material is a request only the listener ever learns the answer to. It \
                 REFUSES a root that overlaps this box's own yog data root in either \
                 direction: a lay wipes its root before it writes. The `hold` list it \
                 answers with names the fds a harness keeps open for the run to make a \
                 streaming conversation read as a live model call — a live call is derived \
                 from an open descriptor, so no tree on disk can be one by itself.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: crate::world::hatch::ENV_SUBCMD,
        usage: "yog env [--ws WORKSPACE]",
        summary: "print the world's environment (`eval \"$(yog env)\"`)",
        detail: "Print one shell `export` line per world override, quoted so `eval` reproduces \
                 each value byte-for-byte. `eval \"$(yog env)\"` drops the calling shell into \
                 yog's nested world, where a bare `bl`/`litany`/`bz` is the world's own shim \
                 into yog's embedded substrate. `--ws WORKSPACE` also stands that workspace's \
                 wall, which is what a `bz` needs: providers, sign-ins and the model cache \
                 belong to a workspace, and without one bz refuses rather than reaching the \
                 machine's own. Prints only; it starts nothing.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: crate::world::hatch::EXEC_SUBCMD,
        usage: "yog exec [--cwd DIR] [--ws WORKSPACE] <cmd…>",
        summary: "run one command inside the composed world",
        detail: "Run exactly one command with the world's environment standing, and exit with \
                 that command's own code. `--ws WORKSPACE` also stands that workspace's wall, \
                 which is how a shell **on this box** signs a workspace in: `yog exec --ws \
                 WORKSPACE bz --login --provider NAME --browser` writes the credential into \
                 that workspace and nowhere else. A seat that is not on this box says \
                 `/login <provider>` instead, which runs the same thing here. The leading flags are yog's; every argument from the command \
                 word on belongs to the command. Bad usage exits 2, a command that could not \
                 be spawned exits 127.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: crate::control::SUBCMD,
        usage: "yog tool-control",
        summary: "",
        detail: "The capability control an embedded litany consults before each granted tool \
                 invocation: it speaks a line protocol over stdin/stdout and is spawned with \
                 no arguments beyond this word. Nothing types it by hand.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: "gesture",
        usage: "yog gesture <gesture>",
        summary: "cross the control boundary: a JSON envelope or a /slash line",
        detail: "Deposit one gesture into the running world's inbox and print the reply. The \
                 payload is a JSON envelope or a `/slash` line; `--ws / --agent / --project / \
                 --as` state the context a terminal has no selection for. `yog gesture --help` \
                 lists every gesture and `yog gesture --help <command>` is one gesture's page.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: "litany",
        usage: "yog litany <argv…>",
        summary: "the embedded litany, in yog's own process",
        detail: "Run litany's own verb surface in this process, against the nested world. The \
                 argv after the word is litany's, so `yog litany --help` is litany's own usage.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: "bl",
        usage: "yog bl <argv…>",
        summary: "the embedded balls, on the composed world's store",
        detail: "Run balls' own verb surface in this process, against the world's store and \
                 landing. The argv after the word is balls', so `yog bl --help` is balls' own \
                 usage.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: "bz",
        usage: "yog bz <argv…>",
        summary: "the embedded brazen (sign in with `yog exec --ws WORKSPACE bz --login …`)",
        detail: "Run brazen's own surface in this process. Providers, sign-ins and the model \
                 cache belong to a workspace, so every route but a discovery probe needs a \
                 workspace wall standing — a bare `yog bz --login` outside one is refused \
                 rather than signing in somewhere shared. Name the workspace with a hatch: \
                 `yog exec --ws WORKSPACE bz --login --provider NAME --browser`, or stand the \
                 wall for a whole shell with `eval \"$(yog env --ws WORKSPACE)\"`. The argv \
                 after the word is brazen's, so `yog bz --help` is brazen's own usage.",
        surface: Surface::Machine,
    },
];

/// Answer a help ask at the argv surface, or `None` when this argv is not one
/// — the two shapes of the module doc, read above the router. A namespace
/// that **owns its argv** ([`super::Namespace::owns_argv`]) has its `--help`
/// deliberately **not** answered here: its argv is the tool's, and the arm
/// answers it (world-free) with the tool's own page. One that does not (bl-4667
/// — a class with no member since the severance, and kept because the
/// classification is exhaustive rather than because a word is in it) is answered
/// from [`COMMANDS`] like any of yog's own subcommands.
pub(super) fn answer(argv: &[String]) -> Option<String> {
    let word = argv.get(1)?.as_str();
    if matches!(word, "--help" | "-h" | "help") {
        let about = argv.get(2).map(String::as_str);
        return Some(about.and_then(page).unwrap_or_else(super::usage));
    }
    if super::Namespace::from_arg(word).is_some_and(super::Namespace::owns_argv) {
        return None;
    }
    matches!(argv.get(2).map(String::as_str), Some("--help" | "-h"))
        .then(|| page(word))
        .flatten()
}

/// One command's page — its usage line and paragraph, in [`render`]'s
/// single-row shape. `None` for a word [`COMMANDS`] does not carry.
fn page(verb: &str) -> Option<String> {
    COMMANDS
        .iter()
        .find(|row| row.verb == verb)
        .map(|row| render(&[*row]))
}

/// Whether a namespace's argv is a bare **discovery probe** — the tool asking
/// itself what it is, which reads the interface and never the world (§8.5).
///
/// Recognized only when the whole argv *is* the flag, which is the same
/// narrowness §8.5 already pins on the line reader (*"the flag form is
/// recognized only when the tail is exactly the flag"*). Nothing can then
/// precede the token to make it an option's value, so this can never mistake
/// `bz --system --help` or a prompt after `--` for a probe — and a foreign
/// crate's argv grammar never has to be restated here to be sure.
pub(crate) fn is_discovery(args: &[String]) -> bool {
    matches!(
        args,
        [only] if matches!(only.as_str(), "--help" | "-h" | "--version" | "-V" | "--skill")
    )
}
