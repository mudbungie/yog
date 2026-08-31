//! **The client registry** (REMOTE §1.5, §2, §4, §7; bl-8bbc): the durable
//! server-side fact that client C participates in workspace W, and the
//! per-client home everything else about C hangs off.
//!
//! **A registration is a file, and its existence IS the fact.** REMOTE §2:
//! *"Server-side, in the world, a file — the file religion applies; the wire
//! only ever transports the gesture that writes it."* So there is no registry
//! document to parse, no list to keep in step and no last-writer-wins window:
//!
//! ```text
//! <yog-state-root>/clients/<client>/pane.json          the §7 pane-of-glass facts
//! <yog-state-root>/clients/<client>/tools.json         the §5 advertised set (bl-4e08)
//! <yog-state-root>/clients/<client>/workspaces/<name>   one empty file per registration
//! ```
//!
//! Registering is creating the file, **revocation is deleting it** (§4), and
//! the registered set is the directory listing. Nothing is stored that could be
//! computed: the client is the directory name, the workspace is the file name.
//! It sits at yog's own state root beside `ui.json` because it is yog's durable
//! state, not the operator's key material — the `wire/` directory holds what
//! yog can never mint (§8), and this holds what only yog ever writes.
//!
//! **First registration is an operator-written file** (§4). `mkdir -p
//! <state-root>/clients/<name>/workspaces && touch …/<workspace>` is the whole
//! bootstrap, and it is the same act that provisions the certificates —
//! out-of-channel, by ruling (§1.4). There is no first-client flow, because the
//! general path with an operator-seeded input is not a case of its own.
//!
//! **`local` is the reserved identity of every in-world caller** — the window,
//! the `gestures/` deposit inbox, `yog gesture`. They carry no certificate and
//! are not scoped (§3: each intake the religion of its domain), but they do own
//! a pane document, so they need a name for it. [`Client::parse`] refuses
//! `local` exactly as it refuses `.` and `..`: all three are names the layout
//! has already spent, which is one rule rather than three special cases.

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

/// The certificate leaf name → client identity fold (REMOTE §2).
pub mod leaf;
/// Which clients hold a live connection right now (REMOTE §5) — RAM, never a
/// file, because presence changes with every network blip.
pub mod presence;
/// The workspace's registered clients joined with their presence and their
/// advertised sets (REMOTE §5) — the one derivation both seats render.
pub mod roster;
/// What a tool host advertises, and the document it lands in (REMOTE §5).
pub mod tools;

/// The reserved identity of the window and every other in-world caller.
pub const LOCAL: &str = "local";
/// The registry root's leaf under yog's state root.
pub const CLIENTS: &str = "clients";
/// One client's §7 pane-of-glass document.
pub const PANE: &str = "pane.json";
/// The directory whose entries are one client's registrations.
pub const WORKSPACES: &str = "workspaces";

/// One client identity (REMOTE §2): a certificate leaf name, or [`LOCAL`].
///
/// It is a **path component by construction** — every reachable constructor
/// validates — because the identity names a directory, and a name that could
/// contain a separator would let a certificate address the filesystem.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Client(String);

/// **The default caller is the in-world one.** Every intake that carries no
/// certificate is `local` (§3), so the default is the identity rather than an
/// empty string nothing could resolve — a gesture always has a caller.
impl Default for Client {
    fn default() -> Self {
        Self::local()
    }
}

impl Client {
    /// The in-world identity: the window, the deposit inbox, `yog gesture`.
    pub fn local() -> Self {
        Self(LOCAL.to_owned())
    }

    /// A wire client's identity, or the refusal naming the token. Refuses
    /// anything that is not a plain path component
    /// ([`naming::is_component`](crate::naming::is_component)) and the reserved
    /// [`LOCAL`] name, for the one reason the module doc gives.
    pub fn parse(name: &str) -> Result<Self, String> {
        if !crate::naming::is_component(name) || name == LOCAL {
            return Err(format!("unusable client identity {name:?}"));
        }
        Ok(Self(name.to_owned()))
    }

    /// This identity as text — the directory it names, and the word a `list`
    /// of the workspace's clients will print (§5).
    pub fn name(&self) -> String {
        self.0.clone()
    }

    /// True for the in-world identity. Scoping asks this and nothing else:
    /// an in-world caller sees the whole world (§3), a wire client sees its
    /// registrations (§4).
    pub fn is_local(&self) -> bool {
        self.0 == LOCAL
    }
}

