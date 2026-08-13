//! The **shipped bash ruleset** (VISION §4.11 item 1): the default classification
//! of a shell segment's program into the effect vocabulary.
//!
//! This table is *policy shipped as data*, not logic. It is the severable
//! default the per-workspace policy config replaces or extends (bl-765d); until
//! that config exists, absence is these rows — the `cadence.yaml` pattern, one
//! layer down.
//!
//! Three properties decide the shape:
//!
//! 1. **First match wins, most specific first.** `git push --force` must be read
//!    as destructive before `git push` is read as open-world and long before
//!    plain `git` is read as a target write.
//! 2. **Unmatched is open-world.** There is no catch-all row, deliberately: a
//!    program the table does not name is a program whose reach nobody has
//!    stated, so it takes the widest class short of loss rather than a narrow
//!    one it might not deserve. That is why interpreters (`python`, `node`,
//!    `sh`) are absent — an interpreter's reach is its script's, which no rule
//!    can see.
//! 3. **Some rows classify by operand.** `rm` inside the writable root is the
//!    ordinary work of a build tree; the same `rm` outside it is loss the repo
//!    cannot give back. One row says both ([`Reach::ByRoot`]).
//!
//! The rows that pass without qualification are the ones VISION §4.11 item 8
//! names honestly: `cargo` and `make` execute arbitrary code from the tree they
//! build, and they pass anyway, because refusing them refuses the job. The wall
//! that stops *that* is OS confinement, later and platform-explicit.

use super::classify::Effect;
use Effect::{Destructive, OpenWorld, Read, Secret, TargetWrite};
use Reach::{ByRoot, Fixed};

/// How a rule decides its segment's reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// The class regardless of operands.
    Fixed(Effect),
    /// The class depends on where the segment's path operands land: `inside`
    /// when every one of them resolves inside the writable root, `outside`
    /// otherwise.
    ByRoot { inside: Effect, outside: Effect },
}

