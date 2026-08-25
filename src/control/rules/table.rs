//! **The rows themselves** — policy shipped as data, in match order. Split from
//! the rule grammar at §12's budget on the seam the module's own doc draws:
//! [`super`] states what a row *is* and how one is read, this is the one
//! shipped instance of it, and a per-workspace `capability.yaml` is another
//! (`super::super::policy`).
//!
//! **Order is the policy**, so these rows are one list and stay one list: first
//! match wins, most specific first, and a table cut in two would be a table
//! whose match order depends on which half was consulted.

use super::super::classify::Effect;
use super::{ANY, Reach, Rule};
use Effect::{Destructive, OpenWorld, Read, Secret, TargetWrite};
use Reach::{ByRoot, Fixed};

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
