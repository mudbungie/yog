//! The six `bz` routes, each wired to the wall's seams (DESIGN §16.7 W10,
//! §16.2 as amended). Split from [`super`] at §12's line budget along the seam
//! the module already had: the parent is the *entry* — the wall fold, the
//! snapshot brazen reads, the route decision — and this is what each route is
//! made of. `bz`'s own `main.rs` builds the same bundles at the same six sites.

use std::io::{Read, Write};

use brazen::native::{
    HttpTransport, LoopbackReceiver, RealPacer, SystemBrowserLauncher, SystemClock, TcpBind,
    random_token, stash_root,
};

use super::store::{WallCredStore, WallModelCache};
use crate::config_edit::brazen::BrazenPaths;
use crate::xdg::Env;

/// The four data-plane seam impls, built **once per invocation** from the
/// wall's own [`BrazenPaths`] — every route that reaches a provider wires
/// exactly these. Two of the four are yog's ([`store`]): rooting the credential
/// store and model cache at a path is what puts them inside the workspace wall,
/// where brazen's process-env-folding shim could not go (§16.2 as amended).
pub(super) struct Seams {
    transport: HttpTransport,
    store: WallCredStore,
    cache: WallModelCache,
    clock: SystemClock,
}

impl Seams {
    pub(super) fn wire(paths: &BrazenPaths, env: &Env) -> Self {
        Self {
            transport: HttpTransport::new(),
            store: WallCredStore::new(paths.credentials_dir.clone(), env.clone()),
            cache: WallModelCache::new(paths.models_cache_dir.clone()),
            clock: SystemClock,
        }
    }
}

/// The data plane: the five impure seams behind [`brazen::Host`], then
/// `brazen::run`.
pub(super) fn data_plane(
    args: brazen::Args,
    seams: &Seams,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let stash = brazen::ReplayStash::new(stash_root());
    let host = brazen::Host {
        transport: &seams.transport,
        store: &seams.store,
        cache: &seams.cache,
        clock: &seams.clock,
        stash: &stash,
    };
    brazen::run(args, stdin, stdout, stderr, &host)
}

/// `--list-models`: the data-plane seams with a listing's output shape.
pub(super) fn list_models(
    args: &brazen::Args,
    seams: &Seams,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let mut io = brazen::ListIo {
        stdout,
        stderr,
        transport: &seams.transport,
        store: &seams.store,
        cache: &seams.cache,
        clock: &seams.clock,
    };
    brazen::list_models(args, &mut io)
}

/// `--list-providers`: the effective provider table. Offline by construction —
/// [`brazen::ProvidersIo`] carries no transport at all, so yog's provider rows
/// cannot make a network call even by mistake. Its credential column is the
/// wall's, so a row reads *signed in* only where this workspace signed it in.
pub(super) fn list_providers(
    args: &brazen::Args,
    seams: &Seams,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let mut io = brazen::ProvidersIo {
        stdout,
        stderr,
        store: &seams.store,
    };
    brazen::list_providers(args, &mut io)
}

/// `--count-tokens`: consumes a request, so it takes the reader too.
pub(super) fn count_tokens(
    args: &brazen::Args,
    seams: &Seams,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let mut io = brazen::CountIo {
        stdout,
        stderr,
        transport: &seams.transport,
        store: &seams.store,
        cache: &seams.cache,
        clock: &seams.clock,
    };
    brazen::count_tokens(args, stdin, &mut io)
}

/// `--serve`: the data-plane seams plus the TCP bind seam and the replay stash.
pub(super) fn serve(
    args: &brazen::Args,
    seams: &Seams,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let (bind, stash) = (TcpBind, brazen::ReplayStash::new(stash_root()));
    let mut io = brazen::ServeIo {
        stdout,
        stderr,
        bind: &bind,
        transport: &seams.transport,
        store: &seams.store,
        cache: &seams.cache,
        clock: &seams.clock,
        stash: &stash,
    };
    brazen::serve(args, &mut io)
}

/// `--login`: the interactive seams (browser, loopback receiver, pacer) plus
/// the OS RNG for the PKCE verifier and CSRF state. The receiver is built
/// UNBOUND — the browser flow binds it once the row's redirect port resolves,
/// the device flow never touches it. The credential it writes lands in the
/// wall, so signing in here signs in *this workspace* and no other.
pub(super) fn login(
    args: &brazen::Args,
    seams: &Seams,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let (browser, pacer, receiver) = (SystemBrowserLauncher, RealPacer, LoopbackReceiver::new());
    let (verifier, state) = (random_token(), random_token());
    let mut io = brazen::LoginIo {
        stdout,
        stderr,
        transport: &seams.transport,
        store: &seams.store,
        clock: &seams.clock,
        browser: &browser,
        receiver: &receiver,
        pacer: &pacer,
        verifier: &verifier,
        state: &state,
    };
    brazen::login(args, &mut io)
}
