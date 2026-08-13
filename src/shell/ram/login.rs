//! The Login pane's surface RAM (DESIGN §5.3, §8.3) — its own file per §12's
//! line budget, and a real seam: everything else in [`super`] is viewport
//! ephemera, while this holder owns a *wall-bound* seam bundle (the in-process
//! `bz` runner, the credentials dir a presence read folds against, and the
//! spawn layer a sign-in is fired with, §16.2 as amended) — so it is one field
//! of [`WallRam`](super::WallRam), one holder per workspace.

use std::path::PathBuf;

use crate::xdg::Env;

/// The Login pane's surface RAM (§5.3, §8.3): the runner that answers
/// brazen's effective provider table in-process (§16.7 W10), the **rendered
/// rows** that table and the credential-presence read last derived to
/// (#20/#21/#22), the credentials dir that read folds against, and the one
/// active streamed `bz --login` run, if any. Instance-local by nature — a
/// device code is for the human at *this* keyboard — and discarded on exit.
pub struct LoginHolder {
    pub bz: crate::config_edit::brazen::RealBzRunner,
    pub rows: Vec<crate::config_edit::brazen::ProviderRowView>,
    pub run: Option<crate::login::LoginRun>,
    /// The wall layer (§16.2 as amended) the streamed `bz --login` spawn is
    /// fired with, so the credential it writes lands in *this* workspace.
    pub wall: Vec<(String, String)>,
    /// The workspace this holder belongs to — the §8.3 run-by-hand fallback's
    /// `--ws` (bl-b589). The wall *pairs* above name the wall root, and a wall
    /// root cannot be turned back into the workspace path that keyed it, so the
    /// path is carried rather than recovered.
    pub workspace: Option<PathBuf>,
    creds_dir: PathBuf,
}

impl LoginHolder {
    /// Fold every seam from `wall` — the workspace's lensed env (§16.2 as
    /// amended) this holder belongs to — **and ask** (bl-e290): the holder is
    /// never born empty, so the Login surface — the pane and the auth-failed
    /// banner alike — is populated the moment it is first painted, with no click
    /// standing between the operator and the roster.
    ///
    /// Providers, the credential column and the sign-in spawn fold together
    /// because they are one sphere's settings; outside a wall the fold is empty
    /// and the roster reads empty, which is the truth rather than the machine's
    /// own sign-ins. There is no re-seating verb: a holder is one wall's for
    /// life, and a focus change swaps the whole holder (bl-5894) — which is what
    /// keeps a running `bz --login` with the workspace it will write into.
    pub fn new(wall: &Env, workspace: Option<&std::path::Path>) -> Self {
        let mut holder = Self {
            workspace: workspace.map(std::path::Path::to_path_buf),
            bz: crate::config_edit::brazen::RealBzRunner::resolve(wall),
            rows: Vec::new(),
            run: None,
            wall: crate::world::wall::pairs_of(wall),
            creds_dir: crate::config_edit::brazen::BrazenPaths::of(wall)
                .map(|p| p.credentials_dir)
                .unwrap_or_default(),
        };
        holder.ask();
        holder
    }

    /// Re-derive the rendered rows: brazen's effective table
    /// ([`RealBzRunner::providers`], offline and in-process since §16.7 W10)
    /// paired with the §5.1 #22 credential-presence read, folded to words by
    /// the one shared derivation (`row_views`, bl-402f). A **gesture** read,
    /// never a per-frame one (§7.2): construction, and the pane's `↻` — which
    /// is the freshness answer for both halves at once (a config edit, and the
    /// credential a completed sign-in just wrote).
    pub fn ask(&mut self) {
        use crate::config_edit::brazen::{BzRunner, credential_presence, row_views};
        let rows = self.bz.providers();
        let creds = credential_presence(&self.creds_dir, &rows, &crate::config_edit::RealFileIo);
        self.rows = row_views(&rows, &creds);
    }
}
