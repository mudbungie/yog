//! The **argv seat's** help (DESIGN §8.5's higher-order rule, applied above the
//! router — bl-52ed). Help is asked *about* a command, so it is read once here
//! rather than by each command: `yog --help|-h|help [command]` is the whole
//! surface or one command's page, and `yog <command> --help|-h` is that
//! command's page. Nothing routes, composes, spawns, parks or writes first.
//!
//! **Which commands answer here, and which answer themselves.** yog's own
//! subcommands ([`crate::world::hatch`]'s two, `headless`, `tool-control` —
//! and `tool-host`, which routes as a namespace but takes no argv, bl-4667)
//! have no interface of their own to consult, so their page is [`COMMANDS`]
//! and this module prints it. A namespace that owns its argv
//! ([`super::Namespace::owns_argv`]: `gesture`, `lernie`, `bl`, `bz`, `seat`,
//! balls' two plugin seams): its `--help` is the embedded tool's own answer,
//! so the ask is passed through to the arm, which must reach it **world-free**
//! — that is why [`is_discovery`] exists and why `bl`'s shim converge and
//! `bz`'s wall gate both step aside for a probe.
//!
//! [`COMMANDS`] is the single source: the top-level roster ([`super::usage`]),
//! every per-command page, and the parity tests all read this one list. A row
//! with an empty `summary` is answerable but **unadvertised** — the same rule
//! [`NAMESPACES`](super::NAMESPACES) applies to balls' two plugin binaries.

use crate::boundary::help::{HelpRow, render};

#[cfg(test)]
mod tests;