/// This client's directory: `<state-root>/clients/<client>`.
pub fn dir(state_root: &Path, client: &Client) -> PathBuf {
    state_root.join(CLIENTS).join(&client.0)
}

/// This client's §7 pane-of-glass document — server-held, so any two seats of
/// one client converge on the same panel sizes and view knobs.
pub fn pane(state_root: &Path, client: &Client) -> PathBuf {
    dir(state_root, client).join(PANE)
}

/// This client's registration directory — one file per workspace it
/// participates in.
pub fn registrations(state_root: &Path, client: &Client) -> PathBuf {
    dir(state_root, client).join(WORKSPACES)
}

/// The workspace names `client` is registered in (§4). A client with no
/// directory, an unreadable one, or one holding nothing reads as the empty set
/// — the general path with no input, and the posture a fresh server has for
/// every certificate the operator has not yet seated.
pub fn registered(state_root: &Path, client: &Client) -> BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(registrations(state_root, client)) else {
        return BTreeSet::new();
    };
    entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

/// Record that `client` participates in `workspace` (§4) — the auto-registering
/// half of a create, and the gesture an operator performs with `touch`.
/// Idempotent: an existing registration is rewritten to the same empty file.
///
/// The file is empty on purpose. A registration has no content — it is the
/// **pair**, and the pair is the path.
pub fn register(state_root: &Path, client: &Client, workspace: &str) -> io::Result<()> {
    if !crate::naming::is_component(workspace) {
        return Err(io::Error::other(format!(
            "unusable workspace name {workspace:?}"
        )));
    }
    let dir = registrations(state_root, client);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(workspace), [])
}

#[cfg(test)]
mod tests;

/// The engine-side invocation hand-off (REMOTE §5, bl-024b) — a queue per
/// client and a slot per invocation, RAM beside [`presence`] for its reason.
///
/// **Declared last, and out of order on purpose.** Every other `mod` sits at
/// the top, and adding a fifth there shifted every byte below it — which cost
/// this file its 100 % coverage floor, llvm-cov attributing a phantom
/// uncovered region to `impl Client {`. That is the hazard
/// `rules/locks-outside-state.yml` records twice over, met here by the remedy
/// `state.rs` already uses: append below every line that was here before.
pub mod mailbox;

/// **The local window's own client identity** (REMOTE §1.2, §4.1; bl-ae05):
/// `yog-window`, the subject common name yog's own mint puts on the window leaf
/// ([`Role::Window`](crate::wire::material::Role::Window)) — and therefore the
/// name the engine reads off the certificate the window presents, the directory
/// its §7 pane document lives in, and the name a registration seats. One
/// spelling, here, because an identity's home is the registry.
///
/// **The module doc and [`LOCAL`]'s own doc still say `local` is the
/// window's**, and they are deliberately not edited — the reason [`mailbox`] is
/// declared at the bottom of this file. Editing a byte above shifts every byte
/// below it, and llvm-cov then mis-attributes a phantom *uncovered* region onto
/// `impl Default for Client`, costing this file its 100 % floor (measured
/// twice). **REMOTE §4.1 is the authority and it is narrower than those lines
/// read: `local` is the certificate-less in-world callers — the `gestures/`
/// deposit inbox and `yog gesture` — and the window is not among them.**
pub const WINDOW: &str = "yog-window";

/// The window's identity as a [`Client`] — total, unlike [`Client::parse`],
/// because the name is yog's own const rather than a token read off a
/// certificate. A free function rather than a second `impl Client` block, for
/// the reason above: an added `impl` draws a phantom region onto itself
/// besides, which is the same hazard `state.rs` meets the same way.
pub fn window() -> Client {
    Client(WINDOW.to_owned())
}

/// **What a certificate authorizes** (REMOTE §4.2, bl-1dd3) — the grade its
/// subject carries, and the [`Peer`] an intake answers as.
///
/// Declared at the bottom beside [`mailbox`] and for its reason exactly:
/// adding a `mod` at the top of this file shifts every byte below it, and
/// llvm-cov then draws a phantom uncovered region onto `impl Client`.
pub mod peer;

pub use peer::{Grade, Peer};

/// **Enrollment's two values** (REMOTE §1.4 as amended, §4.2; bl-f4e3) — what
/// an operator asks for when a device joins, and what the engine answers with.
///
/// Declared at the bottom beside [`peer`] and [`mailbox`], for their reason.
pub mod enroll;