/// One row of the ruleset: `(program, qualifying words, reach)`. A tuple rather
/// than a struct so a row stays one readable line — the table is data, and a
/// field-per-line rendering of ninety rows buries the policy it states.
///
/// - **program** — matched against the segment's leading word's basename.
/// - **qualifying words** — all must appear in the segment for the row to bite.
///   A short flag matches inside a bundle, so `-f` bites on `-fd` (see
///   [`super::bash::has_word`]); [`ANY`] means the program alone decides.
/// - **reach** — the class the row yields.
pub type Rule = (&'static str, &'static [&'static str], Reach);

/// No qualifying words: the program alone decides the row.
pub const ANY: &[&str] = &[];

/// The shipped ruleset, in match order.
pub const DEFAULT: &[Rule] = &[
    // ---- credentials and environment -------------------------------------
    // `bz` spends the workspace's own brazen credentials outside lernie's own
    // budget derivation, which is precisely the drone-spending-secrets case.
    ("bz", ANY, Fixed(Secret)),
    ("env", ANY, Fixed(Secret)),
    ("printenv", ANY, Fixed(Secret)),
    ("gpg", ANY, Fixed(Secret)),
    ("ssh-add", ANY, Fixed(Secret)),
    ("ssh-keygen", ANY, Fixed(Secret)),
    ("security", ANY, Fixed(Secret)),
    ("keyctl", ANY, Fixed(Secret)),
    // ---- irreversible loss ------------------------------------------------
    ("git", &["push", "--force"], Fixed(Destructive)),
    ("git", &["push", "-f"], Fixed(Destructive)),
    ("git", &["push", "--force-with-lease"], Fixed(Destructive)),
    ("git", &["reset", "--hard"], Fixed(Destructive)),
    ("git", &["clean", "-f"], Fixed(Destructive)),
    ("git", &["branch", "-D"], Fixed(Destructive)),
    ("git", &["update-ref", "-d"], Fixed(Destructive)),
    ("git", &["filter-branch"], Fixed(Destructive)),
    ("git", &["gc", "--prune"], Fixed(Destructive)),
    ("shred", ANY, Fixed(Destructive)),
    ("mkfs", ANY, Fixed(Destructive)),
    ("dd", ANY, Fixed(Destructive)),
    (
        "rm",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: Destructive,
        },
    ),
    (
        "rmdir",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: Destructive,
        },
    ),
    // ---- past the root and the world --------------------------------------
    ("git", &["push"], Fixed(OpenWorld)),
    ("git", &["fetch"], Fixed(OpenWorld)),
    ("git", &["pull"], Fixed(OpenWorld)),
    ("git", &["clone"], Fixed(OpenWorld)),
    ("git", &["remote"], Fixed(OpenWorld)),
    ("curl", ANY, Fixed(OpenWorld)),
    ("wget", ANY, Fixed(OpenWorld)),
    ("nc", ANY, Fixed(OpenWorld)),
    ("ssh", ANY, Fixed(OpenWorld)),
    ("scp", ANY, Fixed(OpenWorld)),
    ("rsync", ANY, Fixed(OpenWorld)),
    ("sudo", ANY, Fixed(OpenWorld)),
    ("gh", ANY, Fixed(OpenWorld)),
    ("cargo", &["publish"], Fixed(OpenWorld)),
    ("cargo", &["install"], Fixed(OpenWorld)),
    // ---- the world's own substrates, through their gated verbs ------------
    ("bl", ANY, Fixed(TargetWrite)),
    ("lernie", ANY, Fixed(TargetWrite)),
    // ---- building the target ----------------------------------------------
    ("git", ANY, Fixed(TargetWrite)),
    ("cargo", ANY, Fixed(TargetWrite)),
    ("make", ANY, Fixed(TargetWrite)),
    (
        "sed",
        &["-i"],
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "mkdir",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "touch",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "cp",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "mv",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "ln",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "tee",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "chmod",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "truncate",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    (
        "patch",
        ANY,
        ByRoot {
            inside: TargetWrite,
            outside: OpenWorld,
        },
    ),
    // ---- observation ------------------------------------------------------
    ("ls", ANY, Fixed(Read)),
    ("cat", ANY, Fixed(Read)),
    ("head", ANY, Fixed(Read)),
    ("tail", ANY, Fixed(Read)),
    ("grep", ANY, Fixed(Read)),
    ("rg", ANY, Fixed(Read)),
    ("find", ANY, Fixed(Read)),
    ("wc", ANY, Fixed(Read)),
    ("file", ANY, Fixed(Read)),
    ("stat", ANY, Fixed(Read)),
    ("diff", ANY, Fixed(Read)),
    ("cmp", ANY, Fixed(Read)),
    ("which", ANY, Fixed(Read)),
    ("pwd", ANY, Fixed(Read)),
    ("echo", ANY, Fixed(Read)),
    ("printf", ANY, Fixed(Read)),
    ("true", ANY, Fixed(Read)),
    ("false", ANY, Fixed(Read)),
    ("test", ANY, Fixed(Read)),
    ("sort", ANY, Fixed(Read)),
    ("uniq", ANY, Fixed(Read)),
    ("cut", ANY, Fixed(Read)),
    ("tr", ANY, Fixed(Read)),
    ("sed", ANY, Fixed(Read)),
    ("awk", ANY, Fixed(Read)),
    ("jq", ANY, Fixed(Read)),
    ("basename", ANY, Fixed(Read)),
    ("dirname", ANY, Fixed(Read)),
    ("realpath", ANY, Fixed(Read)),
    ("readlink", ANY, Fixed(Read)),
    ("date", ANY, Fixed(Read)),
    ("du", ANY, Fixed(Read)),
    ("df", ANY, Fixed(Read)),
    ("ps", ANY, Fixed(Read)),
    ("whoami", ANY, Fixed(Read)),
    ("uname", ANY, Fixed(Read)),
    ("hostname", ANY, Fixed(Read)),
    ("sleep", ANY, Fixed(Read)),
    ("md5sum", ANY, Fixed(Read)),
    ("sha256sum", ANY, Fixed(Read)),
];

/// Path fragments whose mere appearance in a segment makes it a secret access,
/// whatever the program is: `cat ~/.ssh/id_rsa` is a read of key material, and
/// the read rows must not let it through.
pub const SECRET_FRAGMENTS: &[&str] = &[
    ".ssh",
    ".aws",
    ".netrc",
    ".gnupg",
    "id_rsa",
    "id_ed25519",
    "credentials",
    // brazen's config is credential-adjacent and deliberately shared with the
    // ambient world (§16.2), so it is named by path rather than by tool.
    ".config/brazen",
];