/// Every top-level word yog answers help about — its usage line, the one line
/// the roster gives it (empty = unadvertised), and the paragraph a `--help`
/// prints. `verb` is the const its dispatcher routes on, so a page cannot name
/// a word that does not run.
pub(super) const COMMANDS: &[HelpRow] = &[
    HelpRow {
        verb: crate::boundary::SERVE_SUBCMD,
        usage: "yog serve",
        summary: "the same engine with no window: worker, watcher, gesture consumer, wire",
        detail: "Boot the one engine a window boots — the derivation worker, the watch bridge, \
                 the gestures-inbox consumer and the mTLS wire listener — with no face beside \
                 it, and park until a signal ends the process. State is write-through, so \
                 nothing pends at exit. This is the engine `yog gesture` deposits into and \
                 `yog seat` connects to; boot mints whatever wire material this box lacks, \
                 aimed at loopback on a kernel-chosen port, so the listener is up with no \
                 operator act behind it. `yog wire-certs` is the act for everything wider — a \
                 stated address another machine dials, or a rotation. A mint or a bind that \
                 fails is said on stderr and the engine runs on without a wire.",
    },
    HelpRow {
        verb: crate::wire::SEAT_SUBCMD,
        usage: "yog seat <gesture>",
        summary: "cross the control boundary over the wire: the same envelope or /slash line",
        detail: "Send one gesture to an engine over mTLS and print the reply. The payload and \
                 flags are `yog gesture`'s exactly — a JSON envelope or a `/slash` line, with \
                 `--ws / --agent / --project / --as / --prepared` stating the context a \
                 terminal has no selection for — because the wire transports the boundary and \
                 adds nothing to it. The engine, the certificate this machine presents and the \
                 CA both ends verify against come from the wire material an operator \
                 provisioned (`make wire-certs`); with none, this refuses and says so. Use \
                 `yog gesture` instead from inside the world: disk is the bus there.",
    },
    HelpRow {
        verb: crate::wire::HOST_SUBCMD,
        usage: "yog tool-host",
        summary: "be a tool host: run this machine's tools for an engine over the wire",
        detail: "Presents what `<yog-data-root>/tools.json` says this machine can run — the \
                 same document, with `command`/`cwd` dropped, so what is offered and what can \
                 actually be run cannot drift — then waits for work and runs it. Each element \
                 is `{\"name\", \"description\", \"input_schema\", \"command\": [argv…], \
                 \"cwd\"?}`; the argv is spawned directly, never through a shell, and the \
                 invocation's JSON arrives on the command's stdin exactly as a local tool's \
                 does. One invocation at a time; a tool that has not answered in two minutes \
                 is terminated and the capture says so. The engine, the certificate this \
                 machine presents and the CA both ends verify against come from the wire \
                 material an operator provisioned (`make wire-certs`); with none, or with no \
                 config, this refuses and says which. When the channel fails it exits, naming \
                 the failure — restarting it is the supervision this machine already has.",
    },
    HelpRow {
        verb: crate::wire::provision::verb::SUBCMD,
        usage: "yog wire-certs",
        summary: "mint this box's wire certificates: a local CA and its server/client leaves",
        detail: "Write a private CA and the server, client and window leaves into the wire \
                 material directory, plus the one `address` file naming what the engine binds \
                 and a seat dials. The engine's own boot already does this for a box that has \
                 none, aimed at loopback, so this is the act for everything else: a server \
                 another machine dials by name (`WIRE_HOST=engine.example.com WIRE_PORT=7737`), \
                 a different directory (`WIRE_DIR`), or a rotation (`FORCE=1`). It refuses to \
                 overwrite otherwise, because a rotation distrusts every certificate already \
                 issued and every seat holding one stops connecting. `WIRE_LEAF=<common-name>` \
                 asks for the other act instead: issue ONE extra client leaf under that name, \
                 over the CA already here — no CA, no address, no other leaf. That is the leaf \
                 a visiting box participates as; carry it, its key and `ca.pem` to that box by \
                 hand and place them in its `wire/workspaces/<leaf>/` as `client.pem`, \
                 `client.key` and `ca.pem`, beside an `address` naming this engine. The common \
                 name INSIDE the certificate is the identity, not the basename, so the rename \
                 costs nothing. It shells to `openssl`: provisioning is the operator's \
                 out-of-channel act and yog links no certificate library.",
    },
    HelpRow {
        verb: crate::world::hatch::ENV_SUBCMD,
        usage: "yog env [--ws WORKSPACE]",
        summary: "print the world's environment (`eval \"$(yog env)\"`)",
        detail: "Print one shell `export` line per world override, quoted so `eval` reproduces \
                 each value byte-for-byte. `eval \"$(yog env)\"` drops the calling shell into \
                 yog's nested world, where a bare `bl`/`lernie`/`bz` is the world's own shim \
                 into yog's embedded substrate. `--ws WORKSPACE` also stands that workspace's \
                 wall, which is what a `bz` needs: providers, sign-ins and the model cache \
                 belong to a workspace, and without one bz refuses rather than reaching the \
                 machine's own. Prints only; it starts nothing.",
    },
    HelpRow {
        verb: crate::world::hatch::EXEC_SUBCMD,
        usage: "yog exec [--cwd DIR] [--ws WORKSPACE] <cmd…>",
        summary: "run one command inside the composed world",
        detail: "Run exactly one command with the world's environment standing, and exit with \
                 that command's own code. `--ws WORKSPACE` also stands that workspace's wall, \
                 which is how a headless seat signs in: `yog exec --ws WORKSPACE bz --login \
                 --provider NAME --browser` writes the credential into that workspace and \
                 nowhere else. The leading flags are yog's; every argument from the command \
                 word on belongs to the command. Bad usage exits 2, a command that could not \
                 be spawned exits 127.",
    },
    HelpRow {
        verb: crate::control::SUBCMD,
        usage: "yog tool-control",
        summary: "",
        detail: "The capability control an embedded lernie consults before each granted tool \
                 invocation: it speaks a line protocol over stdin/stdout and is spawned with \
                 no arguments beyond this word. Nothing types it by hand.",
    },
    HelpRow {
        verb: "gesture",
        usage: "yog gesture <gesture>",
        summary: "cross the control boundary: a JSON envelope or a /slash line",
        detail: "Deposit one gesture into the running world's inbox and print the reply. The \
                 payload is a JSON envelope or a `/slash` line; `--ws / --agent / --project / \
                 --as` state the context a terminal has no selection for. `yog gesture --help` \
                 lists every gesture and `yog gesture --help <command>` is one gesture's page.",
    },
    HelpRow {
        verb: "lernie",
        usage: "yog lernie <argv…>",
        summary: "the embedded lernie, in yog's own process",
        detail: "Run lernie's own verb surface in this process, against the nested world. The \
                 argv after the word is lernie's, so `yog lernie --help` is lernie's own usage.",
    },
    HelpRow {
        verb: "bl",
        usage: "yog bl <argv…>",
        summary: "the embedded balls, on the composed world's store",
        detail: "Run balls' own verb surface in this process, against the world's store and \
                 landing. The argv after the word is balls', so `yog bl --help` is balls' own \
                 usage.",
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
    },
];

/// Answer a help ask at the argv surface, or `None` when this argv is not one
/// — the two shapes of the module doc, read above the router. A namespace
/// that **owns its argv** ([`super::Namespace::owns_argv`]) has its `--help`
/// deliberately **not** answered here: its argv is the tool's, and the arm
/// answers it (world-free) with the tool's own page. One that does not
/// (`tool-host`, bl-4667) is answered from [`COMMANDS`] like any of yog's own
/// subcommands.
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
